#![macro_use]

#[cfg_attr(can_bxcan, path = "bxcan.rs")]
#[cfg_attr(can_fdcan, path = "fdcan.rs")]
mod _version;
pub use _version::*;
