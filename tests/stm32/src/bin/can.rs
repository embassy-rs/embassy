#![no_std]
#![no_main]

// required-features: can

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

mod can_common;
use can_common::*;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    let options = TestOptions {
        max_latency: Duration::from_micros(1200),
        max_buffered: 2,
    };

    let can = peri!(p, CAN);
    let tx = peri!(p, CAN_TX);
    let mut rx = peri!(p, CAN_RX);

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(rx.reborrow(), Pull::Up);
    core::mem::forget(rx_pin);

    let mut can = embassy_stm32::can::Can::new(can, rx, tx, Irqs);

    info!("Configuring can...");

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        // .set_bit_timing(0x001c0003)
        .set_bitrate(1_000_000);

    can.enable().await;

    info!("Can configured");

    run_can_tests(&mut can, &options).await;

    // Test again with a split
    let (mut tx, mut rx) = can.split();
    run_split_can_tests(&mut tx, &mut rx, &options).await;

    info!("Test OK");

    cortex_m::asm::bkpt();
}
