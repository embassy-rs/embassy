//! RPC backends — dialect-specific ioctl sequences behind an operation-centric trait.

mod fg;
mod ioctl_ctx;

pub use fg::FgBackend;
pub use ioctl_ctx::IoctlCtx;

use crate::InterfaceType;
use crate::control::{Error, Status};

/// Normalized control-path events from the coprocessor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostedEvent {
    /// Coprocessor finished booting.
    Init,
    /// Periodic heartbeat.
    Heartbeat,
    /// Station associated to an access point.
    StaConnected {
        /// Firmware status code.
        resp: i32,
    },
    /// Station left an access point.
    StaDisconnected {
        /// Disconnect reason.
        reason: u32,
    },
}

/// User-facing control operations; each method may perform one or more ioctl round-trips.
pub trait RpcBackend {
    fn encode_ioctl(&self, buffer: &mut [u8], req: &[u8]) -> usize;
    fn process_serial_data<'pl>(&mut self, payload: &'pl [u8]) -> Option<(bool, &'pl [u8])>;

    fn encode_iface_type(&self, iface_type: InterfaceType) -> u8;
    fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType>;

    async fn config_heartbeat(&self, ctx: &mut IoctlCtx<'_>, secs: u32) -> Result<(), Error>;
    async fn set_sta_mode(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn get_mac_addr(&self, ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error>;
    async fn connect_ap(&self, ctx: &mut IoctlCtx<'_>, ssid: &str, pwd: &str) -> Result<(), Error>;
    async fn disconnect_ap(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn get_status(&self, ctx: &mut IoctlCtx<'_>) -> Result<Status, Error>;
    async fn ota_begin(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn ota_write(&self, ctx: &mut IoctlCtx<'_>, chunk: &[u8]) -> Result<(), Error>;
    async fn ota_end(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    fn normalize_event(&self, raw: &[u8]) -> Option<HostedEvent>;
}
