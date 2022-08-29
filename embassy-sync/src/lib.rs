#![cfg_attr(not(any(feature = "std", feature = "wasm")), no_std)]
#![cfg_attr(feature = "nightly", feature(generic_associated_types, type_alias_impl_trait))]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

// internal use
mod ring_buffer;

pub mod blocking_mutex;
pub mod channel;
pub mod mutex;
pub mod pipe;
pub mod pubsub;
pub mod signal;
pub mod waitqueue;
