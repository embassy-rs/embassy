//! USB Audio Class (UAC).

#[cfg(feature = "device")]
pub mod device;
#[cfg(feature = "host")]
pub mod host;
pub mod terminal_type;
pub(crate) mod uac1_codes;
#[allow(missing_docs)]
pub mod uac2_codes;
