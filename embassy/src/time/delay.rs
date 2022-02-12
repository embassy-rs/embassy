use super::{Duration, Instant};

/// Blocks for at least `duration`.
pub fn block_for(duration: Duration) {
    let expires_at = Instant::now() + duration;
    while Instant::now() < expires_at {}
}

/// Type implementing async delays and blocking `embedded-hal` delays.
///
/// The delays are implemented in a "best-effort" way, meaning that the cpu will block for at least
/// the amount provided, but accuracy can be affected by many factors, including interrupt usage.
/// Make sure to use a suitable tick rate for your use case. The tick rate is defined by the currently
/// active driver.
pub struct Delay;

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::delay::blocking::DelayUs for Delay {
        type Error = core::convert::Infallible;

        fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
            Ok(block_for(Duration::from_micros(us as u64)))
        }

        fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
            Ok(block_for(Duration::from_millis(ms as u64)))
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eh1a {
    use super::*;
    use crate::time::Timer;
    use core::future::Future;
    use futures::FutureExt;

    impl embedded_hal_async::delay::DelayUs for Delay {
        type Error = core::convert::Infallible;

        type DelayUsFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn delay_us(&mut self, micros: u32) -> Self::DelayUsFuture<'_> {
            Timer::after(Duration::from_micros(micros as _)).map(Ok)
        }

        type DelayMsFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn delay_ms(&mut self, millis: u32) -> Self::DelayMsFuture<'_> {
            Timer::after(Duration::from_millis(millis as _)).map(Ok)
        }
    }
}

mod eh02 {
    use super::*;
    use embedded_hal_02::blocking::delay::{DelayMs, DelayUs};

    impl DelayMs<u8> for Delay {
        fn delay_ms(&mut self, ms: u8) {
            block_for(Duration::from_millis(ms as u64))
        }
    }

    impl DelayMs<u16> for Delay {
        fn delay_ms(&mut self, ms: u16) {
            block_for(Duration::from_millis(ms as u64))
        }
    }

    impl DelayMs<u32> for Delay {
        fn delay_ms(&mut self, ms: u32) {
            block_for(Duration::from_millis(ms as u64))
        }
    }

    impl DelayUs<u8> for Delay {
        fn delay_us(&mut self, us: u8) {
            block_for(Duration::from_micros(us as u64))
        }
    }

    impl DelayUs<u16> for Delay {
        fn delay_us(&mut self, us: u16) {
            block_for(Duration::from_micros(us as u64))
        }
    }

    impl DelayUs<u32> for Delay {
        fn delay_us(&mut self, us: u32) {
            block_for(Duration::from_micros(us as u64))
        }
    }
}
