#[cfg_attr(crc_v2, path = "v2.rs")]
#[cfg_attr(crc_v1, path = "v1.rs")]
#[cfg_attr(crc_v3, path = "v2.rs")]
mod _version;

pub use _version::Crc;
