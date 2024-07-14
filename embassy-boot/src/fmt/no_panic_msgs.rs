//! Equivalents of panicking macros that discard their messages
//!
//! Enabled by the `no-panic-msgs` feature.  Panics that include messages waste valuable flash
//! storage, and removing these messages are useful when failure is very unlikely, e.g. when
//! running as part of a bootloader.

macro_rules! assert {
    ($cond:expr $(,$($rest:tt)+)?) => {
        ::core::assert!($cond, "")
    };
}
pub(crate) use assert;

macro_rules! assert_eq {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::assert_eq!($left, $right, "")
    };
}
pub(crate) use assert_eq;

macro_rules! assert_ne {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::assert_ne!($left, $right, "")
    };
}
pub(crate) use assert_ne;

macro_rules! debug_assert {
    ($cond:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert!($cond, "")
    };
}
pub(crate) use debug_assert;

macro_rules! debug_assert_eq {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert_eq!($left, $right, "")
    };
}
pub(crate) use debug_assert_eq;

macro_rules! debug_assert_ne {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert_ne!($left, $right, "")
    };
}
pub(crate) use debug_assert_ne;

macro_rules! unwrap {
    ($arg:expr) => {
        match $crate::fmt::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(_) => {
                ::core::panic!();
            }
        }
    };
    ($arg:expr $(,$($rest:tt)+)?) => {
        match $crate::fmt::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(_) => {
                ::core::panic!();
            }
        }
    };
}
pub(crate) use unwrap;
