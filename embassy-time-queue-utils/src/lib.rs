#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(feature = "_generic-queue")]
pub mod queue_generic;
#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(feature = "_generic-queue")]
pub use queue_generic::Queue;
#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;
