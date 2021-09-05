use core::time::Duration;

use crate::subghz::value_error::ValueError;

const fn abs_diff(a: u64, b: u64) -> u64 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

/// Timeout argument.
///
/// This is used by:
/// * [`set_rx`]
/// * [`set_tx`]
/// * [`TcxoMode`]
///
/// Each timeout has 3 bytes, with a resolution of 15.625µs per bit, giving a
/// range of 0s to 262.143984375s.
///
/// [`set_rx`]: crate::subghz::SubGhz::set_rx
/// [`set_tx`]: crate::subghz::SubGhz::set_tx
/// [`TcxoMode`]: crate::subghz::TcxoMode
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timeout {
    bits: u32,
}

impl Timeout {
    const BITS_PER_MILLI: u32 = 64; // 1e-3 / 15.625e-6
    const BITS_PER_SEC: u32 = 64_000; // 1 / 15.625e-6

    /// Disable the timeout (0s timeout).
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// const TIMEOUT: Timeout = Timeout::DISABLED;
    /// assert_eq!(TIMEOUT.as_duration(), Duration::from_secs(0));
    /// ```
    pub const DISABLED: Timeout = Timeout { bits: 0x0 };

    /// Minimum timeout, 15.625µs.
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// const TIMEOUT: Timeout = Timeout::MIN;
    /// assert_eq!(TIMEOUT.into_bits(), 1);
    /// ```
    pub const MIN: Timeout = Timeout { bits: 1 };

    /// Maximum timeout, 262.143984375s.
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// const TIMEOUT: Timeout = Timeout::MAX;
    /// assert_eq!(TIMEOUT.as_duration(), Duration::from_nanos(262_143_984_375));
    /// ```
    pub const MAX: Timeout = Timeout { bits: 0x00FF_FFFF };

    /// Timeout resolution in nanoseconds, 15.625µs.
    pub const RESOLUTION_NANOS: u16 = 15_625;

    /// Timeout resolution, 15.625µs.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(
    ///     Timeout::RESOLUTION.as_nanos(),
    ///     Timeout::RESOLUTION_NANOS as u128
    /// );
    /// ```
    pub const RESOLUTION: Duration = Duration::from_nanos(Self::RESOLUTION_NANOS as u64);

    /// Create a new timeout from a [`Duration`].
    ///
    /// This will return the nearest timeout value possible, or a
    /// [`ValueError`] if the value is out of bounds.
    ///
    /// Use [`from_millis_sat`](Self::from_millis_sat) for runtime timeout
    /// construction.
    /// This is not _that_ useful right now, it is simply future proofing for a
    /// time when `Result::unwrap` is avaliable for `const fn`.
    ///
    /// # Example
    ///
    /// Value within bounds:
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::{Timeout, ValueError};
    ///
    /// const MIN: Duration = Timeout::RESOLUTION;
    /// assert_eq!(Timeout::from_duration(MIN).unwrap(), Timeout::MIN);
    /// ```
    ///
    /// Value too low:
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::{Timeout, ValueError};
    ///
    /// const LOWER_LIMIT_NANOS: u128 = 7813;
    /// const TOO_LOW_NANOS: u128 = LOWER_LIMIT_NANOS - 1;
    /// const TOO_LOW_DURATION: Duration = Duration::from_nanos(TOO_LOW_NANOS as u64);
    /// assert_eq!(
    ///     Timeout::from_duration(TOO_LOW_DURATION),
    ///     Err(ValueError::too_low(TOO_LOW_NANOS, LOWER_LIMIT_NANOS))
    /// );
    /// ```
    ///
    /// Value too high:
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::{Timeout, ValueError};
    ///
    /// const UPPER_LIMIT_NANOS: u128 = Timeout::MAX.as_nanos() as u128 + 7812;
    /// const TOO_HIGH_NANOS: u128 = UPPER_LIMIT_NANOS + 1;
    /// const TOO_HIGH_DURATION: Duration = Duration::from_nanos(TOO_HIGH_NANOS as u64);
    /// assert_eq!(
    ///     Timeout::from_duration(TOO_HIGH_DURATION),
    ///     Err(ValueError::too_high(TOO_HIGH_NANOS, UPPER_LIMIT_NANOS))
    /// );
    /// ```
    pub const fn from_duration(duration: Duration) -> Result<Timeout, ValueError<u128>> {
        // at the time of development many methods in
        // `core::Duration` were not `const fn`, which leads to the hacks
        // you see here.
        let nanos: u128 = duration.as_nanos();
        const UPPER_LIMIT: u128 =
            Timeout::MAX.as_nanos() as u128 + (Timeout::RESOLUTION_NANOS as u128) / 2;
        const LOWER_LIMIT: u128 = (((Timeout::RESOLUTION_NANOS as u128) + 1) / 2) as u128;

        if nanos > UPPER_LIMIT {
            Err(ValueError::too_high(nanos, UPPER_LIMIT))
        } else if nanos < LOWER_LIMIT {
            Err(ValueError::too_low(nanos, LOWER_LIMIT))
        } else {
            // safe to truncate here because of previous bounds check.
            let duration_nanos: u64 = nanos as u64;

            let div_floor: u64 = duration_nanos / (Self::RESOLUTION_NANOS as u64);
            let div_ceil: u64 = 1 + (duration_nanos - 1) / (Self::RESOLUTION_NANOS as u64);

            let timeout_ceil: Timeout = Timeout::from_raw(div_ceil as u32);
            let timeout_floor: Timeout = Timeout::from_raw(div_floor as u32);

            let error_ceil: u64 = abs_diff(timeout_ceil.as_nanos(), duration_nanos);
            let error_floor: u64 = abs_diff(timeout_floor.as_nanos(), duration_nanos);

            if error_ceil < error_floor {
                Ok(timeout_ceil)
            } else {
                Ok(timeout_floor)
            }
        }
    }

