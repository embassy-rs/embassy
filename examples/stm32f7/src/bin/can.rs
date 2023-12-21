#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::bxcan::{Fifo, Frame, StandardId};
use embassy_stm32::can::{
    Can, CanTx, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN3;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN3_RX0 => Rx0InterruptHandler<CAN3>;
    CAN3_RX1 => Rx1InterruptHandler<CAN3>;
    CAN3_SCE => SceInterruptHandler<CAN3>;
    CAN3_TX => TxInterruptHandler<CAN3>;
});

#[embassy_executor::task]
pub async fn send_can_message(tx: &'static mut CanTx<'static, 'static, CAN3>) {
    loop {
        let frame = Frame::new_data(unwrap!(StandardId::new(0 as _)), [0]);
        tx.write(&frame).await;
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(&mut p.PA15, Pull::Up);
    core::mem::forget(rx_pin);

    static CAN: StaticCell<Can<'static, CAN3>> = StaticCell::new();
    let can = CAN.init(Can::new(p.CAN3, p.PA8, p.PA15, Irqs));
    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut()
        .modify_config()
        .set_bit_timing(0x001c0001) // http://www.bittiming.can-wiki.info/
        .set_loopback(true)
        .enable();

    let (tx, mut rx) = can.split();

    static CAN_TX: StaticCell<CanTx<'static, 'static, CAN3>> = StaticCell::new();
    let tx = CAN_TX.init(tx);
    spawner.spawn(send_can_message(tx)).unwrap();

    loop {
        let envelope = rx.read().await.unwrap();
        println!("Received: {:?}", envelope);
    }
}
