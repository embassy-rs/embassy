#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use defmt::*;
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

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(&mut p.PA11, Pull::Up);
    core::mem::forget(rx_pin);

    let mut can = Can::new(p.CAN1, p.PA11, p.PA12, Irqs);

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_bit_timing(0x001c0003) // http://www.bittiming.can-wiki.info/
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        .enable();

    let mut i: u8 = 0;
    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), [i]);
        unwrap!(nb::block!(can.transmit(&tx_frame)));
        while !can.is_transmitter_idle() {}
        let rx_frame = unwrap!(nb::block!(can.receive()));
        info!("loopback frame {=u8}", unwrap!(rx_frame.data())[0]);
        i += 1;
    }
}
