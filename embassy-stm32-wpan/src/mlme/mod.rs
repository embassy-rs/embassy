//! The MLME-SAP allows the transport of management commands between the next higher layer and the MLME.
//! The MCPS-SAP supports the transport of data.

pub mod commands;
mod consts;
pub mod control;
mod driver;
pub mod event;
pub mod indications;
mod macros;
mod opcodes;
pub mod responses;
pub mod runner;
pub mod typedefs;

pub use crate::mlme::control::Control;
pub use crate::mlme::driver::{Driver, DriverState};
pub use crate::mlme::runner::Runner;

const MTU: usize = 127;
