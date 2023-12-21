#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, Id, StandardId};
use embassy_stm32::can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::peripherals::CAN;
use embassy_stm32::{bind_interrupts, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

// This example is configured to work with real CAN transceivers on B8/B9.
// See other examples for loopback.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    // Set alternate pin mapping to B8/B9
    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));

    let mut can = Can::new(p.CAN, p.PB8, p.PB9, Irqs);

    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut()
        .modify_config()
        .set_loopback(false)
        .set_silent(false)
        .leave_disabled();

    can.set_bitrate(250_000);

    can.enable().await;

    let mut i: u8 = 0;
    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), [i]);
        can.write(&tx_frame).await;

        match can.read().await {
            Ok(env) => match env.frame.id() {
                Id::Extended(id) => {
                    defmt::println!("Extended Frame id={:x}", id.as_raw());
                }
                Id::Standard(id) => {
                    defmt::println!("Standard Frame id={:x}", id.as_raw());
                }
            },
            Err(err) => {
                defmt::println!("Error {}", err);
            }
        }
        i += 1;
    }
}
