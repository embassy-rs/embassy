//! Async low-level wait queues

#[cfg_attr(feature = "turbowakers", path = "atomic_waker_turbo.rs")]
mod atomic_waker;
pub use atomic_waker::*;

#[cfg(not(feature = "turbowakers"))]
mod critical_section_waker;
#[cfg(not(feature = "turbowakers"))]
pub use critical_section_waker::*;

mod waker_registration;
pub use waker_registration::*;

mod multi_waker;
pub use multi_waker::*;

mod cell_waker_registration;
pub use cell_waker_registration::*;
