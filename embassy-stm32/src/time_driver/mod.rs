use core::cell::Cell;

#[cfg(feature = "low-power")]
use critical_section::CriticalSection;

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

#[cfg(feature = "low-power")]
pub(crate) trait LPTimeDriver {
    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&self, cs: CriticalSection) -> embassy_time::Duration;

    /// Set the minimum pause time beyond which the executor will enter a low-power state.
    #[cfg(not(feature = "_lp-time-driver"))]
    fn set_min_stop_pause(&self, cs: CriticalSection, min_stop_pause: embassy_time::Duration);

    /// Set the rtc but panic if it's already been set
    #[cfg(not(feature = "_lp-time-driver"))]
    fn set_rtc(&self, cs: CriticalSection, rtc: crate::rtc::Rtc);

    /// Pause the timer if ready; return err if not
    fn pause_time(&self, cs: CriticalSection) -> Result<(), ()>;

    /// Resume the timer with the given offset
    fn resume_time(&self, cs: CriticalSection);

    /// Returns whether time is currently stopped
    fn is_stopped(&self) -> bool;
}

#[cfg_attr(feature = "_lp-time-driver", path = "lptim.rs")]
#[cfg_attr(not(feature = "_lp-time-driver"), path = "gp16.rs")]
mod driver;
pub(crate) use driver::*;
