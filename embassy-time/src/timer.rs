use core::future::{Future, poll_fn};
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::Stream;
use futures_core::stream::FusedStream;

use crate::{Duration, Instant};

/// Error returned by [`with_timeout`] and [`with_deadline`] on timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeoutError;

/// Runs a given future with a timeout.
///
/// If the future completes before the timeout, its output is returned. Otherwise, on timeout,
/// work on the future is stopped (`poll` is no longer called), the future is dropped and `Err(TimeoutError)` is returned.
pub fn with_timeout<F: Future>(timeout: impl Into<Duration>, fut: F) -> TimeoutFuture<F> {
    TimeoutFuture {
        timer: Timer::after(timeout.into()),
        fut,
    }
}

/// Runs a given future with a deadline time.
///
/// If the future completes before the deadline, its output is returned. Otherwise, on timeout,
/// work on the future is stopped (`poll` is no longer called), the future is dropped and `Err(TimeoutError)` is returned.
pub fn with_deadline<F: Future>(at: Instant, fut: F) -> TimeoutFuture<F> {
    TimeoutFuture {
        timer: Timer::at(at),
        fut,
    }
}

/// Provides functions to run a given future with a timeout or a deadline.
pub trait WithTimeout: Sized {
    /// Output type of the future.
    type Output;

    /// Runs a given future with a timeout.
    ///
    /// If the future completes before the timeout, its output is returned. Otherwise, on timeout,
    /// work on the future is stopped (`poll` is no longer called), the future is dropped and `Err(TimeoutError)` is returned.
    fn with_timeout(self, timeout: impl Into<Duration>) -> TimeoutFuture<Self>;

    /// Runs a given future with a deadline time.
    ///
    /// If the future completes before the deadline, its output is returned. Otherwise, on timeout,
    /// work on the future is stopped (`poll` is no longer called), the future is dropped and `Err(TimeoutError)` is returned.
    fn with_deadline(self, at: Instant) -> TimeoutFuture<Self>;
}

impl<F: Future> WithTimeout for F {
    type Output = F::Output;

    fn with_timeout(self, timeout: impl Into<Duration>) -> TimeoutFuture<Self> {
        with_timeout(timeout.into(), self)
    }

    fn with_deadline(self, at: Instant) -> TimeoutFuture<Self> {
        with_deadline(at, self)
    }
}

/// Future for the [`with_timeout`] and [`with_deadline`] functions.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeoutFuture<F> {
    timer: Timer,
    fut: F,
}

impl<F: Unpin> Unpin for TimeoutFuture<F> {}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let fut = unsafe { Pin::new_unchecked(&mut this.fut) };
        let timer = unsafe { Pin::new_unchecked(&mut this.timer) };
        if let Poll::Ready(x) = fut.poll(cx) {
            return Poll::Ready(Ok(x));
        }
        if let Poll::Ready(_) = timer.poll(cx) {
            return Poll::Ready(Err(TimeoutError));
        }
        Poll::Pending
    }
}

/// A future that completes at a specified [Instant](struct.Instant.html).
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timer {
    expires_at: Instant,
    yielded_once: bool,
}

impl Timer {
    /// Expire at specified [Instant](struct.Instant.html)
    /// Will expire immediately if the Instant is in the past.
    pub fn at(expires_at: Instant) -> Self {
        Self {
            expires_at,
            yielded_once: false,
        }
    }

    /// Expire after specified [Duration](struct.Duration.html).
    /// This can be used as a `sleep` abstraction.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after()`].
    ///
    /// ## Example
    ///
    /// ``` no_run
    /// use embassy_time::{Duration, Timer};
    ///
    /// #[embassy_executor::task]
    /// async fn demo_sleep_seconds() {
    ///     // suspend this task for one second.
    ///     Timer::after(Duration::from_secs(1)).await;
    /// }
    /// ```
    pub fn after(duration: impl Into<Duration>) -> Self {
        let expires_at = Instant::now() + duration.into();
        Self::at(expires_at)
    }

