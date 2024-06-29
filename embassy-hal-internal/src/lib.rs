#![no_std]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This crate MUST go first, and use the old `extern crate` syntax, so that textual scope is used
// and these macros become globally available here.
#[macro_use]
extern crate embassy_fmt;

pub mod atomic_ring_buffer;
pub mod drop;
mod macros;
mod peripheral;
pub mod ratio;
pub use peripheral::{Peripheral, PeripheralRef};

#[cfg(feature = "cortex-m")]
pub mod interrupt;
