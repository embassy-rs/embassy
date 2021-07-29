#[cfg_attr(any(pwr_h7, pwr_h7smps), path = "h7.rs")]
#[cfg_attr(not(any(pwr_h7, pwr_h7smps)), path = "none.rs")]
mod _version;

pub use _version::*;
