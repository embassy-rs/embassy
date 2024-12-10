use std::cell::{RefCell, UnsafeCell};
use std::mem::MaybeUninit;
use std::sync::{Condvar, Mutex, Once};
use std::time::{Duration as StdDuration, Instant as StdInstant};
use std::{ptr, thread};

use critical_section::Mutex as CsMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_driver::GlobalTimerQueue;

struct AlarmState {
    timestamp: u64,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self { timestamp: u64::MAX }
    }
}

struct TimeDriver {
    once: Once,
    // The STD Driver implementation requires the alarm's mutex to be reentrant, which the STD Mutex isn't
    // Fortunately, mutexes based on the `critical-section` crate are reentrant, because the critical sections
    // themselves are reentrant
    alarm: UninitCell<CsMutex<RefCell<AlarmState>>>,
    zero_instant: UninitCell<StdInstant>,
    signaler: UninitCell<Signaler>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    once: Once::new(),
    alarm: UninitCell::uninit(),
    zero_instant: UninitCell::uninit(),
    signaler: UninitCell::uninit(),
});

impl TimeDriver {
    fn init(&self) {
        self.once.call_once(|| unsafe {
            self.alarm
                .write(CsMutex::new(RefCell::new(const { AlarmState::new() })));
            self.zero_instant.write(StdInstant::now());
            self.signaler.write(Signaler::new());

            thread::spawn(Self::alarm_thread);
        });
    }

    fn alarm_thread() {
        let zero = unsafe { DRIVER.zero_instant.read() };
        loop {
            let now = DRIVER.now();

            let next_alarm = critical_section::with(|cs| {
                let mut alarm = unsafe { DRIVER.alarm.as_ref() }.borrow_ref_mut(cs);
                if alarm.timestamp <= now {
                    alarm.timestamp = u64::MAX;

                    TIMER_QUEUE_DRIVER.dispatch();
                }
                alarm.timestamp
            });

            // Ensure we don't overflow
            let until = zero
                .checked_add(StdDuration::from_micros(next_alarm))
                .unwrap_or_else(|| StdInstant::now() + StdDuration::from_secs(1));

            unsafe { DRIVER.signaler.as_ref() }.wait_until(until);
        }
    }

    fn set_alarm(&self, timestamp: u64) -> bool {
        self.init();
        critical_section::with(|cs| {
            let mut alarm = unsafe { self.alarm.as_ref() }.borrow_ref_mut(cs);
            alarm.timestamp = timestamp;
            unsafe { self.signaler.as_ref() }.signal();
        });

        true
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        self.init();

        let zero = unsafe { self.zero_instant.read() };
        StdInstant::now().duration_since(zero).as_micros() as u64
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

    fn wait_until(&self, until: StdInstant) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            let now = StdInstant::now();

            if now >= until {
                break;
            }

            let dur = until - now;
            let (signaled2, timeout) = self.condvar.wait_timeout(signaled, dur).unwrap();
            signaled = signaled2;
            if timeout.timed_out() {
                break;
            }
        }
        *signaled = false;
    }

    fn signal(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        *signaled = true;
        self.condvar.notify_one();
    }
}

pub(crate) struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>);
unsafe impl<T> Send for UninitCell<T> {}
unsafe impl<T> Sync for UninitCell<T> {}

impl<T> UninitCell<T> {
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    pub unsafe fn as_ptr(&self) -> *const T {
        (*self.0.as_ptr()).get()
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        (*self.0.as_ptr()).get()
    }

    pub unsafe fn as_ref(&self) -> &T {
        &*self.as_ptr()
    }

    pub unsafe fn write(&self, val: T) {
        ptr::write(self.as_mut_ptr(), val)
    }
}

impl<T: Copy> UninitCell<T> {
    pub unsafe fn read(&self) -> T {
        ptr::read(self.as_mut_ptr())
    }
}

embassy_time_queue_driver::timer_queue_impl!(
    static TIMER_QUEUE_DRIVER: GlobalTimerQueue
        = GlobalTimerQueue::new(|next_expiration| DRIVER.set_alarm(next_expiration))
);
