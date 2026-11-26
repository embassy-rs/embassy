//! Helpers for implementing isochronous usb transfers

use core::num::NonZeroU32;

/// The slot [`IsoSchedule::offer`] picked, including the frame-counter value to arm.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct IsoOffer {
    /// Target frame-counter value of the slot, in `0..span`.
    pub frame: u32,
    /// Service intervals left empty by the writer before this slot, if any.
    pub skipped: Option<NonZeroU32>,
}

impl IsoOffer {
    /// A slot at `frame` on the frame counter, with `skipped` empty intervals before it.
    pub const fn new(frame: u32, skipped: Option<NonZeroU32>) -> Self {
        Self { frame, skipped }
    }
}

/// Next isochronous IN slot, aligned to the bus frame clock and host polling phase.
///
/// Service intervals occur every `interval` ticks of a frame counter of modulus `span`
/// (2048 at full speed, 16384 at high speed). `interval` should divide `span` (true for all
/// USB power-of-two intervals).
///
/// For `interval == 1`, every tick is a service interval and the phase is trivially `0`.
/// For `interval > 1`, the USB host chooses an arbitrary phase in `0..interval` when scheduling
/// the endpoint. Until [`lock_phase`](Self::lock_phase) is called with the frame of a completed
/// IN transfer, the schedule paces at `interval` ticks with provisional phase `0`. Once locked,
/// all subsequent slots align to `frame % interval == phase` until [`reset`](Self::reset).
///
/// A gap of more than one revolution of the frame counter is counted as a single wrap.
#[derive(Clone, Debug)]
pub struct IsoSchedule {
    last_now: Option<u32>,
    epoch: u32,
    phase: Option<u32>,
    last_frame: Option<u64>,
    last_written: Option<u64>,
}

impl IsoSchedule {
    /// A schedule that has not offered a slot yet.
    pub const fn new() -> Self {
        Self {
            last_now: None,
            epoch: 0,
            phase: None,
            last_frame: None,
            last_written: None,
        }
    }

    /// Clear all tracking state, including any locked host phase.
    ///
    /// Call when the endpoint is disabled or the bus resets.
    pub fn reset(&mut self) {
        self.last_now = None;
        self.epoch = 0;
        self.phase = None;
        self.last_frame = None;
        self.last_written = None;
    }

    /// Lock the host polling phase from a completed IN transfer at `frame`.
    ///
    /// Subsequent calls while already locked have no effect until [`reset`](Self::reset).
    pub fn lock_phase(&mut self, frame: u32, interval: u32) {
        if self.phase.is_none() {
            self.phase = Some(frame % interval.max(1));
        }
    }

    /// Offer the next slot when the frame counter reads `now`.
    ///
    /// `interval` is the service interval in frame-counter ticks, at least 1 and less than
    /// `span`. `span` is the counter modulus. `now` is taken modulo `span`.
    pub fn offer(&mut self, now: u32, interval: u32, span: u32) -> IsoOffer {
        let interval = u64::from(interval.max(1));
        let span = u64::from(span.max(1));
        let now = u64::from(now) % span;
        let extended = self.observe(now, span);

        let phase = u64::from(self.phase.unwrap_or(0)) % interval;
        // Smallest frame after `extended` and the last written interval, with `frame % interval == phase`.
        let min_frame = match self.last_written {
            Some(written) => (extended + 1).max(written + interval),
            None => extended + 1,
        };
        let rem = min_frame % interval;
        let add = if phase >= rem {
            phase - rem
        } else {
            interval - (rem - phase)
        };
        let frame = min_frame + add;

        // Cumulative since the last write; nothing counts as skipped before the first one.
        let skipped = self
            .last_written
            .and_then(|written| NonZeroU32::new((frame.saturating_sub(written) / interval).saturating_sub(1) as u32));

        self.last_frame = Some(frame);

        IsoOffer::new((frame % span) as u32, skipped)
    }

    /// Record a write attempt for the most recently offered slot.
    /// Later offers count skipped intervals after this slot, even if the write failed.
    pub fn accept(&mut self) {
        if self.last_frame.is_some() {
            self.last_written = self.last_frame;
        }
    }

    fn observe(&mut self, now: u64, span: u64) -> u64 {
        let now_u = now as u32;
        if let Some(prev) = self.last_now {
            // The counter only moves forward. A smaller sample is the next revolution.
            if now_u < prev {
                self.epoch = self.epoch.wrapping_add(1);
            }
        }
        self.last_now = Some(now_u);
        u64::from(self.epoch) * span + now
    }
}

