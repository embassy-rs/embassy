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
    /// Compute whether an alarm is currently pending in the time driver
    fn has_pending_alarm(&self, cs: CriticalSection) -> bool;

    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&self, cs: CriticalSection) -> embassy_time::Duration;

    /// Set the minimum pause floor beyond which the executor will enter a low-power state.
    fn set_min_stop_pause(&self, cs: CriticalSection, min_stop_pause: embassy_time::Duration);

    /// User-configured minimum pause floor.
    fn min_stop_pause(&self, cs: CriticalSection) -> embassy_time::Duration;

    /// Time-driver stop/resume overhead in microseconds (beyond RCC clock restoration).
    fn driver_overhead_us(&self) -> u32;

    /// Pause the timer if ready; return err if not
    fn pause_time(&self, cs: CriticalSection, exit_latency: embassy_time::Duration) -> Result<(), ()>;

    /// Resume the timer with the given offset
    fn resume_time(&self, cs: CriticalSection);
}

#[cfg_attr(feature = "_lp-time-driver", path = "lptim.rs")]
#[cfg_attr(not(feature = "_lp-time-driver"), path = "tim.rs")]
mod driver;
pub(crate) use driver::*;
