use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
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