    /// Try to expire after specified [Duration](struct.Duration.html).
    /// This can be used as a `sleep` abstraction.
    ///
    /// This is a panic-free [`Timer::after()`].
    pub fn try_after(duration: impl Into<Duration>) -> Option<Self> {
        let expires_at = Instant::now().checked_add(duration.into())?;
        Some(Self::at(expires_at))
    }

    /// Expire after the specified number of ticks.
    ///
    /// This method is a convenience wrapper for calling `Timer::after(Duration::from_ticks())`.
    /// For more details, refer to [`Timer::after()`] and [`Duration::from_ticks()`].
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after_ticks()`].
    #[inline]
    pub fn after_ticks(ticks: u64) -> Self {
        Self::after(Duration::from_ticks(ticks))
    }

    /// Try to expire after the specified number of ticks.
    ///
    /// This method is a convenience wrapper for calling `Timer::try_after(Duration::from_ticks())`.
    /// For more details, refer to [`Timer::try_after()`] and [`Duration::from_ticks()`].
    ///
    /// This is a panic-free [`Timer::after_ticks()`].
    #[inline]
    pub fn try_after_ticks(ticks: u64) -> Option<Self> {
        Self::try_after(Duration::from_ticks(ticks))
    }

    /// Expire after the specified number of nanoseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::after(Duration::from_nanos())`.
    /// For more details, refer to [`Timer::after()`] and [`Duration::from_nanos()`].
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after_nanos()`].
    #[inline]
    pub fn after_nanos(nanos: u64) -> Self {
        Self::after(Duration::from_nanos(nanos))
    }

    /// Try to expire after the specified number of nanoseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::try_after(Duration::from_ticks())`.
    /// For more details, refer to [`Timer::try_after()`] and [`Duration::from_nanos()`].
    ///
    /// This is a panic-free [`Timer::after_nanos()`].
    #[inline]
    pub fn try_after_nanos(nanos: u64) -> Option<Self> {
        Self::try_after(Duration::from_nanos(nanos))
    }

    /// Expire after the specified number of microseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::after(Duration::from_micros())`.
    /// For more details, refer to [`Timer::after()`] and [`Duration::from_micros()`].
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after_nanos()`].
    #[inline]
    pub fn after_micros(micros: u64) -> Self {
        Self::after(Duration::from_micros(micros))
    }

    /// Try to expire after the specified number of microseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::try_after(Duration::from_micros())`.
    /// For more details, refer to [`Timer::try_after()`] and [`Duration::from_micros()`].
    ///
    /// This is a panic-free [`Timer::after_micros()`].
    #[inline]
    pub fn try_after_micros(micros: u64) -> Option<Self> {
        Self::try_after(Duration::from_micros(micros))
    }

    /// Expire after the specified number of milliseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::after(Duration::from_millis())`.
    /// For more details, refer to [`Timer::after`] and [`Duration::from_millis()`].
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after_millis()`].
    #[inline]
    pub fn after_millis(millis: u64) -> Self {
        Self::after(Duration::from_millis(millis))
    }

    /// Try to expire after the specified number of milliseconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::try_after(Duration::from_millis())`.
    /// For more details, refer to [`Timer::try_after`] and [`Duration::from_millis()`].
    ///
    /// This is a panic-free [`Timer::after_millis()`].
    #[inline]
    pub fn try_after_millis(millis: u64) -> Option<Self> {
        Self::try_after(Duration::from_millis(millis))
    }

