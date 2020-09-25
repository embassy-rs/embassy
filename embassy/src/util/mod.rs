#![macro_use]

mod macros;

mod signal;
pub use signal::*;
mod portal;
pub use portal::*;
mod waker_store;
pub use waker_store::*;
mod drop_bomb;
pub use drop_bomb::*;

use defmt::{debug, error, info, intern, trace, warn};

pub trait Dewrap<T> {
    /// dewrap = defmt unwrap
    fn dewrap(self) -> T;

    /// dexpect = defmt expect
    fn dexpect<M: defmt::Format>(self, msg: M) -> T;

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self;
}

impl<T> Dewrap<T> for Option<T> {
    fn dewrap(self) -> T {
        match self {
            Some(t) => t,
            None => depanic!("unwrap failed: enum is none"),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Some(t) => t,
            None => depanic!("unexpected None: {:?}", msg),
        }
    }

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self {
        if self.is_none() {
            warn!("{:?} is none", msg);
        }
        self
    }
}

impl<T, E: defmt::Format> Dewrap<T> for Result<T, E> {
    fn dewrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("unwrap failed: {:?}", e),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("unexpected error: {:?}: {:?}", msg, e),
        }
    }

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self {
        if let Err(e) = &self {
            warn!("{:?} err: {:?}", msg, e);
        }
        self
    }
}
