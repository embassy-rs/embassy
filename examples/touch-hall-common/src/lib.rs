//! JSON-driven hall lighting config and CAN bitmask protocol shared by the
//! RVT50 OxivGL demo and the SDL host port.

#![no_std]

pub mod button_status;
pub mod can_bridge;
pub mod can_input;
pub mod can_refresh;
pub mod can_scheduler;
pub mod input_state;
pub mod touch_feedback;
pub mod touch_hold;

#[cfg(feature = "rhai")]
pub mod rhai_state;

mod config {
    include!(concat!(env!("OUT_DIR"), "/touch_config.rs"));
}

pub use config::*;
