//! RPC backends — dialect-specific ioctl sequences behind an operation-centric trait.

#[cfg(not(any(feature = "esp-hosted-fg", feature = "esp-hosted-mcu")))]
compile_error!("At least one esp-hosted-* feature is required");

#[cfg(feature = "esp-hosted-fg")]
mod fg;
pub(crate) mod ioctl_ctx;
#[cfg(feature = "esp-hosted-mcu")]
mod mcu;

use heapless::Vec;
pub use ioctl_ctx::IoctlCtx;

use crate::control::{Error, Status};
use crate::{FwVersion, InterfaceType, Network, WifiMode};

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

    fn encode_iface_type(&self, iface_type: InterfaceType) -> Option<u8>;
    fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType>;

    async fn init_radio(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn start_wifi(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn config_heartbeat(&self, ctx: &mut IoctlCtx<'_>, secs: u32) -> Result<(), Error>;
    async fn set_mode(&self, ctx: &mut IoctlCtx<'_>, mode: WifiMode) -> Result<(), Error>;
    async fn get_mac_addr(&self, ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error>;
    async fn scan<const N: usize>(&self, ctx: &mut IoctlCtx<'_>, result: &mut Vec<Network, N>) -> Result<(), Error>;
    async fn connect_ap(&self, ctx: &mut IoctlCtx<'_>, ssid: &str, pwd: &str) -> Result<(), Error>;
    async fn disconnect_ap(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn get_fw_version(&self, ctx: &mut IoctlCtx<'_>) -> Result<FwVersion, Error>;
    async fn get_status(&self, ctx: &mut IoctlCtx<'_>) -> Result<Status, Error>;
    async fn ota_begin(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    async fn ota_write(&self, ctx: &mut IoctlCtx<'_>, chunk: &[u8]) -> Result<(), Error>;
    async fn ota_end(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
    fn normalize_event(&self, raw: &[u8]) -> Option<HostedEvent>;
}

#[cfg(all(feature = "esp-hosted-fg", not(feature = "esp-hosted-mcu")))]
pub type Backend = fg::FgBackend;
#[cfg(all(feature = "esp-hosted-mcu", not(feature = "esp-hosted-fg")))]
pub type Backend = mcu::McuBackend;

#[cfg(all(feature = "esp-hosted-fg", feature = "esp-hosted-mcu"))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Backend {
    Auto(NoBackend),
    Fg(fg::FgBackend),
    Mcu(mcu::McuBackend),
}

#[cfg(all(feature = "esp-hosted-fg", feature = "esp-hosted-mcu"))]
impl Default for Backend {
    fn default() -> Self {
        Backend::Auto(NoBackend)
    }
}

#[cfg(all(feature = "esp-hosted-fg", feature = "esp-hosted-mcu"))]
impl RpcBackend for Backend {
    delegate::delegate! {
        to match self {
            Backend::Auto(backend) => backend,
            Backend::Fg(backend) => backend,
            Backend::Mcu(backend) => backend,
        } {
            fn encode_ioctl(&self, buffer: &mut [u8], req: &[u8]) -> usize;

            fn encode_iface_type(&self, iface_type: InterfaceType) -> Option<u8>;
            fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType>;

            async fn start_wifi(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
            async fn config_heartbeat(&self, ctx: &mut IoctlCtx<'_>, secs: u32) -> Result<(), Error>;
            async fn init_radio(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
            async fn set_mode(&self, ctx: &mut IoctlCtx<'_>, mode: WifiMode) -> Result<(), Error>;
            async fn get_mac_addr(&self, ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error>;
            async fn scan<const N: usize>(&self, ctx: &mut IoctlCtx<'_>, result: &mut Vec<Network, N>) -> Result<(), Error>;
            async fn connect_ap(&self, ctx: &mut IoctlCtx<'_>, ssid: &str, pwd: &str) -> Result<(), Error>;
            async fn disconnect_ap(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
            async fn get_status(&self, ctx: &mut IoctlCtx<'_>) -> Result<Status, Error>;
            async fn get_fw_version(&self, ctx: &mut IoctlCtx<'_>) -> Result<FwVersion, Error>;
            async fn ota_begin(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
            async fn ota_write(&self, ctx: &mut IoctlCtx<'_>, chunk: &[u8]) -> Result<(), Error>;
            async fn ota_end(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error>;
            fn normalize_event(&self, raw: &[u8]) -> Option<HostedEvent>;
        }
    }

    fn process_serial_data<'pl>(&mut self, payload: &'pl [u8]) -> Option<(bool, &'pl [u8])> {
        match self {
            Backend::Auto(_backend) => {
                // Ignore responses, we haven't sent anything yet.
                // We will consider the backend detected when we recognise a valid init event.

                if payload.len() < 12 {
                    warn!("serial rx: too short");
                    return None;
                }

                match &payload[0..10] {
                    b"\x01\x06\x00RPCRsp\x02" => return None,
                    b"\x01\x06\x00RPCEvt\x02" => {
                        let len = u16::from_le_bytes(payload[10..][..2].try_into().unwrap()) as usize;
                        if payload.len() < 10 + 2 + len {
                            warn!("serial rx: too short 2");
                            return None;
                        }

                        let evt_data = &payload[10 + 2..][..len];

                        if mcu::McuBackend.normalize_event(evt_data) == Some(HostedEvent::Init) {
                            info!("Detected esp-hosted-mcu");
                            *self = Backend::Mcu(mcu::McuBackend);
                            return Some((true, evt_data));
                        }
                    }
                    _ => {}
                }

                if payload.len() < 14 {
                    warn!("serial rx: too short");
                    return None;
                }

                match &payload[0..12] {
                    b"\x01\x08\x00ctrlResp\x02" => return None,
                    b"\x01\x08\x00ctrlEvnt\x02" => {
                        let len = u16::from_le_bytes(payload[12..][..2].try_into().unwrap()) as usize;
                        if payload.len() < 12 + 2 + len {
                            warn!("serial rx: too short 2");
                            return None;
                        }

                        let evt_data = &payload[12 + 2..][..len];

                        if fg::FgBackend.normalize_event(evt_data) == Some(HostedEvent::Init) {
                            info!("Detected esp-hosted-fg");
                            *self = Backend::Fg(fg::FgBackend);
                            return Some((true, evt_data));
                        }
                    }
                    _ => warn!("serial rx: bad tlv"),
                };

                None
            }
            Backend::Fg(backend) => backend.process_serial_data(payload),
            Backend::Mcu(backend) => backend.process_serial_data(payload),
        }
    }
}

#[cfg(all(feature = "esp-hosted-fg", feature = "esp-hosted-mcu"))]
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct NoBackend;

#[cfg(all(feature = "esp-hosted-fg", feature = "esp-hosted-mcu"))]
impl RpcBackend for NoBackend {
    fn encode_ioctl(&self, _buffer: &mut [u8], _req: &[u8]) -> usize {
        0
    }
    fn process_serial_data<'pl>(&mut self, _payload: &'pl [u8]) -> Option<(bool, &'pl [u8])> {
        None
    }

    fn encode_iface_type(&self, _iface_type: InterfaceType) -> Option<u8> {
        None
    }
    fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType> {
        #[cfg(feature = "esp-hosted-fg")]
        if let Some(InterfaceType::Serial) = fg::FgBackend.decode_iface_type(iface_type) {
            return Some(InterfaceType::Serial);
        }

        #[cfg(feature = "esp-hosted-mcu")]
        if let Some(InterfaceType::Serial) = mcu::McuBackend.decode_iface_type(iface_type) {
            return Some(InterfaceType::Serial);
        }

        None
    }

    async fn init_radio(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        Err(Error::Internal)
    }

    async fn start_wifi(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        Err(Error::Internal)
    }

    async fn config_heartbeat(&self, _ctx: &mut IoctlCtx<'_>, _secs: u32) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn set_mode(&self, _ctx: &mut IoctlCtx<'_>, _mode: WifiMode) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn get_mac_addr(&self, _ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error> {
        Err(Error::Internal)
    }
    async fn scan<const N: usize>(&self, _ctx: &mut IoctlCtx<'_>, _result: &mut Vec<Network, N>) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn connect_ap(&self, _ctx: &mut IoctlCtx<'_>, _ssid: &str, _pwd: &str) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn disconnect_ap(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn get_status(&self, _ctx: &mut IoctlCtx<'_>) -> Result<Status, Error> {
        Err(Error::Internal)
    }
    async fn get_fw_version(&self, _ctx: &mut IoctlCtx<'_>) -> Result<FwVersion, Error> {
        Err(Error::Internal)
    }
    async fn ota_begin(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn ota_write(&self, _ctx: &mut IoctlCtx<'_>, _chunk: &[u8]) -> Result<(), Error> {
        Err(Error::Internal)
    }
    async fn ota_end(&self, _ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        Err(Error::Internal)
    }
    fn normalize_event(&self, _raw: &[u8]) -> Option<HostedEvent> {
        None
    }
}

fn check_resp(resp: i32) -> Result<(), Error> {
    if resp != 0 {
        Err(Error::Failed(resp as u32))
    } else {
        Ok(())
    }
}

fn from_utf8_lossy<const N: usize>(bytes: &[u8]) -> heapless::String<N> {
    const REPLACEMENT: &str = "\u{FFFD}";

    let mut str = heapless::String::new();

    let end = bytes.iter().position(|c| *c == b'\x00').unwrap_or(bytes.len());
    let bytes = &bytes[..end];

    for chunk in bytes.utf8_chunks() {
        if str.push_str(chunk.valid()).is_err() {
            break;
        }
        if !chunk.invalid().is_empty() {
            if str.push_str(REPLACEMENT).is_err() {
                break;
            }
        }
    }

    str
}
