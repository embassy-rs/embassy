#![allow(non_snake_case)]

use core::cell::Cell;
use core::ptr;

#[cfg_attr(
    any(
        time_driver_tim1,
        time_driver_tim2,
        time_driver_tim3,
        time_driver_tim4,
        time_driver_tim5,
        time_driver_tim8,
        time_driver_tim9,
        time_driver_tim12,
        time_driver_tim15,
        time_driver_tim20,
        time_driver_tim21,
        time_driver_tim22,
        time_driver_tim23,
        time_driver_tim24
    ),
    path = "tim.rs"
)]
#[cfg_attr(any(time_driver_lptim1, time_driver_lptim2), path = "lptim.rs")]
mod _timer;
#[allow(unused_imports)]
pub use _timer::*;

struct AlarmState {
    timestamp: Cell<u64>,

    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: Cell<*const ()>,
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const ALARM_STATE_NEW: AlarmState = AlarmState::new();
