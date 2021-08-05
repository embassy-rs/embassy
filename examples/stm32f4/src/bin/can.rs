#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use cortex_m_rt::entry;
use embassy_stm32::bxcan::{Can, Frame, StandardId};
use embassy_stm32::dbgmcu::Dbgmcu;
use example_common::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        Dbgmcu::enable_all();
    }

    let p = embassy_stm32::init(Default::default());

    let mut can = Can::new(p.CAN1, p.PA11, p.PA12);

    can.modify_config().set_loopback(true);
    unwrap!(nb::block!(can.enable()));

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
