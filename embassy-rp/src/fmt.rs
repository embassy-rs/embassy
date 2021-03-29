#![macro_use]

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("You may not enable both `defmt` and `log` features.");

pub use fmt::*;

#[cfg(feature = "defmt")]
mod fmt {
    pub use defmt::{
        assert, assert_eq, assert_ne, debug, debug_assert, debug_assert_eq, debug_assert_ne, error,
        info, panic, todo, trace, unreachable, unwrap, warn,
    };
}

#[cfg(feature = "log")]
mod fmt {
    pub use core::{
        assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne, panic, todo,
        unreachable,
    };
    pub use log::{debug, error, info, trace, warn};
}

#[cfg(not(any(feature = "defmt", feature = "log")))]
mod fmt {
    #![macro_use]

    pub use core::{
        assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne, panic, todo,
        unreachable,
    };

    #[macro_export]
    macro_rules! trace {
        ($($msg:expr),+ $(,)?) => {
            ()
        };
    }

    #[macro_export]
    macro_rules! debug {
        ($($msg:expr),+ $(,)?) => {
            ()
        };
    }

    #[macro_export]
    macro_rules! info {
        ($($msg:expr),+ $(,)?) => {
            ()
        };
    }

    #[macro_export]
    macro_rules! warn {
        ($($msg:expr),+ $(,)?) => {
            ()
        };
    }

    #[macro_export]
    macro_rules! error {
        ($($msg:expr),+ $(,)?) => {
            ()
        };
    }
}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! unwrap {
    ($arg:expr) => {
        match $crate::fmt::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {:?}", ::core::stringify!($arg), e);
            }
        }
    };
    ($arg:expr, $($msg:expr),+ $(,)? ) => {
        match $crate::fmt::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {}: {:?}", ::core::stringify!($arg), ::core::format_args!($($msg,)*), e);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NoneError;

pub trait Try {
    type Ok;
    type Error;
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
