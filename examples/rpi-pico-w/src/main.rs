#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, concat_bytes)]

use core::slice;

use defmt::{assert, assert_eq, panic, *};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_rp::gpio::{Flex, Level, Output, Pin};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::Peripherals;
use {defmt_rtt as _, panic_probe as _};

macro_rules! forever {
    ($val:expr) => {{
        type T = impl Sized;
        static FOREVER: Forever<T> = Forever::new();
        FOREVER.put_with(move || $val)
    }};
}

#[embassy::task]
async fn wifi_task(runner: cyw43::Runner<'static, PIN_23, PIN_25, PIN_29, PIN_24>) -> ! {
    runner.run().await
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_25, p.PIN_29, p.PIN_24);
    //let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_0, p.PIN_1, p.PIN_2);

    let state = forever!(cyw43::State::new());
    let (mut control, runner) = cyw43::new(
        state,
        Output::new(pwr, Level::Low),
        Output::new(cs, Level::High),
        Output::new(clk, Level::Low),
        Flex::new(dio),
    )
    .await;

    spawner.spawn(wifi_task(runner)).unwrap();

    control.init().await;

    let ssid = "MikroTik-951589";
    control.join(ssid).await;
}
