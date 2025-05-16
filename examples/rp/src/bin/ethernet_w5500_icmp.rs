//! This example implements an echo (ping) with an ICMP Socket and using defmt to report the results.
//!
//! Although there is a better way to execute pings using the child module ping of the icmp module,
//! this example allows for other icmp messages like `Destination unreachable` to be sent aswell.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico`](https://docs.wiznet.io/Product/iEthernet/W5500/w5500-evb-pico) board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::icmp::{ChecksumCapabilities, IcmpEndpoint, IcmpSocket, Icmpv4Packet, Icmpv4Repr, PacketMetadata};
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_time::{Delay, Instant, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use rand::RngCore;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type ExclusiveSpiDevice = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>;

#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, W5500, ExclusiveSpiDevice, Input<'static>, Output<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;
    let (miso, mosi, clk) = (p.PIN_16, p.PIN_19, p.PIN_18);
    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_cfg);
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
    unwrap!(spawner.spawn(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    unwrap!(spawner.spawn(net_task(runner)));

    info!("Waiting for DHCP...");
    let cfg = wait_for_config(stack).await;
    let local_addr = cfg.address.address();
    info!("IP address: {:?}", local_addr);

    // Then we can use it!
    let mut rx_buffer = [0; 256];
    let mut tx_buffer = [0; 256];
    let mut rx_meta = [PacketMetadata::EMPTY];
    let mut tx_meta = [PacketMetadata::EMPTY];

    // Identifier used for the ICMP socket
    let ident = 42;

    // Create and bind the socket
    let mut socket = IcmpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(IcmpEndpoint::Ident(ident)).unwrap();

    // Create the repr for the packet
    let icmp_repr = Icmpv4Repr::EchoRequest {
        ident,
        seq_no: 0,
        data: b"Hello, icmp!",
    };

    // Send the packet and store the starting instant to mesure latency later
    let start = socket
        .send_to_with(icmp_repr.buffer_len(), cfg.gateway.unwrap(), |buf| {
            // Create and populate the packet buffer allocated by `send_to_with`
            let mut icmp_packet = Icmpv4Packet::new_unchecked(buf);
            icmp_repr.emit(&mut icmp_packet, &ChecksumCapabilities::default());
            Instant::now() // Return the instant where the packet was sent
        })
        .await
        .unwrap();

    // Recieve and log the data of the reply
    socket
        .recv_from_with(|(buf, addr)| {
            let packet = Icmpv4Packet::new_checked(buf).unwrap();
            info!(
                "Recieved {:?} from {} in {}ms",
                packet.data(),
                addr,
                start.elapsed().as_millis()
            );
        })
        .await
        .unwrap();

    loop {
        Timer::after_secs(10).await;
    }
}

async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}
