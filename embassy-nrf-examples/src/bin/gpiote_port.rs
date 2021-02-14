#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::pin::Pin;
use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::gpio::{WaitForHigh, WaitForLow};
use embassy::util::Forever;
use embassy_nrf::gpiote::{Gpiote, GpiotePin};
use embassy_nrf::interrupt;

async fn button(n: usize, mut pin: GpiotePin<gpio::PullUp>) {
    loop {
        Pin::new(&mut pin).wait_for_low().await;
        info!("Button {:?} pressed!", n);
        Pin::new(&mut pin).wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[task]
async fn run() {
    let p = unwrap!(embassy_nrf::pac::Peripherals::take());
    let port0 = gpio::p0::Parts::new(p.P0);

    let (g, _) = Gpiote::new(p.GPIOTE, interrupt::take!(GPIOTE));

    let button1 = button(
        1,
        GpiotePin::new(g, port0.p0_11.into_pullup_input().degrade()),
    );
    let button2 = button(
        2,
        GpiotePin::new(g, port0.p0_12.into_pullup_input().degrade()),
    );
    let button3 = button(
        3,
        GpiotePin::new(g, port0.p0_24.into_pullup_input().degrade()),
    );
    let button4 = button(
        4,
        GpiotePin::new(g, port0.p0_25.into_pullup_input().degrade()),
    );
    futures::join!(button1, button2, button3, button4);
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run()));
    });
}
