use core::sync::atomic::{AtomicU32, Ordering};

/// A type for interacting with the deadline of the current task
///
/// Requires the `scheduler-deadline` feature.
///
/// Note: Interacting with the deadline should be done locally in a task.
/// In theory you could try to set or read the deadline from another task,
/// but that will result in weird (though not unsound) behavior.
pub(crate) struct Deadline {
    instant_ticks_hi: AtomicU32,
    instant_ticks_lo: AtomicU32,
}

impl Deadline {
    pub(crate) const fn new(instant_ticks: u64) -> Self {
        Self {
            instant_ticks_hi: AtomicU32::new((instant_ticks >> 32) as u32),
            instant_ticks_lo: AtomicU32::new(instant_ticks as u32),
        }
    }

    pub(crate) const fn new_unset() -> Self {
        Self::new(Self::UNSET_TICKS)
    }

    pub(crate) fn set(&self, instant_ticks: u64) {
        self.instant_ticks_hi
            .store((instant_ticks >> 32) as u32, Ordering::Relaxed);
        self.instant_ticks_lo.store(instant_ticks as u32, Ordering::Relaxed);
    }

    /// Deadline value in ticks, same time base and ticks as `embassy-time`
    pub(crate) fn instant_ticks(&self) -> u64 {
        let hi = self.instant_ticks_hi.load(Ordering::Relaxed) as u64;
        let lo = self.instant_ticks_lo.load(Ordering::Relaxed) as u64;

        (hi << 32) | lo
    }

    /// Sentinel value representing an "unset" deadline, which has lower priority
    /// than any other set deadline value
    pub(crate) const UNSET_TICKS: u64 = u64::MAX;
}
