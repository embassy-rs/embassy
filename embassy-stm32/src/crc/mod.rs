//! Cyclic Redundancy Check (CRC)
#[cfg_attr(crc_v1, path = "v1.rs")]
#[cfg_attr(crc_v2, path = "v2v3.rs")]
#[cfg_attr(crc_v3, path = "v2v3.rs")]
mod _version;

pub use _version::*;
