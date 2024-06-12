#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[allow(unused)]
#[macro_use(assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne, todo, unreachable, panic, trace, debug, info, warn, error, unwrap)]
#[cfg(feature = "defmt")]
extern crate defmt;

#[allow(unused)]
#[macro_use(trace, debug, info, warn, error)]
#[cfg(feature = "log")]
extern crate log;

mod block_on;
mod yield_now;

pub mod join;
pub mod select;

pub use block_on::*;
pub use yield_now::*;
