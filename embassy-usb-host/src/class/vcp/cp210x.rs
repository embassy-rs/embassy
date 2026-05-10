//! Silicon Labs CP210x USB ↔ UART bridge driver.
//!
//! Implements the AN571 protocol: vendor-class bulk data transport
//! (one bulk IN + one bulk OUT per interface) plus vendor-interface
//! control requests for line coding, flow control, modem signalling
//! and break. Covers single- and multi-port parts.
//!
//! A [`Cp210xDevice`] owns the device-level control pipe on endpoint 0.
//! Each UART interface is then opened as a [`Cp210xPort`] that borrows
//! the device; control requests from all ports are serialized through
//! the shared control pipe.
//!
//! [`Cp210xDevice::new`] guards the control pipe with a `NoopRawMutex`,
//! which is free but `!Sync`; use it for single-port parts or whenever
//! the device stays in one task. For multi-port parts driven concurrently,
//! use [`Cp210xDevice::new_with_raw_mutex`] with a `Sync` raw mutex such as
//! `CriticalSectionRawMutex`.
//!
//! # Example
//!
//! ```rust,ignore
//! use embassy_usb_host::class::vcp::cp210x::{Cp210xDevice, LineCoding, Parity, StopBits, id};
//!
//! if enum_info.device_desc.vendor_id != id::VID_SILABS {
//!     continue;
//! }
//!
//! let device = Cp210xDevice::new(&bus, &enum_info)?;
//! let mut port = device.port(&config_buf[..config_len], 0)?;
//! port.enable().await?;
//! port.set_line_coding(&LineCoding {
//!     baud_rate: 115200,
//!     data_bits: 8,
//!     parity: Parity::None,
//!     stop_bits: StopBits::One,
//! }).await?;
//! port.set_control_line_state(true, true).await?;
//!
//! let mut buf = [0u8; 64];
//! let n = port.read(&mut buf).await?;
//! port.write(&buf[..n]).await?;
//! ```

use core::marker::PhantomData;

use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_sync::mutex::Mutex;
use embassy_usb_driver::host::{PipeError, SplitInfo, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::control::SetupPacket;
use crate::descriptor::ConfigurationDescriptorChain;
use crate::handler::EnumerationInfo;

/// Silicon Labs VID and CP210x PIDs.
pub mod id {
    /// Silicon Laboratories vendor ID.
    pub const VID_SILABS: u16 = 0x10C4;
    /// Default CP210x product ID.
    pub const PID_CP210X: u16 = 0xEA60;
    /// Alternate CP210x product ID used by some SKUs.
    pub const PID_CP210X_ALT: u16 = 0xEA70;
}

// AN571 §5, Table 6.
const IFC_ENABLE: u8 = 0x00;
const SET_BAUDDIV: u8 = 0x01;
const GET_BAUDDIV: u8 = 0x02;
const SET_LINE_CTL: u8 = 0x03;
const GET_LINE_CTL: u8 = 0x04;
const SET_BREAK: u8 = 0x05;
const SET_MHS: u8 = 0x07;
const GET_MDMSTS: u8 = 0x08;
const PURGE: u8 = 0x12;
const SET_FLOW: u8 = 0x13;
const GET_FLOW: u8 = 0x14;
const GET_BAUDRATE: u8 = 0x1D;
const SET_BAUDRATE: u8 = 0x1E;

const VENDOR_CLASS: u8 = 0xFF;

/// Baud-rate generator reference clock (AN571 §5.1).
const BAUD_CLOCK: u32 = 3_686_400;

/// Cached baud-rate API flavour supported by the device's firmware.
#[derive(Copy, Clone, PartialEq, Eq)]
enum BaudMode {
    /// Not yet probed.
    Unknown,
    /// `SET_BAUDRATE` / `GET_BAUDRATE` (CP2102 with current firmware,
    /// CP2102N, CP2103/4/5/8).
    Direct,
    /// `SET_BAUDDIV` / `GET_BAUDDIV` (CP2101, early CP2102/3 firmware).
    Divisor,
}

/// Shared state protected by [`Cp210xDevice::ctrl`].
struct CtrlState<P> {
    pipe: P,
    baud_mode: BaudMode,
}

async fn vendor_out<P>(pipe: &mut P, interface: u8, request: u8, value: u16, data: &[u8]) -> Result<(), Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket::vendor_interface_out(request, value, interface as u16, data.len() as u16);
    pipe.control_out(&setup.to_bytes(), data).await?;
    Ok(())
}

