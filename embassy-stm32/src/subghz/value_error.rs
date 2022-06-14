/// Error for a value that is out-of-bounds.
///
/// Used by [`Timeout::from_duration`].
///
/// [`Timeout::from_duration`]: super::Timeout::from_duration
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ValueError<T> {
    value: T,
    limit: T,
    over: bool,
}

impl<T> ValueError<T> {
    /// Create a new `ValueError` for a value that exceeded an upper bound.
    ///
    /// Unfortunately panic is not available in `const fn`, so there are no
    /// guarantees on the value being greater than the limit.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_high(101u8, 100u8);
    /// assert!(ERROR.over());
    /// assert!(!ERROR.under());
    /// ```
    pub const fn too_high(value: T, limit: T) -> ValueError<T> {
        ValueError {
            value,
            limit,
            over: true,
        }
    }

    /// Create a new `ValueError` for a value that exceeded a lower bound.
    ///
    /// Unfortunately panic is not available in `const fn`, so there are no
    /// guarantees on the value being less than the limit.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_low(200u8, 201u8);
    /// assert!(ERROR.under());
    /// assert!(!ERROR.over());
    /// ```
    pub const fn too_low(value: T, limit: T) -> ValueError<T> {
        ValueError {
            value,
            limit,
            over: false,
        }
    }

    /// Get the value that caused the error.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_high(101u8, 100u8);
    /// assert_eq!(ERROR.value(), &101u8);
    /// ```
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Get the limit for the value.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_high(101u8, 100u8);
    /// assert_eq!(ERROR.limit(), &100u8);
    /// ```
    pub const fn limit(&self) -> &T {
        &self.limit
    }

    /// Returns `true` if the value was over the limit.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_high(101u8, 100u8);
    /// assert!(ERROR.over());
    /// assert!(!ERROR.under());
    /// ```
    pub const fn over(&self) -> bool {
        self.over
    }

    /// Returns `true` if the value was under the limit.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::ValueError;
    ///
    /// const ERROR: ValueError<u8> = ValueError::too_low(200u8, 201u8);
    /// assert!(ERROR.under());
    /// assert!(!ERROR.over());
    /// ```
    pub const fn under(&self) -> bool {
        !self.over
    }
}

impl<T> core::fmt::Display for ValueError<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.over {
            write!(f, "Value is too high {} > {}", self.value, self.limit)
        } else {
            write!(f, "Value is too low {} < {}", self.value, self.limit)
        }
    }
}
