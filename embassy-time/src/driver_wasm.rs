use std::sync::Mutex;

use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use wasm_bindgen::prelude::*;
use wasm_timer::Instant as StdInstant;

struct AlarmState {
    token: Option<f64>,
}

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
    inner: Mutex<Inner>,
}

struct Inner {
    alarm: AlarmState,
    zero_instant: Option<StdInstant>,
    queue: Queue,
    closure: Option<Closure<dyn FnMut()>>,
}

unsafe impl Send for Inner {}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    inner: Mutex::new(Inner{
        zero_instant: None,
        queue: Queue::new(),
        alarm: AlarmState::new(),
        closure: None,
    }),
});

impl Inner {
    fn init(&mut self) -> StdInstant {
        *self.zero_instant.get_or_insert_with(StdInstant::now)
    }

    fn now(&mut self) -> u64 {
        StdInstant::now().duration_since(self.zero_instant.unwrap()).as_micros() as u64
    }

    fn set_alarm(&mut self, timestamp: u64) -> bool {
        if let Some(token) = self.alarm.token {
            clearTimeout(token);
        }

        let now = self.now();
        if timestamp <= now {
            false
        } else {
            let timeout = (timestamp - now) as u32;
            let closure = self.closure.get_or_insert_with(|| Closure::new(dispatch));
            self.alarm.token = Some(setTimeout(closure, timeout / 1000));

            true
        }
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let zero = inner.init();
        StdInstant::now().duration_since(zero).as_micros() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        let mut inner = self.inner.lock().unwrap();
        inner.init();
        if inner.queue.schedule_wake(at, waker) {
            let now = inner.now();
            let mut next = inner.queue.next_expiration(now);
            while !inner.set_alarm(next) {
                let now = inner.now();
                next = inner.queue.next_expiration(now);
            }
        }
    }
}

fn dispatch() {
    let inner = &mut *DRIVER.inner.lock().unwrap();

    let now = inner.now();
    let mut next = inner.queue.next_expiration(now);
    while !inner.set_alarm(next) {
        let now = inner.now();
        next = inner.queue.next_expiration(now);
    }
}
