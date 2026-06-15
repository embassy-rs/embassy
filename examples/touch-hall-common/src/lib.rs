//! JSON-driven hall lighting config and CAN bitmask protocol shared by the
//! RVT50 OxivGL demo and the SDL host port.

#![no_std]

pub mod can_bridge;

#[cfg(feature = "rhai")]
pub mod rhai_state;

mod config {
    include!(concat!(env!("OUT_DIR"), "/touch_config.rs"));
}

pub use config::*;
