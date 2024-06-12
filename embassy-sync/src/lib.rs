#![cfg_attr(not(feature = "std"), no_std)]
#![allow(async_fn_in_trait)]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[allow(unused)]
#[macro_use(
    assert,
    assert_eq,
    assert_ne,
    debug_assert,
    debug_assert_eq,
    debug_assert_ne,
    todo,
    unreachable,
    panic,
    trace,
    debug,
    info,
    warn,
    error,
    unwrap
)]
#[cfg(feature = "defmt")]
extern crate defmt;

#[allow(unused)]
#[macro_use(trace, debug, info, warn, error)]
#[cfg(feature = "log")]
extern crate log;

// internal use
mod ring_buffer;

pub mod blocking_mutex;
pub mod channel;
pub mod mutex;
pub mod once_lock;
pub mod pipe;
pub mod priority_channel;
pub mod pubsub;
pub mod semaphore;
pub mod signal;
pub mod waitqueue;
pub mod zerocopy_channel;
