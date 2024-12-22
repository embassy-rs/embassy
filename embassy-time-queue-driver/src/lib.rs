#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! This crate is an implementation detail of `embassy-time-driver`.
//!
//! As a HAL user, you should not need to depend on this crate directly.
//!
//! As a HAL implementer, you need to depend on this crate if you want to implement a time driver,
//! but how you should do so is documented in `embassy-time-driver`.

#[cfg(feature = "_generic-queue")]
pub mod queue_generic;
#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(feature = "_generic-queue")]
pub use queue_generic::Queue;
#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;
