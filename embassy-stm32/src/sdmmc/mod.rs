#![macro_use]

#[cfg_attr(feature = "_sdmmc_v1", path = "v1.rs")]
#[cfg_attr(feature = "_sdmmc_v2", path = "v2.rs")]
mod _version;

pub use _version::*;
