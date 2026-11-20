//! USB Test & Measurement Class (USBTMC) and USB488 implementation.
//!
//! USBTMC is a USB class protocol used by test and measurement instruments (e.g. oscilloscopes,
//! digital multimeters, power supplies, RF synthesizers) to communicate with host software such as
//! VISA (NI-VISA, Keysight IO Libraries, PyVISA, MATLAB, and LabVIEW).
//!
//! This module implements both the base USBTMC 1.0 specification and the USB488 subclass for
//! IEEE 488.2 / SCPI instrument control.
//!
//! A Bulk-OUT message that cannot be parsed (a bad header tag, an unexpected MsgID, a packet too
//! short to hold a header, a payload larger than the caller's buffer, or a transfer that ends
//! early) leaves the message stream with no way to tell where the next header starts. The class
//! stalls Bulk-OUT in that case, so the host sees the failure and aborts the transfer instead of
//! having the device read the rest of the message as headers. The stall is cleared by the host
//! with `CLEAR_FEATURE(ENDPOINT_HALT)` or `viClear()`, or by the application calling
//! [`UsbtmcClass::clear_stall`].

#[cfg(feature = "device")]
pub mod device;
pub mod types;

#[cfg(feature = "device")]
pub use device::*;
pub use types::*;
