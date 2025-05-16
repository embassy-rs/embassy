//! This example implements a LAN ping scan with the ping utilities in the icmp module of embassy-net.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico`](https://docs.wiznet.io/Product/iEthernet/W5500/w5500-evb-pico) board.

#![no_std]
#![no_main]

use core::net::Ipv4Addr;
use core::ops::Not;
use core::str::FromStr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::icmp::ping::{PingManager, PingParams};
use embassy_net::icmp::PacketMetadata;
use embassy_net::{Ipv4Cidr, Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_time::{Delay, Duration};
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
    let gateway = cfg.gateway.unwrap();
    let mask = cfg.address.netmask();
    let lower_bound = (gateway.to_bits() & mask.to_bits()) + 1;
    let upper_bound = gateway.to_bits() | mask.to_bits().not();
    let addr_range = lower_bound..=upper_bound;

    // Then we can use it!
    let mut rx_buffer = [0; 256];
    let mut tx_buffer = [0; 256];
    let mut rx_meta = [PacketMetadata::EMPTY];
    let mut tx_meta = [PacketMetadata::EMPTY];

    // Create the ping manager instance
    let mut ping_manager = PingManager::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    let addr = "192.168.8.1"; // Address to ping to
                              // Create the PingParams with the target address
    let mut ping_params = PingParams::new(Ipv4Addr::from_str(addr).unwrap());
    // (optional) Set custom properties of the ping
    ping_params.set_payload(b"Hello, Ping!"); // custom payload
    ping_params.set_count(1); // ping 1 times per ping call
    ping_params.set_timeout(Duration::from_millis(500)); // wait .5 seconds instead of 4

    info!("Online hosts in {}:", Ipv4Cidr::from_netmask(gateway, mask).unwrap());
    let mut total_online_hosts = 0u32;
    for addr in addr_range {
        let ip_addr = Ipv4Addr::from_bits(addr);
        // Set the target address in the ping params
        ping_params.set_target(ip_addr);
        // Execute the ping with the given parameters and wait for the reply
        match ping_manager.ping(&ping_params).await {
            Ok(time) => {
                info!("{} is online\n- latency: {}ms\n", ip_addr, time.as_millis());
                total_online_hosts += 1;
            }
            _ => continue,
        }
    }
    info!("Ping scan complete, total online hosts: {}", total_online_hosts);
}

async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}
