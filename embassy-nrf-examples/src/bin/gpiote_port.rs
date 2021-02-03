#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embassy_nrf::gpiote::{Gpiote, PortInputPolarity};
use embassy_nrf::interrupt;

async fn button(g: &Gpiote, n: usize, pin: gpio::Pin<gpio::Input<gpio::PullUp>>) {
    loop {
        g.wait_port_input(&pin, PortInputPolarity::Low).await;
        info!("Button {:?} pressed!", n);
        g.wait_port_input(&pin, PortInputPolarity::High).await;
        info!("Button {:?} released!", n);
    }
}

#[task]
async fn run() {
    let p = unwrap!(embassy_nrf::pac::Peripherals::take());
    let port0 = gpio::p0::Parts::new(p.P0);

    let g = Gpiote::new(p.GPIOTE, interrupt::take!(GPIOTE));
    info!(
        "sizeof Signal<()> = {:usize}",
        mem::size_of::<embassy::util::Signal<()>>()
    );
    info!("sizeof gpiote = {:usize}", mem::size_of::<Gpiote>());

    info!("Starting!");

    let button1 = button(&g, 1, port0.p0_11.into_pullup_input().degrade());
    let button2 = button(&g, 2, port0.p0_12.into_pullup_input().degrade());
    let button3 = button(&g, 3, port0.p0_24.into_pullup_input().degrade());
    let button4 = button(&g, 4, port0.p0_25.into_pullup_input().degrade());
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
