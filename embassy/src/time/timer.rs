use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::Stream;
use futures_intrusive::timer::{LocalTimer, LocalTimerFuture};

use super::{Duration, Instant};
use crate::executor::current_timer_queue;

pub struct Timer {
    inner: LocalTimerFuture<'static>,
}

impl Timer {
    pub fn at(when: Instant) -> Self {
        Self {
            inner: current_timer_queue().deadline(when.as_ticks()),
        }
    }

    pub fn after(dur: Duration) -> Self {
        Self::at(Instant::now() + dur)
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }.poll(cx)
    }
}

pub struct Ticker {
    inner: LocalTimerFuture<'static>,
    next: Instant,
    dur: Duration,
}

impl Ticker {
    pub fn every(dur: Duration) -> Self {
        let next = Instant::now() + dur;
        Self {
            inner: current_timer_queue().deadline(next.as_ticks()),
            next,
            dur,
        }
    }
}

impl Stream for Ticker {
    type Item = ();
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = unsafe { self.get_unchecked_mut() };
        match unsafe { Pin::new_unchecked(&mut this.inner) }.poll(cx) {
            Poll::Ready(_) => {
                this.next += this.dur;
                this.inner = current_timer_queue().deadline(this.next.as_ticks());
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
