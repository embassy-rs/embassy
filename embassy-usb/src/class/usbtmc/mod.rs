//! USB Test & Measurement Class (USBTMC) and USB488 implementation.
//!
//! USBTMC is a USB class protocol used by test and measurement instruments (e.g. oscilloscopes,
//! digital multimeters, power supplies, RF synthesizers) to communicate with host software such as
//! VISA (NI-VISA, Keysight IO Libraries, PyVISA, MATLAB, and LabVIEW).
//!
//! This module implements both the base USBTMC 1.0 specification and the USB488 subclass for
//! IEEE 488.2 / SCPI instrument control.

pub mod device;
pub mod types;

pub use device::*;
pub use types::*;
