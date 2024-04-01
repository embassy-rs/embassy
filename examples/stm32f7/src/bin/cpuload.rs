#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use static_cell::StaticCell;

/// Global executor.
static EXECUTOR: StaticCell<Executor> = StaticCell::new();


#[cortex_m_rt::entry]
fn main() -> ! {
    // Create the executor.
    let mut executor = embassy_executor::Executor::new();

    // Set the measure function.
    executor.measure_cpu_load( measure );

    // Initialize the singleton.
    let executor = EXECUTOR.init(executor);

    executor.run(|spawner| {
        spawner.must_spawn( asyncmain(spawner.clone()) );
    })
}

#[embassy_executor::task]
async fn asyncmain(_spawner: Spawner) -> ! {
    let config = Config::default();
    let _p = embassy_stm32::init(config);

    // Modify these values to see the change in CPU load.
    const ITERS: usize = 10_000_000;
    const DELAY: u64 = 10;

    info!("Hello World!");

    loop {
        // Do some work.
        for _ in 0..ITERS {
            cortex_m::asm::nop();
        }

        // Go to sleep.
        Timer::after_millis( DELAY ).await;
    }
}


/// Sync function that receives the CPU load variables.
/// `ts`: Sleep time. The amount of timer cycles between the last sleep and the most recent wakeup.
/// `tc`: Cycle time. The amount of timer cycles between the last sleep and now.
/// The formulas to calculate the CPU load are:
///   - Fraction = 1 - (ts / tc).
///   - Percent  = 100 - ((100 * ts) / tc)
/// The current implementation only measures the load for the current poll cycle. Some cycles can be very short
/// in the order of milliseconds or even microseconds. To reduce the effect of these short cycles on the CPU load
/// values, it is recommended to store a series of cycles and then do a rolling average with the last handful
/// of measurements weighted with the duration of the cycle.
fn measure(ts: u64, tc: u64) {
    // Calculate the load.
    let load = 100 - ((100 * ts) / tc);

    // Report the CPU load.
    defmt::info!("CPU load: {}% [{} - {}]", load, ts, tc);
}
