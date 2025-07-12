//! Async low-level wait queues

#[cfg_attr(feature = "turbowakers", path = "atomic_waker_turbo.rs")]
mod atomic_waker;
pub use atomic_waker::*;

mod waker_registration;
pub use waker_registration::*;

mod multi_waker;
pub use multi_waker::*;