async fn vendor_in<P>(pipe: &mut P, interface: u8, request: u8, value: u16, buf: &mut [u8]) -> Result<(), Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket::vendor_interface_in(request, value, interface as u16, buf.len() as u16);
    let n = pipe.control_in(&setup.to_bytes(), buf).await?;
    if n != buf.len() {
        return Err(Cp210xError::InvalidResponse);
    }
    Ok(())
}

async fn set_baud_rate_direct<P>(pipe: &mut P, interface: u8, baud: u32) -> Result<(), Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    vendor_out(pipe, interface, SET_BAUDRATE, 0, &baud.to_le_bytes()).await
}

async fn set_baud_rate_divisor<P>(pipe: &mut P, interface: u8, baud: u32) -> Result<(), Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let div = (BAUD_CLOCK + baud / 2) / baud;
    if div == 0 || div > u16::MAX as u32 {
        return Err(Cp210xError::InvalidArgument);
    }
    vendor_out(pipe, interface, SET_BAUDDIV, div as u16, &[]).await
}

async fn get_baud_rate_direct<P>(pipe: &mut P, interface: u8) -> Result<u32, Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let mut buf = [0u8; 4];
    vendor_in(pipe, interface, GET_BAUDRATE, 0, &mut buf).await?;
    Ok(u32::from_le_bytes(buf))
}

async fn get_baud_rate_divisor<P>(pipe: &mut P, interface: u8) -> Result<u32, Cp210xError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let mut buf = [0u8; 2];
    vendor_in(pipe, interface, GET_BAUDDIV, 0, &mut buf).await?;
    let div = u16::from_le_bytes(buf);
    if div == 0 {
        return Err(Cp210xError::InvalidResponse);
    }
    Ok(BAUD_CLOCK / div as u32)
}

/// Parity setting.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Parity {
    /// No parity bit.
    None = 0,
    /// Odd parity.
    Odd = 1,
    /// Even parity.
    Even = 2,
    /// Always 1.
    Mark = 3,
    /// Always 0.
    Space = 4,
}

impl Parity {
    fn from_bits(b: u8) -> Option<Self> {
        Some(match b {
            0 => Self::None,
            1 => Self::Odd,
            2 => Self::Even,
            3 => Self::Mark,
            4 => Self::Space,
            _ => return None,
        })
    }
}

/// Number of stop bits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum StopBits {
    /// 1 stop bit.
    One = 0,
    /// 1.5 stop bits.
    OneAndHalf = 1,
    /// 2 stop bits.
    Two = 2,
}

impl StopBits {
    fn from_bits(b: u8) -> Option<Self> {
        Some(match b {
            0 => Self::One,
            1 => Self::OneAndHalf,
            2 => Self::Two,
            _ => return None,
        })
    }
}

/// Serial line parameters.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LineCoding {
    /// Baud rate in bits per second.
    pub baud_rate: u32,
    /// Data bits. Legal values are 5, 6, 7 and 8.
    pub data_bits: u8,
    /// Parity setting.
    pub parity: Parity,
    /// Stop bits.
    pub stop_bits: StopBits,
}

impl Default for LineCoding {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: StopBits::One,
        }
    }
}

macro_rules! bitflags {
    ($($tt:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::bitflags! { $($tt)* }
        #[cfg(not(feature = "defmt"))]
        bitflags::bitflags! { #[derive(Debug, Clone, PartialEq)] $($tt)* }
    };
}

bitflags! {
    /// Modem status byte returned by `GET_MDMSTS` (AN571 §5.10).
    pub struct ModemStatus: u8 {
        /// DTR output asserted.
        const DTR = 1 << 0;
        /// RTS output asserted.
        const RTS = 1 << 1;
        /// CTS input asserted.
        const CTS = 1 << 4;
        /// DSR input asserted.
        const DSR = 1 << 5;
        /// Ring indicator input asserted.
        const RI  = 1 << 6;
        /// Data-carrier-detect input asserted.
        const DCD = 1 << 7;
    }
}

bitflags! {
    /// Bitmask passed to [`Cp210xPort::purge`] (AN571 §5.27).
    pub struct PurgeMask: u16 {
        /// Clear the transmit queue.
        const TX = (1 << 0) | (1 << 2);
        /// Clear the receive queue.
        const RX = (1 << 1) | (1 << 3);
        /// Clear both queues.
        const ALL = Self::TX.bits() | Self::RX.bits();
    }
}

/// DTR output mode (AN571 Table 10, `SERIAL_DTR_MASK`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DtrMode {
    /// DTR held inactive.
    Inactive = 0,
    /// DTR held active.
    Active = 1,
    /// DTR driven by the CP210x flow-control logic.
    FlowControl = 2,
}

/// RTS output mode (AN571 Table 11, `SERIAL_RTS_MASK`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RtsMode {
    /// RTS statically inactive.
    Inactive = 0,
    /// RTS statically active.
    Active = 1,
    /// RTS used for receive flow control.
    FlowControl = 2,
    /// RTS acts as a transmit-active signal.
    TxActive = 3,
}

