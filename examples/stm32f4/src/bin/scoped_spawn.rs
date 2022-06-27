#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;

use defmt::{info, unwrap};
use embassy::executor::raw::{ScopedTaskStorage, TaskHeader};
use embassy::executor::{Executor, Spawner};
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

async fn run1() {
    Timer::after(Duration::from_secs(1)).await;
    info!("run1(): done!");
}

async fn run2(a: &mut u8) {
    Timer::after(Duration::from_secs(2)).await;
    *a += 1;
    info!("run2(): done!");
}

// static RUN1_HEAD: TaskHeader = TaskHeader::new();
// static RUN2_HEAD: TaskHeader = TaskHeader::new();

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) -> ! {
    // let run1_head = TaskHeader::new();
    // let run2_head = TaskHeader::new();

    // let run1_head = unsafe { make_static(&run1_head) };
    // let run2_head = unsafe { make_static(&run2_head) };

    let run1_task = ScopedTaskStorage::new();
    let run2_task = ScopedTaskStorage::new();

    let mut x = 0;
    // let x_ref = &mut x;

    // Safety: these variables do live forever if main never returns.
    // let run1_task = unsafe { make_static(&run1_task) };
    // let run2_task = unsafe { make_static(&run2_task) };
    {
        let (run1_guard, run1_token) = unwrap!(run1_task.spawn_scoped(run1()));
        let (run2_guard, run2_token) = unwrap!(run2_task.spawn_scoped(run2(&mut x)));

        unwrap!(spawner.spawn(run1_token));
        unwrap!(spawner.spawn(run2_token));

        run1_guard.await;
        run2_guard.await;
    }

    // {
    //     let (run2_guard, run2_token) = unwrap!(run2_task.spawn_scoped(run2(&mut x)));
    //     unwrap!(spawner.spawn(run2_token));
    //     run2_guard.await;
    // }

    info!("x: {}", x);

    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}

unsafe fn make_static<T>(t: &T) -> &'static T {
    mem::transmute(t)
}
