#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
};
use embassy_stm32::peripherals::CAN1;
use embassy_stm32::{Config, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let mut can = Can::new(p.CAN1, p.PA11, p.PA12, Irqs);

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        .set_bitrate(250_000);

    can.enable().await;
    println!("CAN enabled");

    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();
    loop {
        let frame = Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");

        _ = can.write(&frame).await;

        match can.read().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- {}ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(err) => error!("Error in frame: {}", err),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 2 {
            break;
        }
    }
}
