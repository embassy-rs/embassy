#![no_std]
#![no_main]

use core::num::{NonZeroU16, NonZeroU8};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, CanTx, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN3;
use embassy_stm32::{bind_interrupts, can};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN3_RX0 => Rx0InterruptHandler<CAN3>;
    CAN3_RX1 => Rx1InterruptHandler<CAN3>;
    CAN3_SCE => SceInterruptHandler<CAN3>;
    CAN3_TX => TxInterruptHandler<CAN3>;
});

#[embassy_executor::task]
pub async fn send_can_message(tx: &'static mut CanTx<'static>) {
    loop {
        let frame = Frame::new_data(unwrap!(StandardId::new(0 as _)), &[0]).unwrap();
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
    let rx_pin = Input::new(p.PA15.reborrow(), Pull::Up);
    core::mem::forget(rx_pin);

    static CAN: StaticCell<Can<'static>> = StaticCell::new();
    let can = CAN.init(Can::new(p.CAN3, p.PA8, p.PA15, Irqs));
    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_bit_timing(can::util::NominalBitTiming {
            prescaler: NonZeroU16::new(2).unwrap(),
            seg1: NonZeroU8::new(13).unwrap(),
            seg2: NonZeroU8::new(2).unwrap(),
            sync_jump_width: NonZeroU8::new(1).unwrap(),
        }) // http://www.bittiming.can-wiki.info/
        .set_loopback(true);

    can.enable().await;

    let (tx, mut rx) = can.split();

    static CAN_TX: StaticCell<CanTx<'static>> = StaticCell::new();
    let tx = CAN_TX.init(tx);
    spawner.spawn(send_can_message(tx)).unwrap();

    loop {
        let envelope = rx.read().await.unwrap();
        println!("Received: {:?}", envelope);
    }
}
