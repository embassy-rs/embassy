#[cfg_attr(any(pwr_h7, pwr_h7smps), path = "h7.rs")]
#[cfg_attr(pwr_f4, path = "f4.rs")]
#[cfg_attr(pwr_f7, path = "f7.rs")]
#[cfg_attr(pwr_wl5, path = "wl5.rs")]
#[cfg_attr(pwr_g0, path = "g0.rs")]
#[cfg_attr(pwr_l1, path = "l1.rs")]
mod _version;

pub use _version::*;
