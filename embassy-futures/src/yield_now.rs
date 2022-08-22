use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Yield from the current task once, allowing other tasks to run.
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
