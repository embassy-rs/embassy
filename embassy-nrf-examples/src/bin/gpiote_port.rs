#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::pin::Pin;
use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::traits::gpio::{WaitForHigh, WaitForLow};
use embassy::util::Forever;
use embassy_nrf::gpio::{AnyPin, Input, Pin as _, Pull};
use embassy_nrf::gpiote::{self, PortInput};
use embassy_nrf::interrupt;
use embassy_nrf::Peripherals;
use example_common::*;

async fn button(n: usize, mut pin: PortInput<AnyPin>) {
    loop {
        Pin::new(&mut pin).wait_for_low().await;
        info!("Button {:?} pressed!", n);
        Pin::new(&mut pin).wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[task]
async fn run() {
    let p = Peripherals::take().unwrap();

    let g = gpiote::initialize(p.gpiote, interrupt::take!(GPIOTE));

    let button1 = button(
        1,
        PortInput::new(g, Input::new(p.p0_11.degrade(), Pull::Up)),
    );
    let button2 = button(
        2,
        PortInput::new(g, Input::new(p.p0_12.degrade(), Pull::Up)),
    );
    let button3 = button(
        3,
        PortInput::new(g, Input::new(p.p0_24.degrade(), Pull::Up)),
    );
    let button4 = button(
        4,
        PortInput::new(g, Input::new(p.p0_25.degrade(), Pull::Up)),
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
