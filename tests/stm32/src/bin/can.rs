#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// required-features: can

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, StandardId};
use embassy_stm32::can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
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

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.set_bitrate(1_000_000);
    can.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        // .set_bit_timing(0x001c0003)
        .enable();

    info!("Can configured");

    let mut i: u8 = 0;
    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), [i]);

        info!("Transmitting frame...");
        can.write(&tx_frame).await;

        info!("Receiving frame...");
        let (time, rx_frame) = can.read().await.unwrap();

        info!("loopback time {}", time);
        info!("loopback frame {=u8}", rx_frame.data().unwrap()[0]);

        i += 1;
        if i > 10 {
            break;
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
