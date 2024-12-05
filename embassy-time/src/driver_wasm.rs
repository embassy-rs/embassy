use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::{Mutex, Once};

use embassy_time_driver::Driver;
use wasm_bindgen::prelude::*;
use wasm_timer::Instant as StdInstant;

struct AlarmState {
    token: Option<f64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self { token: None }
    }
}

#[wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearTimeout(token: f64);
}

struct TimeDriver {
    once: Once,
    alarm: UninitCell<Mutex<AlarmState>>,
    zero_instant: UninitCell<StdInstant>,
    closure: UninitCell<Closure<dyn FnMut()>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    once: Once::new(),
    alarm: UninitCell::uninit(),
    zero_instant: UninitCell::uninit(),
    closure: UninitCell::uninit()
});

impl TimeDriver {
    fn init(&self) {
        self.once.call_once(|| unsafe {
            self.alarm.write(Mutex::new(const { AlarmState::new() }));
            self.zero_instant.write(StdInstant::now());
            self.closure
                .write(Closure::new(Box::new(|| TIMER_QUEUE_DRIVER.dispatch())));
        });
    }

    fn set_alarm(&self, timestamp: u64) -> bool {
        self.init();
        let mut alarm = unsafe { self.alarm.as_ref() }.lock().unwrap();
        if let Some(token) = alarm.token {
            clearTimeout(token);
        }

        let now = self.now();
        if timestamp <= now {
            false
        } else {
            let timeout = (timestamp - now) as u32;
            alarm.token = Some(setTimeout(unsafe { self.closure.as_ref() }, timeout / 1000));

            true
        }
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        self.init();

        let zero = unsafe { self.zero_instant.read() };
        StdInstant::now().duration_since(zero).as_micros() as u64
    }
}

pub(crate) struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>);
unsafe impl<T> Send for UninitCell<T> {}
unsafe impl<T> Sync for UninitCell<T> {}

impl<T> UninitCell<T> {
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }
    unsafe fn as_ptr(&self) -> *const T {
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

struct RawQueue {
    inner: core::cell::RefCell<embassy_time_queue_driver::queue_generic::Queue>,
}

impl RawQueue {
    const fn new() -> Self {
        Self {
            inner: core::cell::RefCell::new(embassy_time_queue_driver::queue_generic::Queue::new()),
        }
    }

    fn schedule_wake(&self, waker: &core::task::Waker, at: u64) -> bool {
        self.inner.borrow_mut().schedule_wake(at, waker)
    }

    fn next_expiration(&self, now: u64) -> u64 {
        self.inner.borrow_mut().next_expiration(now)
    }
}

struct TimerQueueDriver {
    inner: Mutex<RawQueue>,
}

embassy_time_queue_driver::timer_queue_impl!(static TIMER_QUEUE_DRIVER: TimerQueueDriver = TimerQueueDriver::new());

impl embassy_time_queue_driver::TimerQueue for TimerQueueDriver {
    fn schedule_wake(&'static self, at: u64, waker: &core::task::Waker) {
        let q = self.inner.lock().unwrap();
        if q.schedule_wake(waker, at) {
            self.arm_alarm(at);
        }
    }
}

impl TimerQueueDriver {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(RawQueue::new()),
        }
    }

    pub fn dispatch(&self) {
        let now = DRIVER.now();
        let q = self.inner.lock().unwrap();
        let next_expiration = q.next_expiration(now);
        self.arm_alarm(next_expiration);
    }

    fn arm_alarm(&self, mut next_expiration: u64) {
        while !DRIVER.set_alarm(next_expiration) {
            // next_expiration is in the past, dequeue and find a new expiration
            let q = self.inner.lock().unwrap();
            next_expiration = q.next_expiration(next_expiration);
        }
    }
}
