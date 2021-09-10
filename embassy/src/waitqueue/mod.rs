//! Async low-level wait queues

#[cfg_attr(feature = "executor-agnostic", path = "waker_agnostic.rs")]
mod waker;
pub use waker::*;
