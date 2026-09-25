//! Helpers for waiting: timeouts and poll-until-true utilities.

use core::future::Future;

#[cfg(feature = "time")]
use embassy_time::{Duration, Instant, Timer};

/// Error returned when a [`Timeout`] expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeoutError;

/// A deadline-based timeout.
///
/// Without the `time` feature there is no clock, so a `Timeout` never expires:
/// [`check`](Timeout::check) always succeeds, [`with`](Timeout::with) runs the
/// future to completion, and [`try_until_result`] loops until the predicate
/// returns `true`.
#[derive(Copy, Clone)]
pub struct Timeout {
    #[cfg(feature = "time")]
    deadline: Instant,
}

impl Timeout {
    /// Create a timeout that expires `duration` from now.
    #[cfg(feature = "time")]
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }

    /// Create a timeout that never expires.
    #[cfg(not(feature = "time"))]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "time")]
    pub fn from_micros(micros: u64) -> Self {
        Self::new(Duration::from_micros(micros))
    }

    #[cfg(not(feature = "time"))]
    pub fn from_micros(_micros: u64) -> Self {
        Self::new()
    }

    #[cfg(feature = "time")]
    pub(crate) fn deadline(&self) -> Instant {
        self.deadline
    }

    /// Returns `Err(TimeoutError)` if the timeout has expired.
    #[inline]
    pub fn check(self) -> Result<(), TimeoutError> {
        #[cfg(feature = "time")]
        if Instant::now() > self.deadline {
            return Err(TimeoutError);
        }

        #[cfg(not(feature = "time"))]
        let _ = self;

        Ok(())
    }

    /// Runs a future, returning `Err` if the timeout expires before the future completes.
    #[inline]
    pub fn with<R, E>(self, fut: impl Future<Output = Result<R, E>>) -> impl Future<Output = Result<R, E>>
    where
        E: From<TimeoutError>,
    {
        #[cfg(feature = "time")]
        {
            use futures_util::FutureExt;

            embassy_futures::select::select(embassy_time::Timer::at(self.deadline), fut).map(|r| match r {
                embassy_futures::select::Either::First(_) => Err(TimeoutError.into()),
                embassy_futures::select::Either::Second(r) => r,
            })
        }

        #[cfg(not(feature = "time"))]
        fut
    }
}

/// Performs a busy-wait delay for a specified number of microseconds that is async if possible
#[allow(dead_code)]
pub async fn wait_for_us(micros: u64) {
    #[cfg(feature = "time")]
    Timer::after_micros(micros).await;

    #[cfg(not(feature = "time"))]
    block_for_us(micros);
}

/// Performs a busy-wait delay for a specified number of microseconds.
#[allow(dead_code)]
pub fn block_for_us(micros: u64) {
    #[cfg(feature = "time")]
    embassy_time::block_for(Duration::from_micros(micros));

    #[cfg(not(feature = "time"))]
    cortex_m::asm::delay(
        unsafe { crate::rcc::get_freqs().sys.to_hertz().unwrap().0 as u64 * micros / 1_000_000 } as u32,
    );
}

#[cfg(feature = "time")]
/// Polls `func` until it returns `true` or an error, `Err` from `func` is returned immediately,
/// yielding to other tasks between polls. Returns `Err(TimeoutError)` if the timeout expires first.
pub async fn try_until_result<E: From<TimeoutError>>(
    mut func: impl AsyncFnMut() -> Result<bool, E>,
    timeout: Timeout,
) -> Result<(), E> {
    use core::future::poll_fn;
    use core::task::Poll;

    use embassy_futures::select::{Either, select};
    use embassy_time::{Duration, Ticker};
    use futures_util::FutureExt;

    match select(
        async {
            let mut ticker = Ticker::every(Duration::from_millis(1));

            loop {
                if func().await? {
                    return Ok(());
                }

                // Advance the ticker to the next pending tick
                poll_fn(|cx| {
                    while matches!(ticker.next().poll_unpin(cx), Poll::Ready(())) {}
                    Poll::Ready(())
                })
                .await;

                ticker.next().await;
            }
        },
        Timer::at(timeout.deadline()),
    )
    .await
    {
        Either::First(r) => r,
        Either::Second(()) => Err(TimeoutError.into()),
    }
}

#[cfg(not(feature = "time"))]
/// Polls `func` until it returns `true` or an error, `Err` from `func` is returned immediately,
/// yielding between polls. Without the `time` feature the timeout never expires.
pub async fn try_until_result<E: From<TimeoutError>>(
    mut func: impl AsyncFnMut() -> Result<bool, E>,
    _timeout: Timeout,
) -> Result<(), E> {
    use embassy_futures::yield_now;

    loop {
        if func().await? {
            return Ok(());
        }

        block_for_us(1_000);
        yield_now().await;
    }
}

/// Polls `func` until it returns `true`, yielding to other tasks between polls.
/// Returns `Err(TimeoutError)` if the timeout expires first.
#[allow(dead_code)]
pub async fn try_until_timeout(mut func: impl AsyncFnMut() -> bool, timeout: Timeout) -> Result<(), TimeoutError> {
    try_until_result(async || Ok(func().await), timeout).await
}

/// Function to try until something is true
#[allow(dead_code)]
pub async fn try_until(func: impl AsyncFnMut() -> bool, micros: u64) -> Result<(), ()> {
    try_until_timeout(func, Timeout::from_micros(micros))
        .await
        .map_err(|_| ())
}
