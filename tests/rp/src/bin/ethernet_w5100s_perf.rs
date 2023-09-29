#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"w5100s-evb-pico");
teleprobe_meta::timeout!(120);

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5100S;
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
        W5100S,
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

    perf_client::run(
        stack,
        perf_client::Expected {
            down_kbps: 500,
            up_kbps: 500,
            updown_kbps: 300,
        },
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
