use core::future::Future;

use super::{Duration, Instant};
use crate::Timer;

/// Blocks for at least `duration`.
///
/// This function may require interrupts to be enabled to work correctly.
///
/// ## Panics
///
/// Panics if the computed instant overflows.
/// Avoid panics with [`try_block_for()`].
pub fn block_for(duration: Duration) {
    let expires_at = Instant::now() + duration;
    while Instant::now() < expires_at {}
}

/// Tries to block for at least `duration`.
///
/// This function may require interrupts to be enabled to work correctly.
///
/// This is a panic-free [`block_for()`].
pub fn try_block_for(duration: Duration) -> Option<()> {
    let expires_at = Instant::now().checked_add(duration)?;
    while Instant::now() < expires_at {}
    Some(())
}

/// Type implementing async delays and blocking `embedded-hal` delays.
///
/// The delays are implemented in a "best-effort" way, meaning that the cpu will block for at least
/// the amount provided, but accuracy can be affected by many factors, including interrupt usage.
/// Make sure to use a suitable tick rate for your use case. The tick rate is defined by the currently
/// active driver.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Delay;

impl embedded_hal_1::delay::DelayNs for Delay {
    /// Pauses execution for at minimum `ns` nanoseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ns(&mut self, ns: u32) {
        block_for(Duration::from_nanos(ns as u64))
    }

    /// Pauses execution for at minimum `us` microseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_us(&mut self, us: u32) {
        block_for(Duration::from_micros(us as u64))
    }

    /// Pauses execution for at minimum `ms` milliseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ms(&mut self, ms: u32) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal_async::delay::DelayNs for Delay {
    /// Pauses execution for at minimum `ns` nanoseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ns(&mut self, ns: u32) -> impl Future<Output = ()> {
        Timer::after_nanos(ns as _)
    }

    /// Pauses execution for at minimum `us` microseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_us(&mut self, us: u32) -> impl Future<Output = ()> {
        Timer::after_micros(us as _)
    }

    /// Pauses execution for at minimum `ms` milliseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ms(&mut self, ms: u32) -> impl Future<Output = ()> {
        Timer::after_millis(ms as _)
    }
}

impl embedded_hal_02::blocking::delay::DelayMs<u8> for Delay {
    /// Pauses execution for at minimum `ms` milliseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ms(&mut self, ms: u8) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal_02::blocking::delay::DelayMs<u16> for Delay {
    /// Pauses execution for at minimum `ms` milliseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ms(&mut self, ms: u16) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal_02::blocking::delay::DelayMs<u32> for Delay {
    /// Pauses execution for at minimum `ms` milliseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_ms(&mut self, ms: u32) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal_02::blocking::delay::DelayUs<u8> for Delay {
    /// Pauses execution for at minimum `us` microseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_us(&mut self, us: u8) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal_02::blocking::delay::DelayUs<u16> for Delay {
    /// Pauses execution for at minimum `us` microseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_us(&mut self, us: u16) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal_02::blocking::delay::DelayUs<u32> for Delay {
    /// Pauses execution for at minimum `us` microseconds.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn delay_us(&mut self, us: u32) {
        block_for(Duration::from_micros(us as u64))
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn block_for_panics() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        block_for(Duration::MAX); // PANIC
    }

    #[test]
    fn try_block_for_none() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        assert!(try_block_for(Duration::MAX).is_none());
    }
}
