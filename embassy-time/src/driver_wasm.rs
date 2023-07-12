use core::sync::atomic::{AtomicU8, Ordering};
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::{Mutex, Once};

use wasm_bindgen::prelude::*;
use wasm_timer::Instant as StdInstant;

use crate::driver::{AlarmHandle, Driver};

const ALARM_COUNT: usize = 4;

struct AlarmState {
    token: Option<f64>,
    closure: Option<Closure<dyn FnMut() + 'static>>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            token: None,
            closure: None,
        }
    }
}

#[wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearTimeout(token: f64);
}

struct TimeDriver {
    alarm_count: AtomicU8,

    once: Once,
    alarms: UninitCell<Mutex<[AlarmState; ALARM_COUNT]>>,
    zero_instant: UninitCell<StdInstant>,
}

const ALARM_NEW: AlarmState = AlarmState::new();
crate::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    alarm_count: AtomicU8::new(0),
    once: Once::new(),
    alarms: UninitCell::uninit(),
    zero_instant: UninitCell::uninit(),
});

impl TimeDriver {
    fn init(&self) {
        self.once.call_once(|| unsafe {
            self.alarms.write(Mutex::new([ALARM_NEW; ALARM_COUNT]));
            self.zero_instant.write(StdInstant::now());
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        self.init();

        let zero = unsafe { self.zero_instant.read() };
        StdInstant::now().duration_since(zero).as_micros() as u64
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let id = self.alarm_count.fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
            if x < ALARM_COUNT as u8 {
                Some(x + 1)
            } else {
                None
            }
        });

        match id {
            Ok(id) => Some(AlarmHandle::new(id)),
            Err(_) => None,
        }
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        self.init();
        let mut alarms = unsafe { self.alarms.as_ref() }.lock().unwrap();
        let alarm = &mut alarms[alarm.id() as usize];
        alarm.closure.replace(Closure::new(move || {
            callback(ctx);
        }));
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        self.init();
        let mut alarms = unsafe { self.alarms.as_ref() }.lock().unwrap();
        let alarm = &mut alarms[alarm.id() as usize];
        if let Some(token) = alarm.token {
            clearTimeout(token);
        }

        let now = self.now();
        if timestamp <= now {
            false
        } else {
            let timeout = (timestamp - now) as u32;
            alarm.token = Some(setTimeout(alarm.closure.as_ref().unwrap(), timeout / 1000));

            true
        }
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
