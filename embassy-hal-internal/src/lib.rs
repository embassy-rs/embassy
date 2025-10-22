#![no_std]
#![allow(clippy::new_without_default)]
#![allow(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod atomic_ring_buffer;
pub mod drop;
mod macros;
mod peripheral;
pub mod ratio;
pub use peripheral::{Peri, PeripheralType};

#[cfg(feature = "cortex-m")]
pub mod interrupt;