/// Flow-control configuration (AN571 Tables 9–11).
///
/// Maps the commonly used bits of the 16-byte `SET_FLOW` / `GET_FLOW`
/// payload. For bits not covered here (e.g. `SERIAL_XOFF_CONTINUE`,
/// `SERIAL_NULL_STRIPPING`, `SERIAL_ERROR_CHAR`, `SERIAL_BREAK_CHAR`),
/// use [`Cp210xPort::set_flow_control_raw`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControl {
    /// DTR output mode.
    pub dtr: DtrMode,
    /// RTS output mode.
    pub rts: RtsMode,
    /// Treat CTS as a handshake line.
    pub cts_handshake: bool,
    /// Treat DSR as a handshake line.
    pub dsr_handshake: bool,
    /// Treat DCD as a handshake line.
    pub dcd_handshake: bool,
    /// Discard received data while DSR is low.
    pub dsr_sensitivity: bool,
    /// Act on XON/XOFF received from the end device.
    pub auto_transmit: bool,
    /// Emit XON/XOFF to the end device based on local buffer fill.
    pub auto_receive: bool,
    /// Threshold (bytes of free space) for sending XON in auto-receive mode.
    pub xon_limit: u32,
    /// Threshold (bytes of free space) for sending XOFF in auto-receive mode.
    pub xoff_limit: u32,
}

impl Default for FlowControl {
    fn default() -> Self {
        Self {
            dtr: DtrMode::Active,
            rts: RtsMode::Active,
            cts_handshake: false,
            dsr_handshake: false,
            dcd_handshake: false,
            dsr_sensitivity: false,
            auto_transmit: false,
            auto_receive: false,
            xon_limit: 0,
            xoff_limit: 0,
        }
    }
}

impl FlowControl {
    fn to_bytes(self) -> [u8; 16] {
        let mut ctrl = self.dtr as u32;
        if self.cts_handshake {
            ctrl |= 1 << 3;
        }
        if self.dsr_handshake {
            ctrl |= 1 << 4;
        }
        if self.dcd_handshake {
            ctrl |= 1 << 5;
        }
        if self.dsr_sensitivity {
            ctrl |= 1 << 6;
        }

        let mut repl = 0u32;
        if self.auto_transmit {
            repl |= 1 << 0;
        }
        if self.auto_receive {
            repl |= 1 << 1;
        }
        repl |= (self.rts as u32) << 6;

        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&ctrl.to_le_bytes());
        buf[4..8].copy_from_slice(&repl.to_le_bytes());
        buf[8..12].copy_from_slice(&self.xon_limit.to_le_bytes());
        buf[12..16].copy_from_slice(&self.xoff_limit.to_le_bytes());
        buf
    }

    fn from_bytes(buf: &[u8; 16]) -> Self {
        let ctrl = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let repl = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let xon = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let xoff = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);

        let dtr = match ctrl & 0b11 {
            0 => DtrMode::Inactive,
            1 => DtrMode::Active,
            _ => DtrMode::FlowControl,
        };
        let rts = match (repl >> 6) & 0b11 {
            0 => RtsMode::Inactive,
            1 => RtsMode::Active,
            2 => RtsMode::FlowControl,
            _ => RtsMode::TxActive,
        };

        Self {
            dtr,
            rts,
            cts_handshake: ctrl & (1 << 3) != 0,
            dsr_handshake: ctrl & (1 << 4) != 0,
            dcd_handshake: ctrl & (1 << 5) != 0,
            dsr_sensitivity: ctrl & (1 << 6) != 0,
            auto_transmit: repl & (1 << 0) != 0,
            auto_receive: repl & (1 << 1) != 0,
            xon_limit: xon,
            xoff_limit: xoff,
        }
    }
}

