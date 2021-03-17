#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

use embassy::executor::task;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_std::Executor;
use log::*;

#[task]
async fn run() {
    loop {
        info!("tick");
        Timer::after(Duration::from_secs(1)).await;
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();
    });
}