    /// Create a new timeout from a [`Duration`].
    ///
    /// This will return the nearest timeout value possible, saturating at the
    /// limits.
    ///
    /// This is an expensive function to call outside of `const` contexts.
    /// Use [`from_millis_sat`](Self::from_millis_sat) for runtime timeout
    /// construction.
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// const DURATION_MAX_NS: u64 = 262_143_984_376;
    ///
    /// assert_eq!(
    ///     Timeout::from_duration_sat(Duration::from_millis(0)),
    ///     Timeout::MIN
    /// );
    /// assert_eq!(
    ///     Timeout::from_duration_sat(Duration::from_nanos(DURATION_MAX_NS)),
    ///     Timeout::MAX
    /// );
    /// assert_eq!(
    ///     Timeout::from_duration_sat(Timeout::RESOLUTION).into_bits(),
    ///     1
    /// );
    /// ```
    pub const fn from_duration_sat(duration: Duration) -> Timeout {
        // at the time of development many methods in
        // `core::Duration` were not `const fn`, which leads to the hacks
        // you see here.
        let nanos: u128 = duration.as_nanos();
        const UPPER_LIMIT: u128 = Timeout::MAX.as_nanos() as u128;

        if nanos > UPPER_LIMIT {
            Timeout::MAX
        } else if nanos < (Timeout::RESOLUTION_NANOS as u128) {
            Timeout::from_raw(1)
        } else {
            // safe to truncate here because of previous bounds check.
            let duration_nanos: u64 = duration.as_nanos() as u64;

            let div_floor: u64 = duration_nanos / (Self::RESOLUTION_NANOS as u64);
            let div_ceil: u64 = 1 + (duration_nanos - 1) / (Self::RESOLUTION_NANOS as u64);

            let timeout_ceil: Timeout = Timeout::from_raw(div_ceil as u32);
            let timeout_floor: Timeout = Timeout::from_raw(div_floor as u32);

            let error_ceil: u64 = abs_diff(timeout_ceil.as_nanos(), duration_nanos);
            let error_floor: u64 = abs_diff(timeout_floor.as_nanos(), duration_nanos);

            if error_ceil < error_floor {
                timeout_ceil
            } else {
                timeout_floor
            }
        }
    }

    /// Create a new timeout from a milliseconds value.
    ///
    /// This will round towards zero and saturate at the limits.
    ///
    /// This is the preferred method to call when you need to generate a
    /// timeout value at runtime.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::from_millis_sat(0), Timeout::MIN);
    /// assert_eq!(Timeout::from_millis_sat(262_144), Timeout::MAX);
    /// assert_eq!(Timeout::from_millis_sat(1).into_bits(), 64);
    /// ```
    pub const fn from_millis_sat(millis: u32) -> Timeout {
        if millis == 0 {
            Timeout::MIN
        } else if millis >= 262_144 {
            Timeout::MAX
        } else {
            Timeout::from_raw(millis * Self::BITS_PER_MILLI)
        }
    }

    /// Create a timeout from raw bits, where each bit has the resolution of
    /// [`Timeout::RESOLUTION`].
    ///
    /// **Note:** Only the first 24 bits of the `u32` are used, the `bits`
    /// argument will be masked.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::from_raw(u32::MAX), Timeout::MAX);
    /// assert_eq!(Timeout::from_raw(0x00_FF_FF_FF), Timeout::MAX);
    /// assert_eq!(Timeout::from_raw(1).as_duration(), Timeout::RESOLUTION);
    /// assert_eq!(Timeout::from_raw(0), Timeout::DISABLED);
    /// ```
    pub const fn from_raw(bits: u32) -> Timeout {
        Timeout {
            bits: bits & 0x00FF_FFFF,
        }
    }

    /// Get the timeout as nanoseconds.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::MAX.as_nanos(), 262_143_984_375);
    /// assert_eq!(Timeout::DISABLED.as_nanos(), 0);
    /// assert_eq!(Timeout::from_raw(1).as_nanos(), 15_625);
    /// assert_eq!(Timeout::from_raw(64_000).as_nanos(), 1_000_000_000);
    /// ```
    pub const fn as_nanos(&self) -> u64 {
        (self.bits as u64) * (Timeout::RESOLUTION_NANOS as u64)
    }

