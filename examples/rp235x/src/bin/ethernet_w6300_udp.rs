//! This example implements a UDP server listening on port 1234 and echoing back the data.
//!
//! Example written for the [`WIZnet W6300-EVB-Pico2`](https://wiznet.io/products/evaluation-boards/w6300-evb-pico2) board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W6300;
use embassy_net_wiznet::*;
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::spi::Spi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W6300,
        ExclusiveDevice<Spi<'static, PIO0, 0, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
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
    spi_cfg.frequency = 15_000_000;

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let (miso, mosi, clk) = (p.PIN_19, p.PIN_18, p.PIN_17);

    let spi = Spi::new(&mut common, sm0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_cfg);

    let cs = Output::new(p.PIN_16, Level::High);
    let w6300_int = Input::new(p.PIN_15, Pull::Up);
    let w6300_reset = Output::new(p.PIN_22, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs, Delay),
        w6300_int,
        w6300_reset,
    )
    .await
    .unwrap();
    spawner.spawn(unwrap!(ethernet_task(runner)));

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
    spawner.spawn(unwrap!(net_task(runner)));

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

async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}
