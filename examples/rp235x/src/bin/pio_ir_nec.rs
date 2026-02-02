//! This example shows sending and receiving NEC IR frames using the PIO module of the RP235x.
//!
//! Connect an IR receiver with a low-pass filter (such as the VS1838b) to pin 14 and an IR led
//! with a series resistor between pin 25 and ground.

#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio_programs::ir_nec::{NecFrame, PioIrNecRx, PioIrNecRxProgram, PioIrNecTx, PioIrNecTxProgram};
use embassy_rp::{bind_interrupts, pio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pio::Pio {
        mut common,
        sm0,
        sm1,
        sm2,
        ..
    } = pio::Pio::new(p.PIO0, Irqs);

    let rx_program = PioIrNecRxProgram::new(&mut common);
    let rx = PioIrNecRx::new(&mut common, sm0, p.PIN_14, &rx_program);

    let tx_program = PioIrNecTxProgram::new(&mut common, 7);
    let tx = PioIrNecTx::new(&mut common, sm1, sm2, p.PIN_25, &tx_program);

    spawner.spawn(send(tx).unwrap());
    spawner.spawn(receive(rx).unwrap());
}

#[embassy_executor::task]
async fn send(mut tx: PioIrNecTx<'static, PIO0, 1, 2>) {
    let mut data = 0;
    loop {
        let frame = NecFrame { address: 0xAA, data };
        tx.write(frame).await;
        data += 1;
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::task]
async fn receive(mut rx: PioIrNecRx<'static, PIO0, 0>) {
    loop {
        info!("{}", rx.read().await);
    }
}
