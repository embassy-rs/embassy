//! This example implements a UDP server listening on port 1234 and echoing back the data.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico`](https://www.wiznet.io/product-item/w5500-evb-pico/) board.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_17, PIN_20, PIN_21, SPI0};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use rand::RngCore;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static, PIN_17>, Delay>,
        Input<'static, PIN_21>,
        Output<'static, PIN_20>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static>>) -> ! {
    stack.run().await
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
    let state = make_static!(State::<8, 8>::new());
    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs, Delay),
        w5500_int,
        w5500_reset,
    )
    .await;
    unwrap!(spawner.spawn(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        make_static!(StackResources::<2>::new()),
        seed
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(&stack)));

    info!("Waiting for DHCP...");
    let cfg = wait_for_config(stack).await;
    let local_addr = cfg.address.address();
    info!("IP address: {:?}", local_addr);

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut buf = [0; 4096];
    loop {
        let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
        socket.bind(1234).unwrap();

        loop {
            let (n, ep) = socket.recv_from(&mut buf).await.unwrap();
            if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                info!("rxd from {}: {}", ep, s);
            }
            socket.send_to(&buf[..n], ep).await.unwrap();
        }
    }
}

async fn wait_for_config(stack: &'static Stack<Device<'static>>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}
