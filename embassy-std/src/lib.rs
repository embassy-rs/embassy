use embassy::executor::{raw, Spawner};
use embassy::time::driver::{AlarmHandle, Driver};
use embassy::time::TICKS_PER_SECOND;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::{Condvar, Mutex};
use std::time::{Duration as StdDuration, Instant as StdInstant};

static mut CLOCK_ZERO: MaybeUninit<StdInstant> = MaybeUninit::uninit();

static mut ALARM_AT: u64 = u64::MAX;
static mut NEXT_ALARM_ID: u8 = 0;

struct TimeDriver;
embassy::time_driver_impl!(TimeDriver);

impl Driver for TimeDriver {
    fn now() -> u64 {
        let zero = unsafe { CLOCK_ZERO.as_ptr().read() };
        let dur = StdInstant::now().duration_since(zero);
        dur.as_secs() * (TICKS_PER_SECOND as u64)
            + (dur.subsec_nanos() as u64) * (TICKS_PER_SECOND as u64) / 1_000_000_000
    }

    unsafe fn allocate_alarm() -> Option<AlarmHandle> {
        let r = NEXT_ALARM_ID;
        NEXT_ALARM_ID += 1;
        Some(AlarmHandle::new(r))
    }

    fn set_alarm_callback(_alarm: AlarmHandle, _callback: fn(*mut ()), _ctx: *mut ()) {}

    fn set_alarm(_alarm: AlarmHandle, timestamp: u64) {
        unsafe { ALARM_AT = ALARM_AT.min(timestamp) }
    }
}

struct Signaler {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Signaler {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            let alarm_at = unsafe { ALARM_AT };
            if alarm_at == u64::MAX {
                signaled = self.condvar.wait(signaled).unwrap();
            } else {
                unsafe { ALARM_AT = u64::MAX };
                let now = TimeDriver::now();
                if now >= alarm_at {
                    break;
                }

                let left = alarm_at - now;
                let dur = StdDuration::new(
                    left / (TICKS_PER_SECOND as u64),
                    (left % (TICKS_PER_SECOND as u64) * 1_000_000_000 / (TICKS_PER_SECOND as u64))
                        as u32,
                );
                let (signaled2, timeout) = self.condvar.wait_timeout(signaled, dur).unwrap();
                signaled = signaled2;
                if timeout.timed_out() {
                    break;
                }
            }
        }
        *signaled = false;
    }

    fn signal(ctx: *mut ()) {
        let this = unsafe { &*(ctx as *mut Self) };
        let mut signaled = this.mutex.lock().unwrap();
        *signaled = true;
        this.condvar.notify_one();
    }
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
    signaler: Signaler,
}

impl Executor {
    pub fn new() -> Self {
        unsafe {
            CLOCK_ZERO.as_mut_ptr().write(StdInstant::now());
        }

        Self {
            inner: raw::Executor::new(Signaler::signal, ptr::null_mut()),
            not_send: PhantomData,
            signaler: Signaler::new(),
        }
    }

    /// Runs the executor.
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        self.inner.set_signal_ctx(&self.signaler as *const _ as _);

        init(unsafe { self.inner.spawner() });

        loop {
            unsafe { self.inner.run_queued() };
            self.signaler.wait();
        }
    }
}
