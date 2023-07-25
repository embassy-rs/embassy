#![cfg_attr(not(test), no_std)]
#![allow(clippy::new_without_default)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod atomic_ring_buffer;
pub mod drop;
mod macros;
mod peripheral;
pub mod ratio;
pub mod ring_buffer;
pub use peripheral::{Peripheral, PeripheralRef};
pub mod interrupt;