/// CP210x host driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Cp210xError {
    /// Transfer error.
    Transfer(PipeError),
    /// No vendor-class interface at `interface_idx` with a bulk IN/OUT pair.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
    /// Device response had an unexpected length or out-of-range field.
    InvalidResponse,
    /// Argument was out of range for the CP210x protocol.
    InvalidArgument,
}

impl From<PipeError> for Cp210xError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for Cp210xError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No CP210x interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

impl core::error::Error for Cp210xError {}

impl embedded_io_async::Error for Cp210xError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Transfer(e) => match e {
                PipeError::Disconnected => embedded_io_async::ErrorKind::NotConnected,
                PipeError::BufferOverflow => embedded_io_async::ErrorKind::OutOfMemory,
                PipeError::Timeout => embedded_io_async::ErrorKind::TimedOut,
                _ => embedded_io_async::ErrorKind::Other,
            },
            Self::NoInterface => embedded_io_async::ErrorKind::NotFound,
            Self::NoPipe => embedded_io_async::ErrorKind::OutOfMemory,
            Self::InvalidResponse => embedded_io_async::ErrorKind::InvalidData,
            Self::InvalidArgument => embedded_io_async::ErrorKind::InvalidInput,
        }
    }
}

/// Descriptor-located info for a single CP210x interface.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Cp210xInfo {
    /// USB interface number.
    pub interface: u8,
    /// Bulk IN endpoint address.
    pub bulk_in_ep: u8,
    /// Bulk IN max packet size.
    pub bulk_in_mps: u16,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT max packet size.
    pub bulk_out_mps: u16,
}

/// Return the `n`th (0-indexed) vendor-class interface in `config_desc`
/// that exposes a bulk IN + bulk OUT endpoint pair.
///
/// Use `interface_idx = 0` for single-port CP210x parts; `0..2` for
/// CP2105; `0..4` for CP2108.
pub fn find_cp210x(config_desc: &[u8], interface_idx: u8) -> Option<Cp210xInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;

    let mut seen = 0u8;
    for iface in cfg.iter_interface() {
        if iface.interface_class != VENDOR_CLASS || iface.alternate_setting != 0 {
            continue;
        }

        let mut in_ep = None;
        let mut out_ep = None;
        for ep in iface.iter_endpoints() {
            if ep.ep_type() != EndpointType::Bulk {
                continue;
            }
            if ep.is_in() {
                in_ep = Some((ep.endpoint_address, ep.max_packet_size));
            } else {
                out_ep = Some((ep.endpoint_address, ep.max_packet_size));
            }
        }

        if let (Some((in_a, in_m)), Some((out_a, out_m))) = (in_ep, out_ep) {
            if seen == interface_idx {
                return Some(Cp210xInfo {
                    interface: iface.interface_number,
                    bulk_in_ep: in_a,
                    bulk_in_mps: in_m,
                    bulk_out_ep: out_a,
                    bulk_out_mps: out_m,
                });
            }
            seen += 1;
        }
    }

    None
}

