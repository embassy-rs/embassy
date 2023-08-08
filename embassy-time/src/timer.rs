use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use futures_util::future::{select, Either};
use futures_util::stream::FusedStream;
use futures_util::{pin_mut, Stream};

use crate::{Duration, Instant};

/// Error returned by [`with_timeout`] on timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeoutError;

/// Runs a given future with a timeout.
///
/// If the future completes before the timeout, its output is returned. Otherwise, on timeout,
/// work on the future is stopped (`poll` is no longer called), the future is dropped and `Err(TimeoutError)` is returned.
pub async fn with_timeout<F: Future>(timeout: Duration, fut: F) -> Result<F::Output, TimeoutError> {
    let timeout_fut = Timer::after(timeout);
    pin_mut!(fut);
    match select(fut, timeout_fut).await {
        Either::Left((r, _)) => Ok(r),
        Either::Right(_) => Err(TimeoutError),
    }
}

/// A future that completes at a specified [Instant](struct.Instant.html).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Timer {
    expires_at: Instant,
    yielded_once: bool,
}

impl Timer {
    /// Expire at specified [Instant](struct.Instant.html)
    pub fn at(expires_at: Instant) -> Self {
        Self {
            expires_at,
            yielded_once: false,
        }
    }

    /// Expire after specified [Duration](struct.Duration.html).
    /// This can be used as a `sleep` abstraction.
    ///
    /// Example:
    /// ``` no_run
    /// # #![feature(type_alias_impl_trait)]
    /// #
    /// # fn foo() {}
    /// use embassy_time::{Duration, Timer};
    ///
    /// #[embassy_executor::task]
    /// async fn demo_sleep_seconds() {
    ///     // suspend this task for one second.
    ///     Timer::after(Duration::from_secs(1)).await;
    /// }
    /// ```
    pub fn after(duration: Duration) -> Self {
        Self {
            expires_at: Instant::now() + duration,
            yielded_once: false,
        }
    }
}

impl Unpin for Timer {}

impl Future for Timer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded_once && self.expires_at <= Instant::now() {
            Poll::Ready(())
        } else {
            schedule_wake(self.expires_at, cx.waker());
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
/// # #![feature(type_alias_impl_trait)]
/// #
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
/// # #![feature(type_alias_impl_trait)]
/// #
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
pub struct Ticker {
    expires_at: Instant,
    duration: Duration,
}

impl Ticker {
    /// Creates a new ticker that ticks at the specified duration interval.
    pub fn every(duration: Duration) -> Self {
        let expires_at = Instant::now() + duration;
        Self { expires_at, duration }
    }

    /// Waits for the next tick
    pub fn next(&mut self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            if self.expires_at <= Instant::now() {
                let dur = self.duration;
                self.expires_at += dur;
                Poll::Ready(())
            } else {
                schedule_wake(self.expires_at, cx.waker());
                Poll::Pending
            }
        })
    }
}

impl Unpin for Ticker {}

impl Stream for Ticker {
    type Item = ();
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.expires_at <= Instant::now() {
            let dur = self.duration;
            self.expires_at += dur;
            Poll::Ready(Some(()))
        } else {
            schedule_wake(self.expires_at, cx.waker());
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

extern "Rust" {
    fn _embassy_time_schedule_wake(at: Instant, waker: &Waker);
}

fn schedule_wake(at: Instant, waker: &Waker) {
    unsafe { _embassy_time_schedule_wake(at, waker) }
}
