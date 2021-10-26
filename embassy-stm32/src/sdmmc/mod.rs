#![macro_use]

#[cfg_attr(sdmmc_v1, path = "v1.rs")]
#[cfg_attr(sdmmc_v2, path = "v2.rs")]
mod _version;

pub use _version::*;