    /// Expire after the specified number of seconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::after(Duration::from_secs())`.
    /// For more details, refer to [`Timer::after`] and [`Duration::from_secs()`].
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Timer::try_after_secs()`].
    #[inline]
    pub fn after_secs(secs: u64) -> Self {
        Self::after(Duration::from_secs(secs))
    }

    /// Try to expire after the specified number of seconds.
    ///
    /// This method is a convenience wrapper for calling `Timer::try_after(Duration::from_secs())`.
    /// For more details, refer to [`Timer::try_after`] and [`Duration::from_secs()`].
    ///
    /// This is a panic-free [`Timer::after_secs()`].
    #[inline]
    pub fn try_after_secs(secs: u64) -> Option<Self> {
        Self::try_after(Duration::from_secs(secs))
    }
}

impl Unpin for Timer {}

impl Future for Timer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded_once && self.expires_at <= Instant::now() {
            Poll::Ready(())
        } else {
            embassy_time_driver::schedule_wake(self.expires_at.as_ticks(), cx.waker());
            self.yielded_once = true;
            Poll::Pending
        }
    }
}

/// Asynchronous stream that yields every Duration, indefinitely.
///
/// This stream will tick at uniform intervals, even if blocking work is performed between ticks.
///
/// For instance, consider the following code fragment.
/// ``` no_run
/// use embassy_time::{Duration, Timer};
/// # fn foo() {}
///
/// #[embassy_executor::task]
/// async fn ticker_example_0() {
///     loop {
///         foo();
///         Timer::after(Duration::from_secs(1)).await;
///     }
/// }
/// ```
///
/// This fragment will not call `foo` every second.
/// Instead, it will call it every second + the time it took to previously call `foo`.
///
/// Example using ticker, which will consistently call `foo` once a second.
///
/// ``` no_run
/// use embassy_time::{Duration, Ticker};
/// # fn foo(){}
///
/// #[embassy_executor::task]
/// async fn ticker_example_1() {
///     let mut ticker = Ticker::every(Duration::from_secs(1));
///     loop {
///         foo();
///         ticker.next().await;
///     }
/// }
/// ```
///
/// ## Cancel safety
/// It is safe to cancel waiting for the next tick,
/// meaning no tick is lost if the Future is dropped.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ticker {
    expires_at: Instant,
    duration: Duration,
}

impl Ticker {
    /// Creates a new ticker that ticks at the specified duration interval.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Ticker::try_every()`].
    pub fn every(duration: impl Into<Duration>) -> Self {
        let duration = duration.into();
        let expires_at = Instant::now() + duration;
        Self { expires_at, duration }
    }

    /// Tries to create a new ticker that ticks at the specified duration interval.
    ///
    /// This is a panic-free [`Ticker::every()`].
    pub fn try_every(duration: impl Into<Duration>) -> Option<Self> {
        let duration = duration.into();
        let expires_at = Instant::now().checked_add(duration)?;
        Some(Self { expires_at, duration })
    }

    /// Resets the ticker back to its original state.
    /// This causes the ticker to go back to zero, even if the current tick isn't over yet.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Ticker::checked_reset()`].
    pub fn reset(&mut self) {
        self.expires_at = Instant::now() + self.duration;
    }

    /// Checked resets the ticker back to its original state.
    /// This causes the ticker to go back to zero, even if the current tick isn't over yet.
    ///
    /// This is a panic-free [`Ticker::reset()`].
    pub fn checked_reset(&mut self) -> Option<()> {
        self.expires_at = Instant::now().checked_add(self.duration)?;
        Some(())
    }

    /// Reset the ticker at the deadline.
    /// If the deadline is in the past, the ticker will fire before the next scheduled tick.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Ticker::checked_reset_at()`].
    pub fn reset_at(&mut self, deadline: Instant) {
        self.expires_at = deadline + self.duration;
    }

    /// Checked reset the ticker at the deadline.
    /// If the deadline is in the past, the ticker will fire before the next scheduled tick.
    ///
    /// This is a panic-free [`Ticker::reset_at()`].
    pub fn checked_reset_at(&mut self, deadline: Instant) -> Option<()> {
        self.expires_at = deadline.checked_add(self.duration)?;
        Some(())
    }

    /// Resets the ticker, after the specified duration has passed.
    /// If the specified duration is zero, the next tick will be after the duration of the ticker.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    /// Avoid panics with [`Ticker::checked_reset_after()`].
    pub fn reset_after(&mut self, after: impl Into<Duration>) {
        let after = after.into();
        self.expires_at = Instant::now() + after + self.duration;
    }

