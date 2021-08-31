#[cfg_attr(any(pwr_h7, pwr_h7smps), path = "h7.rs")]
#[cfg_attr(pwr_f4, path = "f4.rs")]
#[cfg_attr(pwr_wl5, path = "wl5.rs")]
mod _version;

pub use _version::*;
