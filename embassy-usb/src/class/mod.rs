//! Implementations of well-known USB classes.
pub mod cdc_acm;
pub mod cdc_ncm;
pub mod cmsis_dap_v2;
pub mod hid;
pub mod midi;
#[cfg(feature = "msc")]
pub mod msc;
pub mod uac1;
pub mod web_usb;
