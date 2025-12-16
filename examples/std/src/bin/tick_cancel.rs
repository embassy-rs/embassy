use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use embassy_executor::Executor;
use embassy_time::Timer;
use log::*;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn run() {
    loop {
        info!("tick");
        Timer::after_secs(1).await;
    }
}

static DONE: StaticCell<AtomicBool> = StaticCell::new();
static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    let done = DONE.init(AtomicBool::new(false));
    let done_cb = || done.load(Ordering::Relaxed);

    let server_thread = thread::spawn(move || {
        let executor = EXECUTOR.init(Executor::new());
        executor.run_until(
            |spawner| {
                spawner.spawn(run().unwrap());
            },
            done_cb,
        );
        info!("Executor finished");
    });

    thread::sleep(Duration::from_secs(5));

    info!("Cancelling executor");
    done.store(true, Ordering::Relaxed);

    server_thread.join().unwrap();
}
