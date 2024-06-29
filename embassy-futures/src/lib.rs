#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This crate MUST go first, and use the old `extern crate` syntax, so that textual scope is used
// and these macros become globally available here.
#[macro_use]
extern crate embassy_fmt;

mod block_on;
mod yield_now;

pub mod join;
pub mod select;

pub use block_on::*;
pub use yield_now::*;