impl Default for IsoSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Ticks of the frame counter from `from` forward to `to`, in `0..span`.
pub const fn forward(from: u32, to: u32, span: u32) -> u32 {
    let span = if span == 0 { 1 } else { span };
    (to % span + span - from % span) % span
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_one_offers_the_next_tick_then_the_one_after_it() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(10, 1, 2048);
        assert_eq!(first.frame, 11);
        assert_eq!(first.skipped, None);

        schedule.accept();
        let next = schedule.offer(11, 1, 2048);
        assert_eq!(next.frame, 12);
        assert_eq!(next.skipped, None);
    }

    #[test]
    fn a_late_writer_skips_the_gap_instead_of_the_slots_it_missed_one_by_one() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(10, 1, 2048);
        assert_eq!(first.frame, 11);
        schedule.accept();

        // Thirty ticks pass with nothing queued. The offered slot is the next one that has
        // not started, and the thirty in between were left empty.
        let resumed = schedule.offer(41, 1, 2048);
        assert_eq!(resumed.frame, 42);
        assert_eq!(resumed.skipped, NonZeroU32::new(30));
    }

    #[test]
    fn holding_a_slot_that_is_still_ahead_does_not_count_as_skipped() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(5, 8, 16384);
        assert_eq!(first.frame, 8);
        assert_eq!(first.skipped, None);

        let again = schedule.offer(7, 8, 16384);
        assert_eq!(again.frame, 8);
        assert_eq!(again.skipped, None);
    }

    #[test]
    fn an_accepted_slot_followed_by_next_interval_is_not_a_skip() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(0, 8, 16384);
        assert_eq!(first.frame, 8);
        schedule.accept();

        let next = schedule.offer(8, 8, 16384);
        assert_eq!(next.frame, 16);
        assert_eq!(next.skipped, None);
    }

    #[test]
    fn nothing_counts_as_skipped_before_the_first_write() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(0, 8, 16384);
        assert_eq!(first.frame, 8);

        let next = schedule.offer(40, 8, 16384);
        assert_eq!(next.frame, 48);
        assert_eq!(next.skipped, None);
    }

    #[test]
    fn an_offered_slot_that_was_not_queued_counts_as_skipped_once_it_starts() {
        let mut schedule = IsoSchedule::new();
        schedule.offer(0, 8, 16384);
        schedule.accept();
        let first = schedule.offer(8, 8, 16384);
        assert_eq!(first.frame, 16);

        let next = schedule.offer(16, 8, 16384);
        assert_eq!(next.frame, 24);
        assert_eq!(next.skipped, NonZeroU32::new(1));
    }

    #[test]
    fn repeated_offers_report_the_count_since_the_write_not_a_sum() {
        let mut schedule = IsoSchedule::new();
        schedule.offer(10, 1, 2048);
        schedule.accept();

        assert_eq!(schedule.offer(14, 1, 2048).skipped, NonZeroU32::new(3));
        assert_eq!(schedule.offer(20, 1, 2048).skipped, NonZeroU32::new(9));
    }

    #[test]
    fn the_frame_counter_wrapping_keeps_scheduling_forward() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(15, 1, 16);
        assert_eq!(first.frame, 0);
        assert_eq!(first.skipped, None);
        schedule.accept();

        let next = schedule.offer(0, 1, 16);
        assert_eq!(next.frame, 1);
        assert_eq!(next.skipped, None);
    }

    #[test]
    fn phase_lock_aligns_interval_greater_than_one_without_spurious_skips() {
        for p in 0..8 {
            let mut schedule = IsoSchedule::new();
            let first = schedule.offer(0, 8, 2048);
            assert_eq!(first.frame, 8);
            assert_eq!(first.skipped, None);
            schedule.accept();

            // Host actually polls on frame `8 + p`.
            let polled = 8 + p;
            schedule.lock_phase(polled, 8);
            assert_eq!(schedule.phase, Some(p));

            // Next ready runs right after the poll at `now = polled`.
            let second = schedule.offer(polled, 8, 2048);
            assert_eq!(second.frame, 16 + p);
            assert_eq!(second.skipped, None);
            schedule.accept();

            // Keeping up at locked phase reports zero skips.
            let third = schedule.offer(16 + p, 8, 2048);
            assert_eq!(third.frame, 24 + p);
            assert_eq!(third.skipped, None);
            schedule.accept();

            // Falling behind by 3 intervals reports 3 skipped.
            let late = schedule.offer(24 + p + 24, 8, 2048);
            assert_eq!(late.frame, 24 + p + 32);
            assert_eq!(late.skipped, NonZeroU32::new(3));
        }
    }

    #[test]
    fn phase_lock_survives_counter_wrap_and_clears_on_reset() {
        let mut schedule = IsoSchedule::new();
        schedule.lock_phase(2043, 8); // phase = 3
        let first = schedule.offer(2043, 8, 2048);
        assert_eq!(first.frame, 3);
        assert_eq!(first.skipped, None);
        schedule.accept();

        let second = schedule.offer(3, 8, 2048);
        assert_eq!(second.frame, 11);
        assert_eq!(second.skipped, None);

        schedule.reset();
        assert_eq!(schedule.phase, None);
        let fresh = schedule.offer(10, 8, 2048);
        assert_eq!(fresh.frame, 16);
        assert_eq!(fresh.skipped, None);
    }

    #[test]
    fn a_dropped_slot_reports_its_skips_again() {
        let mut schedule = IsoSchedule::new();
        schedule.offer(10, 1, 2048);
        schedule.accept();

        // Three intervals missed; the slot is dropped without a write.
        let dropped = schedule.offer(14, 1, 2048);
        assert_eq!(dropped.frame, 15);
        assert_eq!(dropped.skipped, NonZeroU32::new(3));

        let again = schedule.offer(14, 1, 2048);
        assert_eq!(again.frame, 15);
        assert_eq!(again.skipped, NonZeroU32::new(3));
    }

    #[test]
    fn a_written_interval_is_not_offered_again() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(0, 8, 2048);
        assert_eq!(first.frame, 8);
        schedule.accept();

        // The host collected frame 8 early; the counter still reads 7.
        let next = schedule.offer(7, 8, 2048);
        assert_eq!(next.frame, 16);
        assert_eq!(next.skipped, None);
    }

    #[test]
    fn forward_distance_wraps_with_the_counter() {
        assert_eq!(forward(15, 0, 16), 1);
        assert_eq!(forward(8, 16, 16384), 8);
        assert_eq!(forward(4, 4, 16), 0);
    }
}
