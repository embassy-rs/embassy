//! This example tests stack overflow handling on core1 of the RP235x chip.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const CORE1_STACK_LENGTH: usize = 4096;

static mut CORE1_STACK: Stack<CORE1_STACK_LENGTH> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::Low);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.spawn(unwrap!(core1_task())));
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(unwrap!(core0_task(led))));
}

#[embassy_executor::task]
async fn core0_task(mut led: Output<'static>) {
    info!("Hello from core 0");
    loop {
        info!("core 0 still alive");
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

fn blow_my_stack() {
    // Allocating an array a little larger than our stack should ensure a stack overflow when it is used.
    let t = [0u8; CORE1_STACK_LENGTH + 64];

    info!("Array initialised without error");
    // We need to use black_box to otherwise the compiler is too smart and will optimise all of this away.
    // We shouldn't get to this code - the initialisation above will touch the stack guard.
    for ref i in t {
        let _data = core::hint::black_box(*i) + 1;
    }
}

#[embassy_executor::task]
async fn core1_task() {
    info!("Hello from core 1");

    blow_my_stack();

    loop {
        info!("core 1 still alive");
        Timer::after_millis(1000).await;
    }
}
