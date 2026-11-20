//! Isochronous IN traits and service-interval scheduling helpers.

use core::num::NonZeroU32;

use crate::{Endpoint, EndpointDisabledError};

/// Why [`IsoInSlot::write`] did not queue the transfer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IsoWriteError {
    /// The service interval had already started before [`IsoInSlot::write`] was called,
    /// so the service interval was skipped.
    ///
    /// This will *not* be increment the skipped service interval counter.
    TooLate,
    /// `data` is larger than larger than the maximum number of
    /// bytes the endpoint is configured to transfer in a service interval.
    ///
    /// For Full Speed endpoints, this means at most
    /// [`EndpointInfo::max_packet_size`](crate::EndpointInfo::max_packet_size)
    /// bytes, but High Speed and Super Speed endpoints can be configured to allow
    /// more than one packet per service interval.
    ///
    /// This also means the service interval will be skipped
    /// *without* incrementing the skipped service interval counter.
    BufferOverflow,
}

impl core::fmt::Display for IsoWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooLate => write!(f, "Isochronous transfer submitted too late"),
            Self::BufferOverflow => write!(f, "Buffer overflow"),
        }
    }
}

impl core::error::Error for IsoWriteError {}

/// A reserved isochronous IN slot in the queue, corresponding to a specific service interval
pub trait IsoInSlot {
    /// Queue one transfer for this slot without waiting.
    ///
    /// The length of `data` may be zero and must be no larger than the maximum number of
    /// bytes the endpoint is configured to transfer in a service interval.
    /// For Full Speed endpoints, this means at most
    /// [`EndpointInfo::max_packet_size`](crate::EndpointInfo::max_packet_size)
    /// bytes, but High Speed and Super Speed endpoints can be configured to allow
    /// more than one packet per service interval.
    ///
    /// A transfer the host does not poll for in its service interval is
    /// silently dropped when the interval ends and is never sent in a later interval.
    ///
    /// See [`IsoEndpointIn::ready`] for more details.
    fn write(self, data: &[u8]) -> Result<(), IsoWriteError>;
}

/// Isochronous IN endpoint trait.
///
/// Drivers implement this only when they can schedule transfers by service interval and drop
/// uncollected transfers instead of sending them late.
pub trait IsoEndpointIn: Endpoint {
    /// A reserved slot borrowing this endpoint.
    type Slot<'a>: IsoInSlot
    where
        Self: 'a;

    /// How many transfers may be committed ahead of the one the host collects next.
    ///
    /// * `1`: one transfer can be committed. [`ready`](Self::ready) waits until the previous transfer
    ///   has been sent or dropped before offering another slot.
    /// * `n`: [`ready`](Self::ready) may offer a slot while `n` transfers are still queued.
    ///
    /// This is what the driver actually provides. [`EndpointConfig::buffers`](crate::EndpointConfig::buffers)
    /// is only a hint.
    fn queue_depth(&self) -> u8;

