#![no_std]
#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod select;
mod yield_now;

pub use select::*;
pub use yield_now::*;
