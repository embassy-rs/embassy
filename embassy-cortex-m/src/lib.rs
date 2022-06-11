#![no_std]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod executor;
pub mod interrupt;
pub mod peripheral;
