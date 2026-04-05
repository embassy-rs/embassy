//! This example implements a UDP server listening on port 1234 and echoing back the data.
//!
//! Example written for the [`WIZnet W6300-EVB-Pico2`](https://wiznet.io/products/evaluation-boards/w6300-evb-pico2) board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::qspi::exclusive::ExclusiveDevice;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W6300;
use embassy_net_wiznet::wiznet_spi_interface::WiznetQspiBus;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::qspi::Qspi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Delay;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W6300,
        WiznetQspiBus<ExclusiveDevice<Qspi<'static, PIO0, 0, Async>, Output<'static>, Delay>>,
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
    let rp_peripherals = embassy_rp::init(Default::default());

    let mut rng = RoscRng;
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 25_000_000;

    let Pio { mut common, sm0, .. } = Pio::new(rp_peripherals.PIO0, Irqs);

    let clk = rp_peripherals.PIN_17;
    let qspi_0 = rp_peripherals.PIN_18;
    let qspi_1 = rp_peripherals.PIN_19;
    let qspi_2 = rp_peripherals.PIN_20;
    let qspi_3 = rp_peripherals.PIN_21;

    let qspi = Qspi::new(
        &mut common,
        sm0,
        clk,
        qspi_0,
        qspi_1,
        qspi_2,
        qspi_3,
        rp_peripherals.DMA_CH0,
        rp_peripherals.DMA_CH1,
        Irqs,
        spi_cfg,
    );

    let cs = Output::new(rp_peripherals.PIN_16, Level::High);
    let w6300_int = Input::new(rp_peripherals.PIN_15, Pull::Up);
    let w6300_reset = Output::new(rp_peripherals.PIN_22, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        WiznetQspiBus(ExclusiveDevice::new(qspi, cs, Delay).unwrap()),
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
