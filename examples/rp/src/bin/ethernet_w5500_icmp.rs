//! This example pings the default gateway with a raw socket, and reports the result with defmt.
//!
//! A raw socket carries whole IP packets, so the example builds the IPv4 and ICMP headers
//! itself. The same socket can carry any other ICMP message, e.g. `Destination unreachable`.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico`](https://docs.wiznet.io/Product/iEthernet/W5500/w5500-evb-pico) board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::raw::{IpProtocol, IpVersion, RawSocket};
use embassy_net::wire::{IPV4_HEADER_LEN, Icmpv4Message, Icmpv4Packet, Ipv4Packet};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::{Delay, Instant, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

type ExclusiveSpiDevice = ExclusiveDevice<Spi<'static, Async>, Output<'static>, Delay>;

#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, W5500, ExclusiveSpiDevice, Input<'static>, Output<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;
    let (miso, mosi, clk) = (p.PIN_16, p.PIN_19, p.PIN_18);
    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, spi_cfg);
    let cs = Output::new(p.PIN_17, Level::High);
    let w5500_int = Input::new(p.PIN_21, Pull::Up);
    let w5500_reset = Output::new(p.PIN_20, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());
    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs, Delay),
        w5500_int,
        w5500_reset,
    )
    .await
    .unwrap();
    spawner.spawn(unwrap!(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Device<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)));
    iface.set_dhcpv4(Some(Default::default()));

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    info!("Waiting for DHCP...");
    iface.wait_config_up().await;
    let lease = unwrap!(iface.dhcpv4_lease());
    let local_addr = lease.address.address();
    info!("IP address: {:?}", local_addr);

    // Then we can use it! A raw socket receives whole IPv4 packets carrying ICMP.
    let socket = unwrap!(RawSocket::new(stack, Some(IpVersion::Ipv4), Some(IpProtocol::Icmp)));

    // Identifier used to recognize our own echo replies.
    let ident = 42;
    let payload = b"Hello, icmp!";
    let icmp_len = 8 + payload.len();
    let total_len = IPV4_HEADER_LEN + icmp_len;

    let gateway = unwrap!(lease.router);
    let mut buf = [0u8; 64];
    {
        let mut ip = unwrap!(Ipv4Packet::new_checked(&mut buf[..total_len]));
        ip.set_version(4);
        ip.set_header_len(IPV4_HEADER_LEN as u8);
        ip.set_dscp(0);
        ip.set_ecn(0);
        ip.set_total_len(total_len as u16);
        ip.set_ident(0);
        ip.clear_flags();
        ip.set_dont_frag(true);
        ip.set_frag_offset(0);
        ip.set_hop_limit(64);
        ip.set_next_header(IpProtocol::Icmp);
        ip.set_src_addr(local_addr);
        ip.set_dst_addr(gateway);
        ip.fill_checksum();

        let mut icmp = unwrap!(Icmpv4Packet::new_checked(ip.payload_mut()));
        icmp.set_msg_type(Icmpv4Message::EchoRequest);
        icmp.set_msg_code(0);
        icmp.set_echo_ident(ident);
        icmp.set_echo_seq_no(0);
        icmp.data_mut().copy_from_slice(payload);
        icmp.fill_checksum();
    }

    // Send the packet, and remember when, to measure the latency of the reply.
    let start = Instant::now();
    unwrap!(socket.send(&buf[..total_len]).await);

    // Receive and log the reply. The socket sees every ICMP packet, so skip the ones
    // that aren't the echo reply we're waiting for.
    let mut rx = [0u8; 128];
    loop {
        let n = unwrap!(socket.recv(&mut rx).await);
        let Ok(ip) = Ipv4Packet::new_checked(&mut rx[..n]) else {
            continue;
        };
        if ip.src_addr() != gateway {
            continue;
        }
        let src = ip.src_addr();
        let Ok(icmp) = Icmpv4Packet::new_checked(&mut rx[IPV4_HEADER_LEN..n]) else {
            continue;
        };
        if icmp.msg_type() != Icmpv4Message::EchoReply || icmp.echo_ident() != ident {
            continue;
        }
        info!(
            "Received {:?} from {} in {}ms",
            icmp.data(),
            src,
            start.elapsed().as_millis()
        );
        break;
    }

    loop {
        Timer::after_secs(10).await;
    }
}
