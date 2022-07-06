#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;

use defmt::{info, unwrap};
use embassy::executor::raw::{ScopedTaskStorage, TaskHeader};
use embassy::executor::{Executor, Spawner};
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;
use futures::future::join;
use {defmt_rtt as _, panic_probe as _};

async fn add_ref(a: &mut u8, b: u8) {
    Timer::after(Duration::from_secs(2)).await;
    info!("add_ref(): done!");
    *a += b;
}

async fn add_rtn(a: u8, b: u8) -> u8 {
    Timer::after(Duration::from_secs(1)).await;
    info!("add_rtn(): done!");
    a + b
}

static TASK_STORAGE1: ScopedTaskStorage<()> = ScopedTaskStorage::new();
static TASK_STORAGE2: ScopedTaskStorage<u8> = ScopedTaskStorage::new();

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) -> ! {
    let mut a = 2;

    let mut fut1 = add_ref(&mut a, 2);
    let mut fut2 = add_rtn(2, 2);
    let (run1_guard, run1_token) = unwrap!(TASK_STORAGE1.spawn_scoped(&mut fut1));
    let (run2_guard, run2_token) = unwrap!(TASK_STORAGE2.spawn_scoped(&mut fut2));

    unwrap!(spawner.spawn(run1_token));
    unwrap!(spawner.spawn(run2_token));

    run1_guard.await;
    drop(fut1); // Release mutable reference to a. Could also use scope brackets instead of drop.

    info!("add_ref: {}", a);
    info!("add_rtn: {}", run2_guard.await);

    // Spawn add_ref again
    let mut fut1 = add_ref(&mut a, 2);
    let (run1_guard, run1_token) = unwrap!(TASK_STORAGE1.spawn_scoped(&mut fut1));
    unwrap!(spawner.spawn(run1_token));

    run1_guard.await;
    drop(fut1);

    info!("add_ref: {}", a);

    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
