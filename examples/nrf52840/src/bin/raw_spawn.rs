#![no_std]
#![no_main]

use core::mem;

use cortex_m_rt::entry;
use defmt::{info, unwrap};
use embassy_executor::raw::TaskStorage;
use embassy_executor::Executor;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after_ticks(64000).await;
    }
}

async fn run2() {
    loop {
        info!("tick");
        Timer::after_ticks(13000).await;
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let _p = embassy_nrf::init(Default::default());
    let executor = EXECUTOR.init(Executor::new());

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
