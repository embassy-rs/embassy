//! Provides a function to try until something is true

use crate::rcc;

/// Performs a busy-wait delay for a specified number of microseconds that is async if possible
#[allow(dead_code)]
pub async fn wait_for_us(us: u64) {
    #[cfg(feature = "time")]
    embassy_time::Timer::after_micros(us).await;

    #[cfg(not(feature = "time"))]
    block_for_us(us);
}

/// Performs a busy-wait delay for a specified number of microseconds.
#[allow(dead_code)]
pub fn block_for_us(us: u64) {
    cortex_m::asm::delay(unsafe { rcc::get_freqs().sys.to_hertz().unwrap().0 as u64 * us / 1_000_000 } as u32);
}

#[cfg(feature = "time")]
/// Function to try until something is true
#[allow(dead_code)]
pub async fn try_until(mut func: impl AsyncFnMut() -> bool, micros: u64) -> Result<(), ()> {
    use embassy_time::{Duration, Ticker};

    let duration = Duration::from_micros(micros);
    let tick = Duration::from_millis(1);
    let mut ticker = Ticker::every(tick);
    let ticks = duration.as_ticks() / tick.as_ticks();

    for _ in 0..ticks {
        if func().await {
            return Ok(());
        }

        ticker.next().await;
    }

    Err(())
}

#[cfg(not(feature = "time"))]
/// Function to try until something is true
#[allow(dead_code)]
pub async fn try_until(mut func: impl AsyncFnMut() -> bool, micros: u64) -> Result<(), ()> {
    use embassy_futures::yield_now;

    let ticks = micros / 1_000;

    for _ in 0..ticks {
        if func().await {
            return Ok(());
        }

        block_for_us(1_000);
        yield_now().await;
    }

    Err(())
}
