use core::future::Future;

use super::{Duration, Instant, Timer};

/// Type implementing async delays and blocking `embedded-hal` delays.
///
/// The delays are implemented in a "best-effort" way, meaning that the cpu will block for at least
/// the amount provided, but accuracy can be affected by many factors, including interrupt usage.
/// Make sure to use a suitable tick rate for your use case. The tick rate is defined by the currently
/// active driver.
pub struct Delay;

impl crate::traits::delay::Delay for Delay {
    type DelayFuture<'a> = impl Future<Output = ()> + 'a;

    fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture<'_> {
        Timer::after(Duration::from_millis(millis))
    }
    fn delay_us(&mut self, micros: u64) -> Self::DelayFuture<'_> {
        Timer::after(Duration::from_micros(micros))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        block_for(Duration::from_micros(us as u64))
    }
}

/// Blocks for at least `duration`.
pub fn block_for(duration: Duration) {
    let expires_at = Instant::now() + duration;
    while Instant::now() < expires_at {}
}
