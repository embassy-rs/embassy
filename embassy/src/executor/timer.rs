use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::Stream;

use crate::time::{Duration, Instant};

pub struct Timer {
    expires_at: Instant,
}

impl Timer {
    pub fn at(expires_at: Instant) -> Self {
        Self { expires_at }
    }

    pub fn after(duration: Duration) -> Self {
        Self {
            expires_at: Instant::now() + duration,
        }
    }
}

impl Unpin for Timer {}

impl Future for Timer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.expires_at <= Instant::now() {
            Poll::Ready(())
        } else {
            unsafe { super::register_timer(self.expires_at, cx.waker()) };
            Poll::Pending
        }
    }
}

pub struct Ticker {
    expires_at: Instant,
    duration: Duration,
}

impl Ticker {
    pub fn every(duration: Duration) -> Self {
        let expires_at = Instant::now() + duration;
        Self {
            expires_at,
            duration,
        }
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
            unsafe { super::register_timer(self.expires_at, cx.waker()) };
            Poll::Pending
        }
    }
}
