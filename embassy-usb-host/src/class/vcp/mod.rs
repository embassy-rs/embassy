//! Virtual COM Port (USB ↔ UART bridge) host drivers.
//!
//! Chips in this category are vendor-class USB-to-serial bridges that
//! transport data over bulk pipes and expose private control requests
//! for line configuration, flow control, and modem signalling. Unlike
//! CDC-ACM they carry no class descriptors, so device discovery is
//! VID/PID based.

pub mod cp210x;
