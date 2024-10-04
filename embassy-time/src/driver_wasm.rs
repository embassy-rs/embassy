use core::sync::atomic::{AtomicU8, Ordering};
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::{Mutex, Once};

use embassy_time_driver::{AlarmHandle, Driver};
use wasm_bindgen::prelude::*;

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
}

const ALARM_NEW: AlarmState = AlarmState::new();
embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    alarm_count: AtomicU8::new(0),
    once: Once::new(),
    alarms: UninitCell::uninit(),
});

#[cfg(feature = "panic_on_webworker")]
thread_local! {
    static CHECK_THREAD: Once = Once::new();
}

impl TimeDriver {
    fn ensure_init(&self) {
        self.once.call_once(|| unsafe {
            self.alarms.write(Mutex::new([ALARM_NEW; ALARM_COUNT]));
        });
        #[cfg(feature = "panic_on_webworker")]
        CHECK_THREAD.with(|val| {
            val.call_once(|| {
                assert!(
                    !is_web_worker_thread(),
                    "Timer currently has issues on Web Workers: https://github.com/embassy-rs/embassy/issues/3313"
                );
            })
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        self.ensure_init();
        // this is calibrated with timeOrigin.
        now_as_calibrated_timestamp().as_micros() as u64
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
        self.ensure_init();
        let mut alarms = unsafe { self.alarms.as_ref() }.lock().unwrap();
        let alarm = &mut alarms[alarm.id() as usize];
        alarm.closure.replace(Closure::new(move || {
            callback(ctx);
        }));
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        self.ensure_init();
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

fn is_web_worker_thread() -> bool {
    js_sys::eval("typeof WorkerGlobalScope !== 'undefined' && self instanceof WorkerGlobalScope")
        .unwrap()
        .is_truthy()
}

// ---------------- taken from web-time/js.rs
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

#[wasm_bindgen]
extern "C" {
    /// Type for the [global object](https://developer.mozilla.org/en-US/docs/Glossary/Global_object).
    type Global;

    /// Returns the [`Performance`](https://developer.mozilla.org/en-US/docs/Web/API/Performance) object.
    #[wasm_bindgen(method, getter)]
    fn performance(this: &Global) -> JsValue;

    /// Type for the [`Performance` object](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
    pub(super) type Performance;

    /// Binding to [`Performance.now()`](https://developer.mozilla.org/en-US/docs/Web/API/Performance/now).
    #[wasm_bindgen(method)]
    pub(super) fn now(this: &Performance) -> f64;

    /// Binding to [`Performance.timeOrigin`](https://developer.mozilla.org/en-US/docs/Web/API/Performance/timeOrigin).
    #[wasm_bindgen(method, getter, js_name = timeOrigin)]
    pub(super) fn time_origin(this: &Performance) -> f64;
}

thread_local! {
    pub(super) static PERFORMANCE: Performance = {
        let global: Global = js_sys::global().unchecked_into();
        let performance = global.performance();

        if performance.is_undefined() {
            panic!("`Performance` object not found")
        } else {
            performance.unchecked_into()
        }
    };
}

// ---------------- taken from web-time/instant.rs

thread_local! {
    static ORIGIN: f64 = PERFORMANCE.with(Performance::time_origin);
}

/// This will get a Duration from a synchronized start point, whether in webworkers or the main browser thread.
///
///  # Panics
///
/// This call will panic if the [`Performance` object] was not found, e.g.
/// calling from a [worklet].
///
/// [`Performance` object]: https://developer.mozilla.org/en-US/docs/Web/API/performance_property
/// [worklet]: https://developer.mozilla.org/en-US/docs/Web/API/Worklet
#[must_use]
pub fn now_as_calibrated_timestamp() -> core::time::Duration {
    let now = PERFORMANCE.with(|performance| {
        return ORIGIN.with(|origin| performance.now() + origin);
    });
    time_stamp_to_duration(now)
}

/// Converts a `DOMHighResTimeStamp` to a [`Duration`].
///
/// # Note
///
/// Keep in mind that like [`Duration::from_secs_f64()`] this doesn't do perfect
/// rounding.
#[allow(clippy::as_conversions, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn time_stamp_to_duration(time_stamp: f64) -> core::time::Duration {
    core::time::Duration::from_millis(time_stamp.trunc() as u64)
        + core::time::Duration::from_nanos((time_stamp.fract() * 1.0e6).round() as u64)
}
