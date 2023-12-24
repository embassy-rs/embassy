//! Controller Area Network (CAN)
#![macro_use]

#[cfg_attr(can_bxcan, path = "bxcan.rs")]
#[cfg_attr(any(can_fdcan_v1, can_fdcan_h7), path = "fdcan.rs")]
mod _version;
pub use _version::*;
