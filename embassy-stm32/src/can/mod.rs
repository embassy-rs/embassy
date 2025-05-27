//! Controller Area Network (CAN)
//!
//! ## Examples
//!
//! - [STM32F4](https://github.com/embassy-rs/embassy/blob/main/examples/stm32f4/src/bin/can.rs)
#![macro_use]

#[cfg_attr(can_bxcan, path = "bxcan/mod.rs")]
#[cfg_attr(any(can_fdcan_v1, can_fdcan_h7), path = "fdcan.rs")]
mod _version;
pub use _version::*;

mod common;
pub mod enums;
pub mod frame;
pub mod util;

pub use frame::Frame;
