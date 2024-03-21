#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static MUTEX: Mutex<ThreadModeRawMutex, u32> = Mutex::new(0);

#[embassy_executor::task]
async fn my_task() {
    loop {
        {
            let mut m = MUTEX.lock().await;
            info!("start long operation");
            *m += 1000;

            // Hold the mutex for a long time.
            Timer::after_secs(1).await;
            info!("end long operation: count = {}", *m);
        }

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    unwrap!(spawner.spawn(my_task()));

    loop {
        Timer::after_millis(300).await;
        let mut m = MUTEX.lock().await;
        *m += 1;
        info!("short operation: count = {}", *m);
    }
}
