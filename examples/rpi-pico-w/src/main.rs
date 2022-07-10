#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, concat_bytes)]

use core::slice;

use defmt::{assert, assert_eq, panic, *};
use embassy::executor::Spawner;
use embassy_rp::gpio::{Flex, Level, Output, Pin};
use embassy_rp::Peripherals;
use {defmt_rtt as _, panic_probe as _};


macro_rules! forever {
    ($val:expr) => {{
        type T = impl Sized;
        static FOREVER: Forever<T> = Forever::new();
        FOREVER.put_with(move || $val)
    }};
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_25, p.PIN_29, p.PIN_24);
    //let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_0, p.PIN_1, p.PIN_2);

    let mut driver = cyw43::Driver::new(
        Output::new(pwr, Level::Low),
        Output::new(cs, Level::High),
        Output::new(clk, Level::Low),
        Flex::new(dio),
    );

    driver.init().await;

    loop {}
}
