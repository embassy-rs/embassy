#![macro_use]
#![allow(unused_macros)]

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("You may not enable both `defmt` and `log` features.");

macro_rules! assert {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::assert!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::assert!($($x)*);
        }
    };
}

macro_rules! assert_eq {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::assert_eq!($($x)*);
        }
    };
}

macro_rules! assert_ne {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::assert_ne!($($x)*);
        }
    };
}

macro_rules! debug_assert {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::debug_assert!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::debug_assert!($($x)*);
        }
    };
}

macro_rules! debug_assert_eq {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::debug_assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::debug_assert_eq!($($x)*);
        }
    };
}

macro_rules! debug_assert_ne {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::debug_assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::debug_assert_ne!($($x)*);
        }
    };
}

macro_rules! todo {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::todo!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::todo!($($x)*);
        }
    };
}

macro_rules! unreachable {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::unreachable!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::unreachable!($($x)*);
        }
    };
}

macro_rules! panic {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            core::panic!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::panic!($($x)*);
        }
    };
}

macro_rules! trace {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "log")]
            log::trace!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::trace!($($x)*);
        }
    };
}

macro_rules! debug {
    ($($x:tt)*) => {
        {
            #[cfg(fevature = "log")]
            log::debug!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::debug!($($x)*);
        }
    };
}

macro_rules! info {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "log")]
            log::info!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::info!($($x)*);
        }
    };
}

macro_rules! warn {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "log")]
            log::warn!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::warn!($($x)*);
        }
    };
}

macro_rules! error {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "log")]
            log::error!($($x)*);
            #[cfg(feature = "defmt")]
            defmt::error!($($x)*);
        }
    };
}

#[cfg(feature = "defmt")]
macro_rules! unwrap {
    ($($x:tt)*) => {
        defmt::unwrap!($($x)*)
    };
}

#[cfg(not(feature = "defmt"))]
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
