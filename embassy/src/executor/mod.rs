//! Async task executor.

#![deny(missing_docs)]

#[cfg_attr(feature = "std", path = "arch/std.rs")]
#[cfg_attr(feature = "wasm", path = "arch/wasm.rs")]
#[cfg_attr(not(any(feature = "std", feature = "wasm")), path = "arch/arm.rs")]
mod arch;
pub mod raw;
mod spawner;

pub use arch::*;
pub use spawner::*;
