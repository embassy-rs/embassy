//! Trace polls of [`Timer`](crate::Timer) futures, including timers used for delays.
//!
//! Enable `trace` and register one [`Trace`] implementation with [`trace_impl!`](crate::trace_impl).
//! A pending poll can repeat or its future may be dropped before another poll. A ready poll
//! records when a caller observes expiration, which may be later than the hardware alarm.
//! Scheduling a wake can notify the executor before the pending-poll callback.
//! Several timers can share a waker and deadline. Ticker and driver alarms are outside this hook.

pub use core::task::{Poll, Waker};

pub use crate::Instant;

unitrait::unitrait! {
    /// Timer future trace hooks.
    #[symbol_prefix = "_embassy_time_trace_v1"]
    pub trait Trace {
        /// Called on each timer poll with the deadline and the result about to be returned.
        /// Implementations must not panic, block, or wake or poll tasks.
        fn timer_poll(deadline: Instant, waker: &Waker, result: Poll<()>);
    }
    /// The registered timer trace implementation.
    pub(crate) struct TraceImpl;
    /// Register an implementation of the timer trace hooks.
    macro trace_impl(path = $crate::trace);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    std::thread_local! { static POLLS: RefCell<Vec<(Instant, Poll<()>)>> = const { RefCell::new(Vec::new()) }; }
    struct Recorder;
    trace_impl!(Recorder);
    impl Trace for Recorder {
        fn timer_poll(deadline: Instant, _: &Waker, result: Poll<()>) {
            POLLS.with_borrow_mut(|events| events.push((deadline, result)));
        }
    }

    #[cfg(all(feature = "mock-driver", feature = "generic-queue-8"))]
    #[test]
    fn past_deadline_yields_once_and_cancellation_does_not_report_expiry() {
        use std::future::Future;
        use std::pin::pin;
        use std::task::Context;

        use crate::Timer;

        let mut cx = Context::from_waker(Waker::noop());
        let past = Instant::from_ticks(0);
        let future = Instant::from_ticks(u64::MAX - 1);
        {
            let mut cancelled = pin!(Timer::at(future));
            assert!(cancelled.as_mut().poll(&mut cx).is_pending());
        }
        let mut expired = pin!(Timer::at(past));
        assert!(expired.as_mut().poll(&mut cx).is_pending());
        assert!(expired.as_mut().poll(&mut cx).is_ready());
        assert_eq!(
            POLLS.take(),
            [(future, Poll::Pending), (past, Poll::Pending), (past, Poll::Ready(()))]
        );
    }
}
