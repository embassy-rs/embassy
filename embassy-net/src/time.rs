#![allow(unused)]

use embassy_time::{Duration, Instant};
use xarxa::time::{Duration as XarxaDuration, Instant as XarxaInstant};

pub(crate) fn instant_to_xarxa(instant: Instant) -> XarxaInstant {
    XarxaInstant::from_micros(instant.as_micros() as i64)
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
