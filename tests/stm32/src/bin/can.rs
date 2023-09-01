#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// required-features: can

#[path = "../common.rs"]
mod common;
use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, StandardId};
use embassy_stm32::can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_time::{Duration, Instant};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(config());
    info!("Hello World!");

    // HW is connected as follows:
    // PB13 -> PD0
    // PB12 -> PD1

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(&mut p.PD0, Pull::Up);
    core::mem::forget(rx_pin);

    let mut can = Can::new(p.CAN1, p.PD0, p.PD1, Irqs);

    info!("Configuring can...");

    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.set_bitrate(1_000_000);
    can.as_mut()
        .modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        // .set_bit_timing(0x001c0003)
        .enable();

    info!("Can configured");

    let mut i: u8 = 0;
    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), [i]);

        info!("Transmitting frame...");
        let tx_ts = Instant::now();
        can.write(&tx_frame).await;

        let envelope = can.read().await.unwrap();
        info!("Frame received!");

        info!("loopback time {}", envelope.ts);
        info!("loopback frame {=u8}", envelope.frame.data().unwrap()[0]);

        let latency = envelope.ts.saturating_duration_since(tx_ts);
        info!("loopback latency {} us", latency.as_micros());

        // Theoretical minimum latency is 55us, actual is usually ~80us
        const MIN_LATENCY: Duration = Duration::from_micros(50);
        const MAX_LATENCY: Duration = Duration::from_micros(150);
        assert!(
            MIN_LATENCY <= latency && latency <= MAX_LATENCY,
            "{} <= {} <= {}",
            MIN_LATENCY,
            latency,
            MAX_LATENCY
        );

        i += 1;
        if i > 10 {
            break;
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