    /// Get the timeout as microseconds, rounding towards zero.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::MAX.as_micros(), 262_143_984);
    /// assert_eq!(Timeout::DISABLED.as_micros(), 0);
    /// assert_eq!(Timeout::from_raw(1).as_micros(), 15);
    /// assert_eq!(Timeout::from_raw(64_000).as_micros(), 1_000_000);
    /// ```
    pub const fn as_micros(&self) -> u32 {
        (self.as_nanos() / 1_000) as u32
    }

    /// Get the timeout as milliseconds, rounding towards zero.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::MAX.as_millis(), 262_143);
    /// assert_eq!(Timeout::DISABLED.as_millis(), 0);
    /// assert_eq!(Timeout::from_raw(1).as_millis(), 0);
    /// assert_eq!(Timeout::from_raw(64_000).as_millis(), 1_000);
    /// ```
    pub const fn as_millis(&self) -> u32 {
        self.into_bits() / Self::BITS_PER_MILLI
    }

    /// Get the timeout as seconds, rounding towards zero.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::MAX.as_secs(), 262);
    /// assert_eq!(Timeout::DISABLED.as_secs(), 0);
    /// assert_eq!(Timeout::from_raw(1).as_secs(), 0);
    /// assert_eq!(Timeout::from_raw(64_000).as_secs(), 1);
    /// ```
    pub const fn as_secs(&self) -> u16 {
        (self.into_bits() / Self::BITS_PER_SEC) as u16
    }

    /// Get the timeout as a [`Duration`].
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(
    ///     Timeout::MAX.as_duration(),
    ///     Duration::from_nanos(262_143_984_375)
    /// );
    /// assert_eq!(Timeout::DISABLED.as_duration(), Duration::from_nanos(0));
    /// assert_eq!(Timeout::from_raw(1).as_duration(), Timeout::RESOLUTION);
    /// ```
    pub const fn as_duration(&self) -> Duration {
        Duration::from_nanos((self.bits as u64) * (Timeout::RESOLUTION_NANOS as u64))
    }

    /// Get the bit value for the timeout.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::from_raw(u32::MAX).into_bits(), 0x00FF_FFFF);
    /// assert_eq!(Timeout::from_raw(1).into_bits(), 1);
    /// ```
    pub const fn into_bits(self) -> u32 {
        self.bits
    }

    /// Get the byte value for the timeout.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wl_hal::subghz::Timeout;
    ///
    /// assert_eq!(Timeout::from_raw(u32::MAX).as_bytes(), [0xFF, 0xFF, 0xFF]);
    /// assert_eq!(Timeout::from_raw(1).as_bytes(), [0, 0, 1]);
    /// ```
    pub const fn as_bytes(self) -> [u8; 3] {
        [
            ((self.bits >> 16) & 0xFF) as u8,
            ((self.bits >> 8) & 0xFF) as u8,
            (self.bits & 0xFF) as u8,
        ]
    }
}

impl From<Timeout> for Duration {
    fn from(to: Timeout) -> Self {
        to.as_duration()
    }
}

impl From<Timeout> for [u8; 3] {
    fn from(to: Timeout) -> Self {
        to.as_bytes()
    }
}

impl From<Timeout> for embassy::time::Duration {
    fn from(to: Timeout) -> Self {
        embassy::time::Duration::from_micros(to.as_micros().into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Timeout, ValueError};
    use core::time::Duration;

    #[test]
    fn saturate() {
        assert_eq!(
            Timeout::from_duration_sat(Duration::from_secs(u64::MAX)),
            Timeout::MAX
        );
    }

    #[test]
    fn rounding() {
        const NANO1: Duration = Duration::from_nanos(1);
        let res_sub_1_ns: Duration = Timeout::RESOLUTION - NANO1;
        let res_add_1_ns: Duration = Timeout::RESOLUTION + NANO1;
        assert_eq!(Timeout::from_duration_sat(res_sub_1_ns).into_bits(), 1);
        assert_eq!(Timeout::from_duration_sat(res_add_1_ns).into_bits(), 1);
    }

    #[test]
    fn lower_limit() {
        let low: Duration = (Timeout::RESOLUTION + Duration::from_nanos(1)) / 2;
        assert_eq!(Timeout::from_duration(low), Ok(Timeout::from_raw(1)));

        let too_low: Duration = low - Duration::from_nanos(1);
        assert_eq!(
            Timeout::from_duration(too_low),
            Err(ValueError::too_low(too_low.as_nanos(), low.as_nanos()))
        );
    }

    #[test]
    fn upper_limit() {
        let high: Duration = Timeout::MAX.as_duration() + Timeout::RESOLUTION / 2;
        assert_eq!(
            Timeout::from_duration(high),
            Ok(Timeout::from_raw(0xFFFFFF))
        );

        let too_high: Duration = high + Duration::from_nanos(1);
        assert_eq!(
            Timeout::from_duration(too_high),
            Err(ValueError::too_high(too_high.as_nanos(), high.as_nanos()))
        );
    }
}
