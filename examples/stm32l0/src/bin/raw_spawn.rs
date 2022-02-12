#![no_std]
#![no_main]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;

use embassy::executor::raw::TaskStorage;
use embassy::executor::Executor;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;

async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after(Duration::from_ticks(64000)).await;
    }
}

async fn run2() {
    loop {
        info!("tick");
        Timer::after(Duration::from_ticks(13000)).await;
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let _p = embassy_stm32::init(Default::default());
    let executor = EXECUTOR.put(Executor::new());

    let run1_task = TaskStorage::new();
    let run2_task = TaskStorage::new();

    // Safety: these variables do live forever if main never returns.
    let run1_task = unsafe { make_static(&run1_task) };
    let run2_task = unsafe { make_static(&run2_task) };

    executor.run(|spawner| {
        unwrap!(spawner.spawn(run1_task.spawn(|| run1())));
        unwrap!(spawner.spawn(run2_task.spawn(|| run2())));
    });
}

unsafe fn make_static<T>(t: &T) -> &'static T {
    mem::transmute(t)
}
