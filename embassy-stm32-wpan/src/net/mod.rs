//! The MLME-SAP allows the transport of management commands between the next higher layer and the MLME.
//! The MCPS-SAP supports the transport of data.

pub mod commands;
mod consts;
pub mod control;
mod driver;
pub mod event;
pub mod iface;
pub mod indications;
mod macros;
mod opcodes;
pub mod responses;
pub mod runner;
pub mod typedefs;

pub use crate::net::control::Control;
pub use crate::net::driver::{Driver, DriverState};
pub use crate::net::runner::Runner;

const MTU: usize = 127;