    /// Wait until the next transfer can be queued, returning
    ///
    /// * the number of skipped service intervals (if any)
    ///   since the service interval of the last [`IsoInSlot`], and
    ///
    /// * an [`IsoInSlot`] for the next available service interval.
    ///
    /// Service intervals are skipped if a transfer isn't submitted in time.
    ///
    /// Skipped service intervals are handled in an implementation defined way:
    /// the device may either respond to the host's request with an empty transfer,
    /// or not respond to the host at all.
    ///
    /// The first successful call to [`ready`](Self::ready) after the endpoint is enabled
    /// will report no skipped service intervals.
    ///
    /// If [`ready`](Self::ready) is called too late for the next sequential service interval,
    /// it will return the number of service intervals that were skipped.
    /// The skipped service interval counter is now reset.
    ///
    /// If [`IsoInSlot::write`] is called too late, it will return [`IsoWriteError::TooLate`].
    /// This means the service interval has been skipped,
    /// but it will *not* increment the skipped service interval counter.
    ///
    /// [`IsoWriteError::BufferOverflow`] also means the service interval will be skipped
    /// without incrementing the skipped service interval counter.
    ///
    /// When the endpoint is disabled, any transfers in the queue will be silently dropped.
    /// If there is an active [`IsoInSlot`], it's transfer will be dropped even if
    /// [`IsoInSlot::write`] returns [`Ok`]
    ///
    /// # Cancel safety
    ///
    /// Dropping this future will not reset the skipped service interval count
    /// or reserve a service interval.
    ///
    /// Dropping the slot before calling [`IsoInSlot::write`] means the slot in the queue
    /// and the corresponding service interval are no longer reserved, and can be re-issued
    /// by [`ready`](Self::ready) (if it is called again in time).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embassy_usb_driver::*;
    /// async fn example(mut ep: impl IsoEndpointIn, mut get_packet: impl FnMut() -> [u8; 8]) {
    ///     assert!(ep.info().max_packet_size >= 8);
    ///     ep.wait_enabled().await;
    ///     loop {
    ///         let Ok((skipped, slot)) = ep.ready().await else {
    ///             break;
    ///         };
    ///         if let Some(skipped) = skipped {
    ///             // throw away packets to avoid desynchronization
    ///             for _ in 0..skipped.get() {
    ///                 let _ = get_packet();
    ///             }
    ///         }
    ///         match slot.write(&get_packet()) {
    ///             // In this case, we don't care if we may have sent an
    ///             // empty packet instead of the real packet
    ///             Ok(()) | Err(IsoWriteError::TooLate) => {}
    ///
    ///             // We already checked that our max transfer size is big enough
    ///             Err(IsoWriteError::BufferOverflow) => unreachable!(),
    ///
    ///             // The error is non-exhaustive, so we need to handle future errors
    ///             Err(err) => panic!("{}", err),
    ///         }
    ///     }
    /// }
    /// ```
    async fn ready(&mut self) -> Result<(Option<NonZeroU32>, Self::Slot<'_>), EndpointDisabledError>;
}

async fn example(mut ep: impl IsoEndpointIn, mut packet_getter: impl FnMut() -> [u8; 8]) {
    assert!(ep.info().max_packet_size >= 8);
    ep.wait_enabled().await;
    loop {
        let Ok((skipped, slot)) = ep.ready().await else {
            break;
        };
        if let Some(skipped) = skipped {
            // throw away packets to avoid desynchronization
            for _ in 0..skipped.get() {
                let _ = packet_getter();
            }
        }
        match slot.write(&packet_getter()) {
            Ok(()) | Err(IsoWriteError::TooLate) => {} // in this case, we don't care if we may have sent an empty packet instead of the real packet
            Err(IsoWriteError::BufferOverflow) => unreachable!(),
        }
    }
}

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
    last_accepted: bool,
}

impl IsoSchedule {
    /// A schedule that has not offered a slot yet.
    pub const fn new() -> Self {
        Self {
            last_now: None,
            epoch: 0,
            phase: None,
            last_frame: None,
            last_accepted: false,
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
        self.last_accepted = false;
    }

    /// The locked polling phase in `0..interval`, if known.
    pub const fn phase(&self) -> Option<u32> {
        self.phase
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
        // Smallest frame strictly after `extended` with `frame % interval == phase`.
        let min_frame = extended + 1;
        let rem = min_frame % interval;
        let add = if phase >= rem {
            phase - rem
        } else {
            interval - (rem - phase)
        };
        let frame = min_frame + add;

        let skipped = NonZeroU32::new(self.skipped_before(frame, interval));

        if self.last_frame != Some(frame) {
            self.last_frame = Some(frame);
            self.last_accepted = false;
        }

        IsoOffer::new((frame % span) as u32, skipped)
    }

    /// Mark the most recently offered slot as queued.
    ///
    /// Later offers will not count that interval as empty.
    pub fn accept(&mut self) {
        if self.last_frame.is_some() {
            self.last_accepted = true;
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

    fn skipped_before(&self, frame: u64, interval: u64) -> u32 {
        match self.last_frame {
            None => 0,
            Some(prev) => {
                let delta = frame.saturating_sub(prev);
                let gap = (delta / interval) as u32;
                if gap == 0 {
                    0
                } else if self.last_accepted {
                    gap - 1
                } else {
                    gap
                }
            }
        }
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
    fn an_offered_slot_that_was_not_queued_counts_as_skipped_once_it_starts() {
        let mut schedule = IsoSchedule::new();
        let first = schedule.offer(0, 8, 16384);
        assert_eq!(first.frame, 8);

        let next = schedule.offer(8, 8, 16384);
        assert_eq!(next.frame, 16);
        assert_eq!(next.skipped, NonZeroU32::new(1));
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
            assert_eq!(schedule.phase(), Some(p));

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
        assert_eq!(schedule.phase(), None);
        let fresh = schedule.offer(10, 8, 2048);
        assert_eq!(fresh.frame, 16);
        assert_eq!(fresh.skipped, None);
    }

    #[test]
    fn forward_distance_wraps_with_the_counter() {
        assert_eq!(forward(15, 0, 16), 1);
        assert_eq!(forward(8, 16, 16384), 8);
        assert_eq!(forward(4, 4, 16), 0);
    }
}
