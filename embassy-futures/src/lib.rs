#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod block_on;
mod yield_now;

pub mod join;
pub mod select;

pub use block_on::*;
pub use yield_now::*;
