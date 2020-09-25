use core::marker::PhantomData;
use static_executor as se;

use crate::time;
use crate::time::Alarm;

pub use se::{task, SpawnError, SpawnToken};

pub trait Model {
    fn signal();
}

pub struct WfeModel;

impl Model for WfeModel {
    fn signal() {
        cortex_m::asm::sev()
    }
}

pub struct Executor<M, A: Alarm> {
    inner: se::Executor,
    alarm: A,
    timer: time::TimerService,
    _phantom: PhantomData<M>,
}

impl<M: Model, A: Alarm> Executor<M, A> {
    pub fn new(alarm: A) -> Self {
        alarm.set_callback(M::signal);
        Self {
            inner: se::Executor::new(M::signal),
            alarm,
            timer: time::TimerService::new(time::IntrusiveClock),
            _phantom: PhantomData,
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
    pub unsafe fn run_once(&'static self) {
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

impl<A: Alarm> Executor<WfeModel, A> {
    /// Runs the executor forever
    /// safety: can only be called from the executor thread
    pub unsafe fn run(&'static self) -> ! {
        loop {
            self.run_once();
            cortex_m::asm::wfe()
        }
    }
}
