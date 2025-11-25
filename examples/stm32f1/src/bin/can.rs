#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::frame::Envelope;
use embassy_stm32::can::{
    Can, Fifo, Frame, Id, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler, filter,
};
use embassy_stm32::peripherals::CAN;
use embassy_stm32::{Config, bind_interrupts};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

// This example is configured to work with real CAN transceivers on B8/B9.
// See other examples for loopback.

fn handle_frame(env: Envelope, read_mode: &str) {
    match env.frame.id() {
        Id::Extended(id) => {
            defmt::println!(
                "{} Extended Frame id={:x} {:02x}",
                read_mode,
                id.as_raw(),
                env.frame.data()
            );
        }
        Id::Standard(id) => {
            defmt::println!(
                "{} Standard Frame id={:x} {:02x}",
                read_mode,
                id.as_raw(),
                env.frame.data()
            );
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    // Set alternate pin mapping to B8/B9
    embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_can1_remap(2));

    static RX_BUF: StaticCell<embassy_stm32::can::RxBuf<10>> = StaticCell::new();
    static TX_BUF: StaticCell<embassy_stm32::can::TxBuf<10>> = StaticCell::new();

    let mut can = Can::new(p.CAN, p.PB8, p.PB9, Irqs);

    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, filter::Mask32::accept_all());

    can.modify_config()
        .set_loopback(false)
        .set_silent(false)
        .set_bitrate(250_000);

    can.enable().await;
    let mut i: u8 = 0;

    /*
       // Example for using buffered Tx and Rx without needing to
       // split first as is done below.
       let mut can = can.buffered(
           TX_BUF.init(embassy_stm32::can::TxBuf::<10>::new()),
           RX_BUF.init(embassy_stm32::can::RxBuf::<10>::new()));
       loop {
           let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), &[i, 0, 1, 2, 3, 4, 5, 6]).unwrap();
           can.write(&tx_frame).await;

           match can.read().await {
               Ok((frame, ts)) => {
                   handle_frame(Envelope { ts, frame }, "Buf");
               }
               Err(err) => {
                   defmt::println!("Error {}", err);
               }
           }
           i = i.wrapping_add(1);
       }

    */
    let (mut tx, mut rx) = can.split();

    // This example shows using the wait_not_empty API before try read
    while i < 3 {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), &[i, 0, 1, 2, 3, 4, 5, 6]).unwrap();
        tx.write(&tx_frame).await;

        rx.wait_not_empty().await;
        let env = rx.try_read().unwrap();
        handle_frame(env, "Wait");
        i += 1;
    }

    // This example shows using the full async non-buffered API
    while i < 6 {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), &[i, 0, 1, 2, 3, 4, 5, 6]).unwrap();
        tx.write(&tx_frame).await;

        match rx.read().await {
            Ok(env) => {
                handle_frame(env, "NoBuf");
            }
            Err(err) => {
                defmt::println!("Error {}", err);
            }
        }
        i += 1;
    }

    // This example shows using buffered RX and TX. User passes in desired buffer (size)
    // It's possible this way to have just RX or TX buffered.
    let mut rx = rx.buffered(RX_BUF.init(embassy_stm32::can::RxBuf::<10>::new()));
    let mut tx = tx.buffered(TX_BUF.init(embassy_stm32::can::TxBuf::<10>::new()));

    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), &[i, 0, 1, 2, 3, 4, 5, 6]).unwrap();
        tx.write(&tx_frame).await;

        match rx.read().await {
            Ok(envelope) => {
                handle_frame(envelope, "Buf");
            }
            Err(err) => {
                defmt::println!("Error {}", err);
            }
        }
        i = i.wrapping_add(1);
    }
}