    /// Checked resets the ticker, after the specified duration has passed.
    /// If the specified duration is zero, the next tick will be after the duration of the ticker.
    ///
    /// This is a panic-free [`Ticker::reset_after()`].
    pub fn checked_reset_after(&mut self, after: impl Into<Duration>) -> Option<()> {
        let after = after.into();
        self.expires_at = Instant::now().checked_add(after)?.checked_add(self.duration)?;
        Some(())
    }

    /// Waits for the next tick.
    ///
    /// ## Panics
    ///
    /// Panics if the next computed instant overflows.
    ///
    /// ## Cancel safety
    /// The produced Future is cancel safe, meaning no tick is lost if the Future is dropped.
    pub fn next(&mut self) -> impl Future<Output = ()> + Send + Sync + '_ {
        poll_fn(|cx| {
            if self.expires_at <= Instant::now() {
                let dur = self.duration;
                self.expires_at += dur;
                Poll::Ready(())
            } else {
                embassy_time_driver::schedule_wake(self.expires_at.as_ticks(), cx.waker());
                Poll::Pending
            }
        })
    }
}

impl Unpin for Ticker {}

impl Stream for Ticker {
    type Item = ();
    /// Attempt to pull the next tick from this [`Stream`].
    ///
    /// * Returns `Poll::Pending` if the instant is in the future.
    /// * Returns `Poll::Ready(Some(()))` if the instant has been reached, and computes the next instant.
    ///
    /// ## Panics
    ///
    /// Panics if the computed instant overflows.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.expires_at <= Instant::now() {
            let dur = self.duration;
            self.expires_at += dur;
            Poll::Ready(Some(()))
        } else {
            embassy_time_driver::schedule_wake(self.expires_at.as_ticks(), cx.waker());
            Poll::Pending
        }
    }
}

impl FusedStream for Ticker {
    fn is_terminated(&self) -> bool {
        // `Ticker` keeps yielding values until dropped, it never terminates.
        false
    }
}

impl core::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TimeoutError")
    }
}
impl core::error::Error for TimeoutError {}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn timer_after_panics() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        let _ = Timer::after(Duration::MAX); // PANIC
    }

    #[test]
    fn timer_try_after_none() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        assert!(Timer::try_after(Duration::MAX).is_none());
    }

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn ticker_every_panics() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        let _ = Ticker::every(Duration::MAX); // PANIC
    }

    #[test]
    fn ticker_try_every_none() {
        while Instant::now() == Instant::MIN {} // with non-0 tick
        assert!(Ticker::try_every(Duration::MAX).is_none());
    }

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn ticker_reset_panics() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        while Instant::now() == Instant::MIN {} // with non-0 tick
        ticker.reset(); // PANIC
    }

    #[test]
    fn ticker_checked_reset_none() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        while Instant::now() == Instant::MIN {} // with non-0 tick
        assert!(ticker.checked_reset().is_none());
    }

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn ticker_reset_at_panics() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        ticker.reset_at(Instant::MAX); // PANIC
    }

    #[test]
    fn ticker_checked_reset_at_none() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        assert!(ticker.checked_reset_at(Instant::MAX).is_none());
    }

    #[test]
    #[should_panic(expected = "overflow")]
    #[cfg(panic = "unwind")]
    fn ticker_reset_after_panics() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        while Instant::now() == Instant::MIN {} // with non-0 tick
        ticker.reset_after(Duration::MAX); // PANIC
    }

    #[test]
    fn ticker_checked_reset_after_none() {
        let mut ticker = Ticker {
            expires_at: Instant::MAX,
            duration: Duration::MAX,
        };
        while Instant::now() == Instant::MIN {} // with non-0 tick
        assert!(ticker.checked_reset_after(Duration::MAX).is_none());
    }
}
