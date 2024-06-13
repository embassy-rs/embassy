#![no_std]
#![no_main]

// required-features: fdcan

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{bind_interrupts, can, Config};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

mod can_common;
use can_common::*;

bind_interrupts!(struct Irqs2 {
    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
});
bind_interrupts!(struct Irqs1 {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[cfg(feature = "stm32h563zi")]
fn options() -> (Config, TestOptions) {
    info!("H563 config");
    (
        config(),
        TestOptions {
            max_latency: Duration::from_micros(1200),
            max_buffered: 3,
        },
    )
}

#[cfg(any(feature = "stm32h755zi", feature = "stm32h753zi"))]
fn options() -> (Config, TestOptions) {
    use embassy_stm32::rcc;
    info!("H75 config");
    let mut c = config();
    c.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    c.rcc.mux.fdcansel = rcc::mux::Fdcansel::HSE;
    (
        c,
        TestOptions {
            max_latency: Duration::from_micros(1200),
            max_buffered: 3,
        },
    )
}

#[cfg(any(feature = "stm32h7a3zi"))]
fn options() -> (Config, TestOptions) {
    use embassy_stm32::rcc;
    info!("H7a config");
    let mut c = config();
    c.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    c.rcc.mux.fdcansel = rcc::mux::Fdcansel::HSE;
    (
        c,
        TestOptions {
            max_latency: Duration::from_micros(1200),
            max_buffered: 3,
        },
    )
}

#[cfg(any(feature = "stm32h7s3l8"))]
fn options() -> (Config, TestOptions) {
    use embassy_stm32::rcc;
    let mut c = config();
    c.rcc.mux.fdcansel = rcc::mux::Fdcansel::HSE;
    (
        c,
        TestOptions {
            max_latency: Duration::from_micros(1200),
            max_buffered: 3,
        },
    )
}

#[cfg(any(feature = "stm32g491re"))]
fn options() -> (Config, TestOptions) {
    info!("G4 config");
    (
        config(),
        TestOptions {
            max_latency: Duration::from_micros(500),
            max_buffered: 6,
        },
    )
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //let peripherals = embassy_stm32::init(config());

    let (config, options) = options();
    let peripherals = embassy_stm32::init(config);

    let mut can = can::CanConfigurator::new(peripherals.FDCAN1, peripherals.PB8, peripherals.PB9, Irqs1);
    let mut can2 = can::CanConfigurator::new(peripherals.FDCAN2, peripherals.PB12, peripherals.PB13, Irqs2);

    // 250k bps
    can.set_bitrate(250_000);
    can2.set_bitrate(250_000);

    can.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );
    can2.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    let mut can = can.into_internal_loopback_mode();
    let mut can2 = can2.into_internal_loopback_mode();

    run_can_tests(&mut can, &options).await;
    run_can_tests(&mut can2, &options).await;

    info!("CAN Configured");

    // Test again with a split
    let (mut tx, mut rx, _props) = can.split();
    let (mut tx2, mut rx2, _props) = can2.split();
    run_split_can_tests(&mut tx, &mut rx, &options).await;
    run_split_can_tests(&mut tx2, &mut rx2, &options).await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
