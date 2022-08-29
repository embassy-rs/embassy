use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Yield from the current task once, allowing other tasks to run.
///
/// This can be used to easily and quickly implement simple async primitives
/// without using wakers. The following snippet will wait for a condition to
/// hold, while still allowing other tasks to run concurrently (not monopolizing
/// the executor thread).
///
/// ```rust,no_run
/// while !some_condition() {
///     yield_now().await;
/// }
/// ```
///
/// The downside is this will spin in a busy loop, using 100% of the CPU, while
/// using wakers correctly would allow the CPU to sleep while waiting.
///
/// The internal implementation is: on first poll the future wakes itself and
/// returns `Poll::Pending`. On second poll, it returns `Poll::Ready`.
pub fn yield_now() -> impl Future<Output = ()> {
    YieldNowFuture { yielded: false }
}

struct YieldNowFuture {
    yielded: bool,
}

impl Future for YieldNowFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
