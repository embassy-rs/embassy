use super::executor::{Executor, SpawnError, SpawnToken};
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use futures_intrusive::timer as fi;

use crate::time::Alarm;

pub(crate) struct IntrusiveClock;

impl fi::Clock for IntrusiveClock {
    fn now(&self) -> u64 {
        crate::time::now()
    }
}

pub(crate) type TimerQueue = fi::LocalTimerService;

pub struct TimerExecutor<A: Alarm> {
    inner: Executor,
    alarm: A,
    timer_queue: TimerQueue,
}

impl<A: Alarm> TimerExecutor<A> {
    pub fn new(alarm: A, signal_fn: fn()) -> Self {
        alarm.set_callback(signal_fn);
        Self {
            inner: Executor::new(signal_fn),
            alarm,
            timer_queue: TimerQueue::new(&IntrusiveClock),
        }
    }

    /// Spawn a future on this executor.
    ///
    /// safety: can only be called from the executor thread
    pub unsafe fn spawn(&'static self, token: SpawnToken) -> Result<(), SpawnError> {
        self.inner.spawn(token)
    }

    /// Runs the executor until the queue is empty.
    ///
    /// safety: can only be called from the executor thread
    pub unsafe fn run(&'static self) {
        with_timer_queue(&self.timer_queue, || {
            self.timer_queue.check_expirations();
            self.inner.run();

            match self.timer_queue.next_expiration() {
                // If this is in the past, set_alarm will immediately trigger the alarm,
                // which will make the wfe immediately return so we do another loop iteration.
                Some(at) => self.alarm.set(at),
                None => self.alarm.clear(),
            }
        })
    }
}

static CURRENT_TIMER_QUEUE: AtomicPtr<TimerQueue> = AtomicPtr::new(ptr::null_mut());

fn with_timer_queue<R>(svc: &'static TimerQueue, f: impl FnOnce() -> R) -> R {
    let svc = svc as *const _ as *mut _;
    let prev_svc = CURRENT_TIMER_QUEUE.swap(svc, Ordering::Relaxed);
    let r = f();
    let svc2 = CURRENT_TIMER_QUEUE.swap(prev_svc, Ordering::Relaxed);
    assert_eq!(svc, svc2);
    r
}

pub(crate) fn current_timer_queue() -> &'static TimerQueue {
    unsafe {
        CURRENT_TIMER_QUEUE
            .load(Ordering::Relaxed)
            .as_ref()
            .unwrap()
    }
}
