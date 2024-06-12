#![no_std]

use core::fmt::{Debug, Display, LowerHex};

#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("You may not enable both `defmt` and `log` features.");

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(not(any(feature = "log", feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! unwrap {
    ($arg:expr) => {
        match crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {:?}", ::core::stringify!($arg), e);
            }
        }
    };
    ($arg:expr, $($msg:expr),+ $(,)? ) => {
        match crate::Try::into_result($arg) {
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
