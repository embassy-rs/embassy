#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use core::fmt::{Debug, Display, LowerHex};

#[cfg(feature = "no-panic-msgs")]
#[macro_use]
#[doc(hidden)]
pub mod no_panic_msgs;

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("You may not enable both `defmt` and `log` features.");

#[cfg(all(feature = "no-panic-msgs", feature = "log"))]
compile_error!("You may not enable both `no-panic-msgs` and `log` features.");

#[cfg(all(feature = "no-panic-msgs", feature = "defmt"))]
compile_error!("You may not enable both `no-panic-msgs` and `defmt` features.");

/// Like core::assert!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! assert {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::assert!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::assert_no_msg!($($x)*);
        }
    };
}

/// Like core::assert_eq!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! assert_eq {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert_eq!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::assert_eq_no_msg!($($x)*);
        }
    };
}

/// Like core::assert_ne!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! assert_ne {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert_ne!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::assert_ne_no_msg!($($x)*);
        }
    };
}

/// Like core::debug_assert!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! debug_assert {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::debug_assert!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::debug_assert_no_msg!($($x)*);
        }
    };
}

/// Like core::debug_assert_eq!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! debug_assert_eq {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::debug_assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert_eq!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::debug_assert_eq_no_msg!($($x)*);
        }
    };
}

/// Like core::debug_assert_ne!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! debug_assert_ne {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::debug_assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert_ne!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            $crate::no_panic_msgs::debug_assert_ne_no_msg!($($x)*);
        }
    };
}

/// Like core::todo!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! todo {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::todo!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::todo!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            ::core::todo!("");
        }
    };
}

/// Like core::unreachable!(), but might re-direct to a different macro depending on crate features.
#[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unreachable {
    ($($x:tt)*) => {
        ::core::unreachable!($($x)*)
    };
}

/// Like core::unreachable!(), but might re-direct to a different macro depending on crate features.
#[cfg(feature = "defmt")]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unreachable {
    ($($x:tt)*) => {
        ::defmt::unreachable!($($x)*)
    };
}

/// Like core::unreachable!(), but might re-direct to a different macro depending on crate features.
#[cfg(feature = "no-panic-msgs")]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unreachable {
    ($($x:tt)*) => {
        ::core::unreachable!("")
    };
}

/// Like core::panic!(), but might re-direct to a different macro depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! panic {
    ($($x:tt)*) => {
        {
            #[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
            ::core::panic!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::panic!($($x)*);
            #[cfg(feature = "no-panic-msgs")]
            ::core::panic!();
        }
    };
}

/// Generic `trace` logging macro that might use `defmt` or `log` depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "log")]
            ::log::trace!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::trace!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature = "defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

/// Generic `debug` logging macro that might use `defmt` or `log` depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "log")]
            ::log::debug!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature = "defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

/// Generic `info` logging macro that might use `defmt` or `log` depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "log")]
            ::log::info!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::info!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature = "defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

/// Generic `warn` logging macro that might use `defmt` or `log` depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "log")]
            ::log::warn!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::warn!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature = "defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

/// Generic `error` logging macro that might use `defmt` or `log` depending on crate features.
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "log")]
            ::log::error!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::error!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature = "defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

/// Like core::unwrap!(), but might re-direct to a different macro depending on crate features.
#[cfg(feature = "defmt")]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unwrap {
    ($($x:tt)*) => {
        ::defmt::unwrap!($($x)*)
    };
}

/// Like core::unwrap!(), but might re-direct to a different macro depending on crate features.
#[cfg(feature = "no-panic-msgs")]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unwrap {
    ($($x:tt)*) => {
        $crate::no_panic_msgs::unwrap_no_msg!($($x)*)
    };
}

/// Like core::unwrap!(), but might re-direct to a different macro depending on crate features.
#[cfg(not(any(feature = "defmt", feature = "no-panic-msgs")))]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! unwrap {
    ($arg:expr) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {:?}", ::core::stringify!($arg), e);
            }
        }
    };
    ($arg:expr, $($msg:expr),+ $(,)? ) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {}: {:?}", ::core::stringify!($arg), ::core::format_args!($($msg,)*), e);
            }
        }
    }
}

/// The error that is produced when unwrapping an `Optional::None`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NoneError;

/// Utility trait used by `unwrap!()`
pub trait Try {
    /// The success value for this `Try`
    type Ok;

    /// The error value for this `Try`
    type Error;

    /// Converts this `Try` into an equivalent `Result`
    fn into_result(self) -> Result<Self::Ok, Self::Error>;
}

impl<T> Try for Option<T> {
    type Ok = T;
    type Error = NoneError;

    #[inline]
    fn into_result(self) -> Result<T, NoneError> {
        self.ok_or(NoneError)
    }
}

impl<T, E> Try for Result<T, E> {
    type Ok = T;
    type Error = E;

    #[inline]
    fn into_result(self) -> Self {
        self
    }
}

/// A convenient newtype wrapper for byte arrays.
///
/// Ensures that bytes are formatted in a consistent way, for example making sure that their hex
/// representation is consistent.
pub struct Bytes<'a>(pub &'a [u8]);

impl<'a> Debug for Bytes<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#02x?}", self.0)
    }
}

impl<'a> Display for Bytes<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#02x?}", self.0)
    }
}

impl<'a> LowerHex for Bytes<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#02x?}", self.0)
    }
}

#[cfg(feature = "defmt")]
impl<'a> defmt::Format for Bytes<'a> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:02x}", self.0)
    }
}
