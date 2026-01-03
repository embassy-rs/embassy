use core::cell::Cell;

// Common AlarmState struct used by both implementations
pub(crate) struct AlarmState {
    pub(crate) timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    pub(crate) const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

#[cfg_attr(feature = "_lp-time-driver", path = "lptim.rs")]
#[cfg_attr(not(feature = "_lp-time-driver"), path = "gp16.rs")]
mod driver;
pub(crate) use driver::*;
