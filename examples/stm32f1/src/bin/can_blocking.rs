#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::frame::Envelope;
use embassy_stm32::can::{
    filter, Can, Fifo, Frame, Id, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler,
};
use embassy_stm32::peripherals::CAN;
use embassy_stm32::{bind_interrupts, Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

// This example demonstrates blocking CAN operations on STM32F1
// Configured to work with real CAN transceivers on B8/B9.

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
        .set_loopback(true) // Enable loopback for testing
        .set_silent(false)
        .set_bitrate(250_000);

    can.blocking_enable();

    info!("CAN Blocking Example Started");

    // Example 1: Basic blocking operations with split CAN
    let (mut tx, mut rx) = can.split();

    for i in 0..3 {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(0x123 + i)), &[i as u8, 0x01, 0x02, 0x03]).unwrap();

        // Blocking write
        info!("Blocking write frame {}", i);
        tx.blocking_write(&tx_frame);

        // Blocking read
        match rx.blocking_read() {
            Ok(env) => {
                handle_frame(env, "Blocking");
            }
            Err(err) => {
                error!("Blocking read error: {}", err);
            }
        }
    }

    // Example 2: Buffered blocking operations
    let mut rx = rx.buffered(RX_BUF.init(embassy_stm32::can::RxBuf::<10>::new()));
    let mut tx = tx.buffered(TX_BUF.init(embassy_stm32::can::TxBuf::<10>::new()));

    for i in 3..6 {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(0x200 + i)), &[i as u8, 0x10, 0x20, 0x30]).unwrap();

        // Buffered blocking write
        info!("Buffered blocking write frame {}", i);
        tx.blocking_write(&tx_frame);

        // Buffered blocking read
        match rx.blocking_read() {
            Ok(envelope) => {
                handle_frame(envelope, "BufferedBlocking");
            }
            Err(err) => {
                error!("Buffered blocking read error: {}", err);
            }
        }
    }

    info!("CAN Blocking Example Completed - all CAN operations were blocking!");

    // Keep the program running (Embassy executor still needed for interrupt handling)
    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
