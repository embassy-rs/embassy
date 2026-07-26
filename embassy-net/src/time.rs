#![allow(unused)]

use embassy_time::{Duration, Instant};
use sporktcp::time::{Duration as SmolDuration, Instant as SmolInstant};

pub(crate) fn instant_to_sporktcp(instant: Instant) -> SmolInstant {
    SmolInstant::from_micros(instant.as_micros() as i64)
}

pub(crate) fn instant_from_sporktcp(instant: SmolInstant) -> Instant {
    Instant::from_micros(instant.total_micros() as u64)
}

pub(crate) fn duration_to_sporktcp(duration: Duration) -> SmolDuration {
    SmolDuration::from_micros(duration.as_micros())
}

pub(crate) fn duration_from_sporktcp(duration: SmolDuration) -> Duration {
    Duration::from_micros(duration.total_micros())
}
