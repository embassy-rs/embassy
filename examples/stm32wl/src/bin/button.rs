#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::SharedData;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init_primary(Default::default(), &SHARED_DATA);

    let button = Input::new(p.PA0, Pull::Up);
    let mut led1 = Output::new(p.PB15, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB9, Level::High, Speed::Low);

    loop {
        if button.is_high() {
            led1.set_high();
            led2.set_low();
        } else {
            led1.set_low();
            led2.set_high();
        }
    }
}