/// CP210x device — owns the shared control pipe on endpoint 0.
///
/// Open one [`Cp210xPort`] per UART interface via [`Cp210xDevice::port`].
/// Control requests from all open ports are serialized through the
/// device's internal async mutex; bulk I/O on different ports runs
/// concurrently.
///
/// Construct with [`Cp210xDevice::new`] for the common single-task case;
/// the control-pipe mutex is a [`NoopRawMutex`] and has no runtime cost.
/// To use ports concurrently, construct with [`Cp210xDevice::new_with_raw_mutex`]
/// and pick a `Sync` raw mutex:
///
/// ```rust,ignore
/// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
/// let device = Cp210xDevice::<_, CriticalSectionRawMutex>::new_with_raw_mutex(
///     &bus, &enum_info,
/// )?;
/// ```
pub struct Cp210xDevice<'d, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    alloc: A,
    /// Device-level control pipe bundled with any state that needs to
    /// stay consistent with it (currently the lazily-probed baud-rate
    /// API flavour).
    ctrl: Mutex<M, CtrlState<A::Pipe<pipe::Control, pipe::InOut>>>,
    device_address: u8,
    split: Option<SplitInfo>,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, A> Cp210xDevice<'d, A, NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
{
    /// Allocate the device-level control pipe on endpoint 0, using a
    /// [`NoopRawMutex`] for the shared control pipe.
    ///
    /// The resulting device is `!Sync`. Use this constructor for
    /// single-port parts, or whenever the device and all its ports
    /// stay confined to one task. For multi-task sharing, use
    /// [`Cp210xDevice::new_with_raw_mutex`] instead.
    ///
    /// Performs no I/O.
    pub fn new(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, Cp210xError> {
        Self::new_with_raw_mutex(alloc, enum_info)
    }
}

impl<'d, A, M> Cp210xDevice<'d, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Allocate the device-level control pipe on endpoint 0, using
    /// the caller-chosen raw mutex `M` for the shared control pipe.
    ///
    /// Pick a `Sync` raw mutex (e.g. `CriticalSectionRawMutex`) to
    /// drive multiple ports concurrently. For single-port devices,
    /// prefer [`Cp210xDevice::new`].
    ///
    /// Performs no I/O.
    pub fn new_with_raw_mutex(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, Cp210xError> {
        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.device_desc.max_packet_size0 as u16,
            interval_ms: 0,
        };

        let device_address = enum_info.device_address;
        let split = enum_info.split();

        let ctrl = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| Cp210xError::NoPipe)?;

        Ok(Self {
            alloc: alloc.clone(),
            ctrl: Mutex::new(CtrlState {
                pipe: ctrl,
                baud_mode: BaudMode::Unknown,
            }),
            device_address,
            split,
            _phantom: PhantomData,
        })
    }

    /// Open the `interface_idx`-th UART port.
    ///
    /// Locates the `interface_idx`-th vendor-class interface with a
    /// bulk IN / bulk OUT pair (use `0` for single-port parts; `0..2`
    /// for CP2105; `0..4` for CP2108) and allocates its bulk pipes.
    ///
    /// No I/O is performed with the device. The caller must call
    /// [`Cp210xPort::enable`] before transferring data; existing
    /// baud-rate, line-coding, flow-control, and modem-line settings
    /// inside the CP210x are preserved.
    ///
    /// The driver does not validate which port is in use.
    /// Avoid opening the same port multiple times.
    pub fn port<'dev>(
        &'dev self,
        config_desc: &[u8],
        interface_idx: u8,
    ) -> Result<Cp210xPort<'dev, 'd, A, M>, Cp210xError> {
        let info = find_cp210x(config_desc, interface_idx).ok_or(Cp210xError::NoInterface)?;

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.bulk_in_ep & 0x0F) as usize, UsbDirection::In),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            interval_ms: 0,
        };

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.bulk_out_ep & 0x0F) as usize, UsbDirection::Out),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval_ms: 0,
        };

        let in_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(self.device_address, &in_ep_info, self.split)
            .map_err(|_| Cp210xError::NoPipe)?;
        let out_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(self.device_address, &out_ep_info, self.split)
            .map_err(|_| Cp210xError::NoPipe)?;

        Ok(Cp210xPort {
            device: self,
            in_ch,
            out_ch,
            interface: info.interface,
        })
    }
}

/// A single UART port on a [`Cp210xDevice`].
///
/// Owns the bulk IN/OUT pipes for one interface and borrows the
/// device for control requests.
pub struct Cp210xPort<'dev, 'd, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    device: &'dev Cp210xDevice<'d, A, M>,
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
    interface: u8,
}

