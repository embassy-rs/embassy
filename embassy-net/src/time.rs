#![allow(unused)]

use embassy_time::{Duration, Instant};
use xarxa::time::{Duration as XarxaDuration, Instant as XarxaInstant};

pub(crate) fn instant_to_xarxa(instant: Instant) -> XarxaInstant {
    // Saturate: embassy-time instants beyond i64::MAX µs (Instant::MAX included)
    // become XarxaInstant::MAX, i.e. "never", instead of going negative.
    XarxaInstant::from_micros(instant.as_micros().min(i64::MAX as u64) as i64)
}

pub(crate) fn instant_from_xarxa(instant: XarxaInstant) -> Instant {
    Instant::from_micros(instant.total_micros().max(0) as u64)
}

pub(crate) fn duration_to_xarxa(duration: Duration) -> XarxaDuration {
    XarxaDuration::from_micros(duration.as_micros())
}

pub(crate) fn duration_from_xarxa(duration: XarxaDuration) -> Duration {
    Duration::from_micros(duration.total_micros())
}
