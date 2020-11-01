#![no_std]

pub mod export {
    #[cfg(feature = "defmt")]
    pub use defmt;
    #[cfg(feature = "log")]
    pub use log;
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! log {
    (trace, $($arg:expr),*) => { $crate::export::log::trace!($($arg),*); };
    (debug, $($arg:expr),*) => { $crate::export::log::debug!($($arg),*); };
    (info, $($arg:expr),*) => { $crate::export::log::info!($($arg),*); };
    (warn, $($arg:expr),*) => { $crate::export::log::warn!($($arg),*); };
    (error, $($arg:expr),*) => { $crate::export::log::error!($($arg),*); };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! log {
    (trace, $($arg:expr),*) => { $crate::export::defmt::trace!($($arg),*); };
    (debug, $($arg:expr),*) => { $crate::export::defmt::debug!($($arg),*); };
    (info, $($arg:expr),*) => { $crate::export::defmt::info!($($arg),*); };
    (warn, $($arg:expr),*) => { $crate::export::defmt::warn!($($arg),*); };
    (error, $($arg:expr),*) => { $crate::export::defmt::error!($($arg),*); };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
macro_rules! log {
    ($level:ident, $($arg:expr),*) => {{}};
}

#[macro_export]
macro_rules! trace {
    ($($arg:expr),*) => (log!(trace, $($arg),*));
}

#[macro_export]
macro_rules! debug {
    ($($arg:expr),*) => ($crate::log!(debug, $($arg),*));
}

#[macro_export]
macro_rules! info {
    ($($arg:expr),*) => ($crate::log!(info, $($arg),*));
}

#[macro_export]
macro_rules! warn {
    ($($arg:expr),*) => ($crate::log!(warn, $($arg),*));
}

#[macro_export]
macro_rules! error {
    ($($arg:expr),*) => ($crate::log!(error, $($arg),*));
}

#[macro_export]
macro_rules! expect {
    ($arg:expr, $msg:expr) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                $crate::panic!("{:?}: {:?}", $crate::intern!($msg), e);
            }
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! intern {
    ($arg:expr) => {
        $crate::export::defmt::intern!($arg)
    };
}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! intern {
    ($arg:expr) => {
        $arg
    };
}

#[macro_export]
macro_rules! unwrap {
    ($arg:expr) => {
        expect!($arg, "Unwrap failed")
    };
}

#[macro_export]
macro_rules! panic {
    () => {
        $crate::panic!("panic")
    };
    ($($arg:expr),*) => {{
        $crate::log!(error, $($arg),*);
        ::core::panic!()
    }};
}

#[macro_export]
macro_rules! assert {
    ($cond:expr) => {
        $crate::assert!($cond, "assertion failed");
    };
    ($cond:expr, $($arg:expr),*) => {
        {
            if !$cond {
                $crate::panic!($($arg),*);
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