impl<'dev, 'd, A, M> Cp210xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// USB interface number this port is bound to.
    pub fn interface(&self) -> u8 {
        self.interface
    }

    async fn vendor_out(&mut self, request: u8, value: u16, data: &[u8]) -> Result<(), Cp210xError> {
        let mut ctrl = self.device.ctrl.lock().await;
        vendor_out(&mut ctrl.pipe, self.interface, request, value, data).await
    }

    async fn vendor_in(&mut self, request: u8, value: u16, buf: &mut [u8]) -> Result<(), Cp210xError> {
        let mut ctrl = self.device.ctrl.lock().await;
        vendor_in(&mut ctrl.pipe, self.interface, request, value, buf).await
    }

    /// Enable the UART interface.
    ///
    /// Issues `IFC_ENABLE(1)`.
    pub async fn enable(&mut self) -> Result<(), Cp210xError> {
        self.vendor_out(IFC_ENABLE, 1, &[]).await
    }

    /// Disable the UART interface.
    ///
    /// Issues `IFC_ENABLE(0)`.
    pub async fn disable(&mut self) -> Result<(), Cp210xError> {
        self.vendor_out(IFC_ENABLE, 0, &[]).await
    }

    /// Program the UART baud rate in bauds per second.
    ///
    /// Issues `SET_BAUDRATE`, or `SET_BAUDDIV` with a rounded divisor
    /// of the 3.6864 MHz baud-rate generator clock on legacy firmware
    /// that stalls the modern request (CP2101, early CP2102/3). The
    /// flavour is probed on the first baud-rate call and cached on
    /// the device.
    pub async fn set_baud_rate(&mut self, baud: u32) -> Result<(), Cp210xError> {
        if baud == 0 {
            return Err(Cp210xError::InvalidArgument);
        }
        let iface = self.interface;
        let mut ctrl = self.device.ctrl.lock().await;
        let CtrlState { pipe, baud_mode } = &mut *ctrl;
        match *baud_mode {
            BaudMode::Direct => set_baud_rate_direct(pipe, iface, baud).await,
            BaudMode::Divisor => set_baud_rate_divisor(pipe, iface, baud).await,
            BaudMode::Unknown => match set_baud_rate_direct(pipe, iface, baud).await {
                Ok(()) => {
                    *baud_mode = BaudMode::Direct;
                    Ok(())
                }
                Err(Cp210xError::Transfer(PipeError::Stall)) => {
                    *baud_mode = BaudMode::Divisor;
                    set_baud_rate_divisor(pipe, iface, baud).await
                }
                Err(e) => Err(e),
            },
        }
    }

    /// Read the UART baud rate in bauds per second.
    ///
    /// Issues `GET_BAUDRATE`, falling back to `GET_BAUDDIV` (divided
    /// out through the 3.6864 MHz baud-rate generator clock) on legacy
    /// firmware that stalls the modern request.
    pub async fn baud_rate(&mut self) -> Result<u32, Cp210xError> {
        let iface = self.interface;
        let mut ctrl = self.device.ctrl.lock().await;
        let CtrlState { pipe, baud_mode } = &mut *ctrl;
        match *baud_mode {
            BaudMode::Direct => get_baud_rate_direct(pipe, iface).await,
            BaudMode::Divisor => get_baud_rate_divisor(pipe, iface).await,
            BaudMode::Unknown => match get_baud_rate_direct(pipe, iface).await {
                Ok(b) => {
                    *baud_mode = BaudMode::Direct;
                    Ok(b)
                }
                Err(Cp210xError::Transfer(PipeError::Stall)) => {
                    *baud_mode = BaudMode::Divisor;
                    get_baud_rate_divisor(pipe, iface).await
                }
                Err(e) => Err(e),
            },
        }
    }

    /// Program baud rate, data/stop bits and parity.
    ///
    /// Issues `SET_LINE_CTL` followed by [`set_baud_rate`](Self::set_baud_rate).
    ///
    /// # Cancellation
    ///
    /// Not cancel-safe: dropping the future between the two control
    /// transfers leaves the device with the new framing but the old
    /// baud rate. Re-issue the full line coding before resuming data
    /// transfer.
    pub async fn set_line_coding(&mut self, coding: &LineCoding) -> Result<(), Cp210xError> {
        if !matches!(coding.data_bits, 5..=8) {
            return Err(Cp210xError::InvalidArgument);
        }
        let line_ctl = (coding.stop_bits as u16) | ((coding.parity as u16) << 4) | ((coding.data_bits as u16) << 8);
        self.vendor_out(SET_LINE_CTL, line_ctl, &[]).await?;
        self.set_baud_rate(coding.baud_rate).await
    }

    /// Read baud rate, data/stop bits and parity.
    ///
    /// Issues `GET_LINE_CTL` followed by [`baud_rate`](Self::baud_rate).
    pub async fn line_coding(&mut self) -> Result<LineCoding, Cp210xError> {
        let mut buf = [0u8; 2];
        self.vendor_in(GET_LINE_CTL, 0, &mut buf).await?;
        let ctl = u16::from_le_bytes(buf);
        let stop_bits = StopBits::from_bits((ctl & 0xF) as u8).ok_or(Cp210xError::InvalidResponse)?;
        let parity = Parity::from_bits(((ctl >> 4) & 0xF) as u8).ok_or(Cp210xError::InvalidResponse)?;
        let data_bits = (ctl >> 8) as u8;
        if !matches!(data_bits, 5..=8) {
            return Err(Cp210xError::InvalidResponse);
        }
        let baud_rate = self.baud_rate().await?;
        Ok(LineCoding {
            baud_rate,
            data_bits,
            parity,
            stop_bits,
        })
    }

    /// Drive DTR and RTS to the given levels.
    ///
    /// Issues `SET_MHS` asserting both DTR and RTS masks, overriding
    /// any [`DtrMode::FlowControl`] or [`RtsMode::FlowControl`] setting
    /// until the next flow-control event.
    pub async fn set_control_line_state(&mut self, dtr: bool, rts: bool) -> Result<(), Cp210xError> {
        let value = (dtr as u16) | ((rts as u16) << 1) | (1 << 8) | (1 << 9);
        self.vendor_out(SET_MHS, value, &[]).await
    }

    /// Read the modem status lines.
    ///
    /// Issues `GET_MDMSTS`. Reserved bits 2 and 3 of the response are
    /// discarded.
    pub async fn modem_status(&mut self) -> Result<ModemStatus, Cp210xError> {
        let mut buf = [0u8; 1];
        self.vendor_in(GET_MDMSTS, 0, &mut buf).await?;
        Ok(ModemStatus::from_bits_truncate(buf[0]))
    }

    /// Assert or release a break condition on TX.
    ///
    /// Issues `SET_BREAK`.
    pub async fn set_break(&mut self, asserted: bool) -> Result<(), Cp210xError> {
        self.vendor_out(SET_BREAK, asserted as u16, &[]).await
    }

    /// Clear the selected TX and/or RX queues.
    ///
    /// Issues `PURGE` with the given mask. On CP2102N this also clears
    /// the current `SET_FLOW` configuration; re-apply flow control
    /// afterwards if needed.
    pub async fn purge(&mut self, mask: PurgeMask) -> Result<(), Cp210xError> {
        self.vendor_out(PURGE, mask.bits(), &[]).await
    }

    /// Apply a flow-control configuration.
    ///
    /// Issues `SET_FLOW`. For bits not covered by [`FlowControl`], use
    /// [`set_flow_control_raw`](Self::set_flow_control_raw).
    pub async fn set_flow_control(&mut self, fc: &FlowControl) -> Result<(), Cp210xError> {
        let bytes = fc.to_bytes();
        self.set_flow_control_raw(&bytes).await
    }

    /// Apply a raw flow-control configuration.
    ///
    /// Issues `SET_FLOW` with the given payload. Layout is little-endian
    /// `ulControlHandshake` | `ulFlowReplace` | `ulXonLimit` |
    /// `ulXoffLimit` (4 B each); see AN571 Tables 9–11.
    pub async fn set_flow_control_raw(&mut self, raw: &[u8; 16]) -> Result<(), Cp210xError> {
        self.vendor_out(SET_FLOW, 0, raw).await
    }

    /// Read the flow-control configuration.
    ///
    /// Issues `GET_FLOW`.
    pub async fn flow_control(&mut self) -> Result<FlowControl, Cp210xError> {
        let mut buf = [0u8; 16];
        self.vendor_in(GET_FLOW, 0, &mut buf).await?;
        Ok(FlowControl::from_bytes(&buf))
    }

    /// Read bytes from the UART receive stream.
    ///
    /// # Cancellation
    ///
    /// Not cancel-safe: bytes already received from the device but
    /// not yet copied into `buf` are lost if the future is dropped.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Cp210xError> {
        Ok(self.in_ch.request_in(buf).await?)
    }

    /// Write bytes to the UART transmit stream.
    ///
    /// # Cancellation
    ///
    /// Not cancel-safe: the remote may observe partial data if the
    /// future is dropped mid-transfer.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Cp210xError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }
}

impl<'dev, 'd, A, M> embedded_io_async::ErrorType for Cp210xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    type Error = Cp210xError;
}

impl<'dev, 'd, A, M> embedded_io_async::Read for Cp210xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Cp210xPort::read(self, buf).await
    }
}

impl<'dev, 'd, A, M> embedded_io_async::Write for Cp210xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Cp210xPort::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
