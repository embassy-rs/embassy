//! Pure arithmetic for the optional stopped-OSTIMER bridge.

/// Return the new monotonic offset and the observed stopped interval.
pub(crate) fn reconcile(
    before: u64,
    after: u64,
    offset: u64,
    elapsed_us: u64,
    uncertainty_us: u64,
) -> Option<(u64, u64)> {
    // The driver's existing 42-bit epoch tracking has already extended both
    // observations. A backwards value or impossible progress is a failure.
    let progressed = after.checked_sub(before)?;
    if progressed > elapsed_us.saturating_add(uncertainty_us) {
        return None;
    }
    let missing = elapsed_us.saturating_sub(progressed);
    let missing = if missing > uncertainty_us { missing } else { 0 };
    let offset = offset.checked_add(missing)?;
    after.checked_add(offset)?;
    Some((offset, missing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shallow_and_skewed_samples_do_not_advance_the_epoch() {
        assert_eq!(reconcile(100, 1_100, 7, 1_000, 123), Some((7, 0)));
        assert_eq!(reconcile(100, 1_100, 7, 1_123, 123), Some((7, 0)));
        assert_eq!(reconcile(100, 1_100, 7, 999, 123), Some((7, 0)));
    }

    #[test]
    fn stopped_time_is_added_once_and_active_recovery_is_subtracted() {
        assert_eq!(reconcile(100, 1_100, 7, 11_000, 123), Some((10_007, 10_000)));
        assert_eq!(reconcile(1_100, 2_100, 10_007, 1_000, 123), Some((10_007, 0)));
    }

    #[test]
    fn expired_and_future_deadlines_share_the_same_epoch() -> Result<(), &'static str> {
        let (offset, _) = reconcile(1_000, 1_500, 100, 5_000, 123).ok_or("valid observation must reconcile")?;
        let now = 1_500 + offset;
        assert_eq!(now, 6_100);
        for deadline in [1_101, 2_000, 6_100] {
            assert!(deadline <= now);
        }
        let future = 9_000u64;
        let raw_compare = future.saturating_sub(offset);
        assert_eq!(raw_compare - 1_500, future - now);
        Ok(())
    }

    #[test]
    fn extended_rollover_and_invalid_observations_are_explicit() {
        assert_eq!(reconcile((1 << 42) - 10, (1 << 42) + 10, 0, 20, 123), Some((0, 0)));
        assert_eq!(reconcile(0, 1, u64::MAX - 2, 1_000, 123), None);
        assert_eq!(reconcile(0, 10, u64::MAX - 2, 10, 123), None);
        assert_eq!(reconcile(1_000, 10, 0, 1_000, 123), None);
        assert_eq!(reconcile(0, 5_000, 0, 1_000, 123), None);
    }
}
