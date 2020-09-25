use core::marker::PhantomData;
use static_executor as se;

use crate::time;
use crate::time::Alarm;

pub use se::{task, SpawnError, SpawnToken};

pub struct Executor<A: Alarm> {
    inner: se::Executor,
    alarm: A,
    timer: time::TimerService,
}

impl<A: Alarm> Executor<A> {
    pub fn new(alarm: A, signal_fn: fn()) -> Self {
        alarm.set_callback(signal_fn);
        Self {
            inner: se::Executor::new(signal_fn),
            alarm,
            timer: time::TimerService::new(time::IntrusiveClock),
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
        time::with_timer_service(&self.timer, || {
            self.timer.check_expirations();
            self.inner.run();

            match self.timer.next_expiration() {
                // If this is in the past, set_alarm will immediately trigger the alarm,
                // which will make the wfe immediately return so we do another loop iteration.
                Some(at) => self.alarm.set(at),
                None => self.alarm.clear(),
            }
        })
    }
}
