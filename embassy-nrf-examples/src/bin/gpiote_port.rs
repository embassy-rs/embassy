#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_nrf::gpio::{AnyPin, Input, Pin as _, Pull};
use example_common::*;
use gpiote::GpioteInput;

use core::pin::Pin;
use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::traits::gpio::{WaitForHigh, WaitForLow};
use embassy::util::Forever;
use embassy_nrf::gpiote;
use embassy_nrf::interrupt;

async fn button(n: usize, mut pin: GpioteInput<AnyPin>) {
    loop {
        Pin::new(&mut pin).wait_for_low().await;
        info!("Button {:?} pressed!", n);
        Pin::new(&mut pin).wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[task]
async fn run() {
    let p = unsafe { embassy_nrf::peripherals::Peripherals::steal() };

    let g = gpiote::initialize(p.gpiote, interrupt::take!(GPIOTE));

    let button1 = button(
        1,
        GpioteInput::new(g, Input::new(p.p0_11.degrade(), Pull::Up)),
    );
    let button2 = button(
        2,
        GpioteInput::new(g, Input::new(p.p0_12.degrade(), Pull::Up)),
    );
    let button3 = button(
        3,
        GpioteInput::new(g, Input::new(p.p0_24.degrade(), Pull::Up)),
    );
    let button4 = button(
        4,
        GpioteInput::new(g, Input::new(p.p0_25.degrade(), Pull::Up)),
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
