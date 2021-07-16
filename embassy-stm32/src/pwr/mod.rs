#[cfg_attr(any(pwr_h7, pwr_h7smps), path = "h7.rs")]
mod _version;

pub use _version::*;
