//! Equivalents of panicking macros that discard their messages
//!
//! Enabled by the `no-panic-msgs` feature.  Panics that include messages waste valuable flash
//! storage, and removing these messages are useful when failure is very unlikely, e.g. when
//! running as part of a bootloader.

#[macro_export]
macro_rules! assert_no_msg {
    ($cond:expr $(,$($rest:tt)+)?) => {
        ::core::assert!($cond, "")
    };
}
pub use assert_no_msg;

#[macro_export]
macro_rules! assert_eq_no_msg {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::assert_eq!($left, $right, "")
    };
}
pub use assert_eq_no_msg;

#[macro_export]
macro_rules! assert_ne_no_msg {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::assert_ne_no_msg!($left, $right, "")
    };
}
pub use assert_ne_no_msg;

#[macro_export]
macro_rules! debug_assert_no_msg {
    ($cond:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert!($cond, "")
    };
}
pub use debug_assert_no_msg;

#[macro_export]
macro_rules! debug_assert_eq_no_msg {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert_eq!($left, $right, "")
    };
}
pub use debug_assert_eq_no_msg;

#[macro_export]
macro_rules! debug_assert_ne_no_msg {
    ($left:expr, $right:expr $(,$($rest:tt)+)?) => {
        ::core::debug_assert_ne!($left, $right, "")
    };
}
pub use debug_assert_ne_no_msg;

#[macro_export]
macro_rules! unwrap_no_msg {
    ($arg:expr) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(_) => {
                ::core::panic!();
            }
        }
    };
    ($arg:expr $(,$($rest:tt)+)?) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(_) => {
                ::core::panic!();
            }
        }
    };
}
pub use unwrap_no_msg;
