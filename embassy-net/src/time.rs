#![allow(unused)]

use embassy_time::{Duration, Instant};
use xarxa::time::{Duration as XarxaDuration, Instant as XarxaInstant};

/// The xarxa instant for `now`. xarxa instants are the low 32 bits of the
/// millisecond count, wrapping around every ~49.7 days.
pub(crate) fn now_to_xarxa(now: Instant) -> XarxaInstant {
    XarxaInstant::from_millis(now.as_millis() as u32)
}

/// Convert an instant, going through its offset from the current time.
///
/// xarxa instants only compare correctly when close to each other, so offsets
/// saturate at `xarxa::time::Duration::MAX` (~12.4 days) in either direction.
pub(crate) fn instant_to_xarxa(instant: Instant) -> XarxaInstant {
    let now = Instant::now();
    let xnow = now_to_xarxa(now);
    if instant >= now {
        xnow + millis_to_xarxa(instant.as_millis() - now.as_millis())
    } else {
        xnow - millis_to_xarxa(now.as_millis() - instant.as_millis())
    }
}

/// Convert an instant, going through its offset from the current time.
pub(crate) fn instant_from_xarxa(instant: XarxaInstant) -> Instant {
    let now = Instant::now();
    let xnow = now_to_xarxa(now);
    match instant.checked_duration_since(xnow) {
        Some(ahead) => now + duration_from_xarxa(ahead),
        None => {
            let behind = duration_from_xarxa(xnow.duration_since(instant));
            now.checked_sub(behind).unwrap_or(Instant::MIN)
        }
    }
}

fn millis_to_xarxa(millis: u64) -> XarxaDuration {
    XarxaDuration::from_millis(millis.min(u32::MAX as u64) as u32)
}

/// Convert a duration, rounding up to the next millisecond and saturating at
/// `xarxa::time::Duration::MAX` (~12.4 days).
pub(crate) fn duration_to_xarxa(duration: Duration) -> XarxaDuration {
    millis_to_xarxa(duration.as_micros().div_ceil(1000))
}

pub(crate) fn duration_from_xarxa(duration: XarxaDuration) -> Duration {
    Duration::from_millis(duration.as_millis() as u64)
}
