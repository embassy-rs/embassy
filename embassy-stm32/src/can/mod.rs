#![macro_use]

#[cfg_attr(can_bxcan, path = "bxcan.rs")]
mod _version;
pub use _version::*;
