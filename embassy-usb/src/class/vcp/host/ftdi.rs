//! FTDI (Future Technology Devices International) USB ↔ UART bridge driver.
//!
//! Driver for FTDI USB-to-UART bridge ICs:
//! - FT232R / FT232BM / FT245R
//! - FT2232C / FT2232D / FT2232H (Dual channel)
//! - FT4232H (Quad channel)
//! - FT232H (Single channel high speed)
//! - FT-X series (FT200XD, FT201X, FT230X, FT231X, FT234XD)
//!
//! FTDI devices transport data over bulk pipes. Every incoming bulk IN packet is
//! prepended with a 2-byte modem/line status header that is automatically parsed
//! and stripped by this driver. Control requests (baud rate, line parameters, flow
//! control, modem signals, latency timer) are sent via vendor-specific device requests.

use core::marker::PhantomData;

use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_sync::mutex::Mutex;
use embassy_usb_driver::host::{PipeError, SplitInfo, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::class::cdc_acm::{LineCoding, ParityType, StopBits};
use crate::host::class::ClassMatch;
use crate::host::control::SetupPacket;
use crate::host::descriptor::{ClassCode, ConfigurationDescriptorChain, InterfaceDescriptor};
use crate::host::handler::EnumerationInfo;

/// FTDI VID and common FTDI PIDs.
pub mod id {
    /// FTDI Vendor ID.
    pub const VID_FTDI: u16 = 0x0403;
    /// FT232R / FT232BM / FT245R PID.
    pub const PID_FT232R: u16 = 0x6001;
    /// FT2232C / FT2232D / FT2232H (dual-port) PID.
    pub const PID_FT2232: u16 = 0x6010;
    /// FT4232H (quad-port) PID.
    pub const PID_FT4232: u16 = 0x6011;
    /// FT232H (single-port high-speed) PID.
    pub const PID_FT232H: u16 = 0x6014;
    /// FT-X series PID (FT200XD, FT201X, FT230X, FT231X, etc.).
    pub const PID_FTX: u16 = 0x6015;
}

/// Matcher for FTDI vendor-class interfaces.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FtdiMatcher;

impl ClassMatch for FtdiMatcher {
    const NAME: &'static str = "FTDI";
    const CODE: Option<ClassCode> = Some(ClassCode::new(0xFF, 0xFF, 0xFF));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == 0xFF
    }
}

// FTDI vendor request codes
const FTDI_SIO_RESET: u8 = 0;
const FTDI_SIO_SET_MODEM_CTRL: u8 = 1;
const FTDI_SIO_SET_FLOW_CTRL: u8 = 2;
const FTDI_SIO_SET_BAUDRATE: u8 = 3;
const FTDI_SIO_SET_DATA: u8 = 4;
const FTDI_SIO_GET_MODEM_STATUS: u8 = 5;
const FTDI_SIO_SET_LATENCY_TIMER: u8 = 9;
const FTDI_SIO_GET_LATENCY_TIMER: u8 = 10;

// FTDI reset values
const FTDI_SIO_RESET_SIO: u16 = 0;
const FTDI_SIO_RESET_PURGE_RX: u16 = 1;
const FTDI_SIO_RESET_PURGE_TX: u16 = 2;

const VENDOR_CLASS: u8 = 0xFF;

/// FTDI host driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FtdiError {
    /// Transfer error.
    Transfer(PipeError),
    /// No vendor-class interface at `interface_idx` with a bulk IN/OUT pair.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
    /// Device response had an unexpected length or out-of-range field.
    InvalidResponse,
    /// Argument was out of range for the FTDI protocol.
    InvalidArgument,
}

impl From<PipeError> for FtdiError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for FtdiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No FTDI interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

impl core::error::Error for FtdiError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for FtdiError {
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

macro_rules! bitflags {
    ($($tt:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::bitflags! { $($tt)* }
        #[cfg(not(feature = "defmt"))]
        bitflags::bitflags! { #[derive(Debug, Clone, Copy, PartialEq, Eq)] $($tt)* }
    };
}

bitflags! {
    /// Modem status byte prepended to bulk IN packets or returned by `GET_MODEM_STATUS`.
    pub struct ModemStatus: u8 {
        /// Clear to Send.
        const CTS = 1 << 4;
        /// Data Set Ready.
        const DSR = 1 << 5;
        /// Ring Indicator.
        const RI  = 1 << 6;
        /// Data Carrier Detect.
        const DCD = 1 << 7;
    }
}

bitflags! {
    /// Line status byte prepended to bulk IN packets or returned by `GET_MODEM_STATUS`.
    pub struct LineStatus: u8 {
        /// Overrun error.
        const OVERRUN_ERROR   = 1 << 1;
        /// Parity error.
        const PARITY_ERROR    = 1 << 2;
        /// Framing error.
        const FRAMING_ERROR   = 1 << 3;
        /// Break interrupt detected.
        const BREAK_INTERRUPT = 1 << 4;
        /// Transmitter holding register empty.
        const THRE            = 1 << 5;
        /// Transmitter empty.
        const TEMT            = 1 << 6;
    }
}

/// Flow control configuration for FTDI UART.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl {
    /// Flow control disabled.
    None,
    /// RTS/CTS hardware handshake.
    RtsCts,
    /// DTR/DSR hardware handshake.
    DtrDsr,
    /// XON/XOFF software flow control.
    XonXoff,
}

/// Descriptor-located info for a single FTDI interface.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FtdiInfo {
    /// USB interface number.
    pub interface: u8,
    /// Bulk IN endpoint address.
    pub bulk_in_ep: u8,
    /// Bulk IN maximum packet size.
    pub bulk_in_mps: u16,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT maximum packet size.
    pub bulk_out_mps: u16,
}

fn find_interface(desc_bytes: &[u8], interface_idx: u8) -> Option<FtdiInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(desc_bytes).ok()?;
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
                return Some(FtdiInfo {
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

/// Calculate baud divisor for FTDI devices.
///
/// Implements FTDI AN120 fractional divisor encoding across both standard 48 MHz
/// base clock (183 baud to 3 Mbaud) and high-speed 120 MHz base clock (up to 12 Mbaud).
/// Returns `(value, index)` parameters for `FTDI_SIO_SET_BAUDRATE`.
pub fn calculate_baud_divisor(baud: u32, port_idx: u8) -> Result<(u16, u16), FtdiError> {
    const DIVFRAC: [u8; 8] = [0, 3, 2, 4, 1, 5, 6, 7];

    let (divisor3, clk_flag) = if baud <= 3_000_000 {
        if baud < 183 {
            return Err(FtdiError::InvalidArgument);
        }
        // Base clock 48 MHz, 16x oversampled -> 3 MHz.
        // divisor3 = round((3_000_000 * 8) / baud) = round(24_000_000 / baud)
        let div = ((48_000_000u64 + baud as u64) / (2 * baud as u64)) as u32;
        (div, 0u32)
    } else if baud <= 12_000_000 {
        // High speed 120 MHz clock, 10x oversampled -> 12 MHz.
        // divisor3 = round((8 * 120_000_000) / (10 * baud)) = round(96_000_000 / baud)
        let div = ((96_000_000u64 + (baud / 2) as u64) / baud as u64) as u32;
        (div, 0x20000u32)
    } else {
        return Err(FtdiError::InvalidArgument);
    };

    if divisor3 == 0 {
        return Err(FtdiError::InvalidArgument);
    }

    let mut divisor = divisor3 >> 3;
    let frac = DIVFRAC[(divisor3 & 0x7) as usize];
    divisor |= (frac as u32) << 14;

    // Special cases for highest baud rates:
    if divisor == 1 {
        divisor = 0; // 3,000,000 or 12,000,000 baud
    } else if divisor == 0x4001 {
        divisor = 1; // 2,000,000 or 8,000,000 baud
    }

    divisor |= clk_flag;

    let value = (divisor & 0xFFFF) as u16;
    let mut index = (divisor >> 16) as u16;

    // Port index is 1-based (Channel A = 1, Channel B = 2, ...)
    let channel = (port_idx + 1) as u16;
    index = (index << 8) | channel;

    Ok((value, index))
}

/// FTDI device — owns the control pipe on endpoint 0.
pub struct FtdiDevice<'d, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    alloc: A,
    ctrl: Mutex<M, A::Pipe<pipe::Control, pipe::InOut>>,
    device_address: u8,
    split: Option<SplitInfo>,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, A> FtdiDevice<'d, A, NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
{
    /// Allocate the device-level control pipe using a [`NoopRawMutex`].
    pub fn new(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, FtdiError> {
        Self::new_with_raw_mutex(alloc, enum_info)
    }
}

impl<'d, A, M> FtdiDevice<'d, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Allocate the device-level control pipe with a custom [`RawMutex`].
    pub fn new_with_raw_mutex(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, FtdiError> {
        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.max_packet_size0(),
            interval_ms: 0,
            max_burst: 0,
        };

        let device_address = enum_info.device_address;
        let split = enum_info.split();

        let ctrl_pipe = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| FtdiError::NoPipe)?;

        Ok(Self {
            alloc: alloc.clone(),
            ctrl: Mutex::new(ctrl_pipe),
            device_address,
            split,
            _phantom: PhantomData,
        })
    }

    /// Open a port on this FTDI device.
    pub fn port<'dev>(
        &'dev self,
        config_desc: &[u8],
        interface_idx: u8,
    ) -> Result<FtdiPort<'dev, 'd, A, M>, FtdiError> {
        let info = find_interface(config_desc, interface_idx).ok_or(FtdiError::NoInterface)?;

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_in_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            interval_ms: 0,
            max_burst: 0,
        };
        let in_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(self.device_address, &in_ep_info, self.split)
            .map_err(|_| FtdiError::NoPipe)?;

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_out_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval_ms: 0,
            max_burst: 0,
        };
        let out_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(self.device_address, &out_ep_info, self.split)
            .map_err(|_| FtdiError::NoPipe)?;

        Ok(FtdiPort {
            device: self,
            info,
            port_idx: interface_idx,
            reader: Reader {
                in_ch,
                rx_buf: [0u8; 512],
                rx_pos: 0,
                rx_len: 0,
                modem_status: ModemStatus::empty(),
                line_status: LineStatus::empty(),
                _phantom: PhantomData,
            },
            writer: Writer {
                out_ch,
                _phantom: PhantomData,
            },
        })
    }
}

/// FTDI UART port.
pub struct FtdiPort<'dev, 'd, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    device: &'dev FtdiDevice<'d, A, M>,
    info: FtdiInfo,
    port_idx: u8,
    reader: Reader<'dev, 'd, A, M>,
    writer: Writer<'dev, 'd, A, M>,
}

impl<'dev, 'd, A, M> FtdiPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn vendor_out(&mut self, request: u8, value: u16, index: u16) -> Result<(), FtdiError> {
        let mut ctrl = self.device.ctrl.lock().await;
        let setup = SetupPacket::vendor_device_out(request, value, index, 0);
        ctrl.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    async fn vendor_in(&mut self, request: u8, value: u16, index: u16, buf: &mut [u8]) -> Result<(), FtdiError> {
        let mut ctrl = self.device.ctrl.lock().await;
        let setup = SetupPacket::vendor_device_in(request, value, index, buf.len() as u16);
        let n = ctrl.control_in(&setup.to_bytes(), buf).await?;
        if n != buf.len() {
            return Err(FtdiError::InvalidResponse);
        }
        Ok(())
    }

    /// Return descriptor-located info for this port.
    pub fn info(&self) -> &FtdiInfo {
        &self.info
    }

    /// Reset port state (SIO reset).
    pub async fn reset(&mut self) -> Result<(), FtdiError> {
        let channel = (self.port_idx + 1) as u16;
        self.vendor_out(FTDI_SIO_RESET, FTDI_SIO_RESET_SIO, channel).await
    }

    /// Purge receive queue on device.
    pub async fn purge_rx(&mut self) -> Result<(), FtdiError> {
        let channel = (self.port_idx + 1) as u16;
        self.reader.rx_pos = 0;
        self.reader.rx_len = 0;
        self.vendor_out(FTDI_SIO_RESET, FTDI_SIO_RESET_PURGE_RX, channel).await
    }

    /// Purge transmit queue on device.
    pub async fn purge_tx(&mut self) -> Result<(), FtdiError> {
        let channel = (self.port_idx + 1) as u16;
        self.vendor_out(FTDI_SIO_RESET, FTDI_SIO_RESET_PURGE_TX, channel).await
    }

    /// Configure the UART baud rate.
    pub async fn set_baudrate(&mut self, baud: u32) -> Result<(), FtdiError> {
        let (val, idx) = calculate_baud_divisor(baud, self.port_idx)?;
        self.vendor_out(FTDI_SIO_SET_BAUDRATE, val, idx).await
    }

    /// Configure line parameters (data bits, stop bits, parity).
    pub async fn set_line_coding(&mut self, line: &LineCoding) -> Result<(), FtdiError> {
        let data_bits = match line.data_bits() {
            7 => 7u16,
            8 => 8u16,
            _ => return Err(FtdiError::InvalidArgument),
        };

        let parity = match line.parity_type() {
            ParityType::None => 0u16,
            ParityType::Odd => 1u16,
            ParityType::Even => 2u16,
            ParityType::Mark => 3u16,
            ParityType::Space => 4u16,
        };

        let stop = match line.stop_bits() {
            StopBits::One => 0u16,
            StopBits::OnePointFive => 1u16,
            StopBits::Two => 2u16,
        };

        let val = data_bits | (parity << 8) | (stop << 11);
        let channel = (self.port_idx + 1) as u16;
        self.vendor_out(FTDI_SIO_SET_DATA, val, channel).await
    }

    /// Set modem control line states (DTR, RTS).
    pub async fn set_control_line_state(&mut self, dtr: bool, rts: bool) -> Result<(), FtdiError> {
        let mut val: u16 = 0;
        val |= 1 << 8; // enable DTR change
        if dtr {
            val |= 1 << 0;
        }
        val |= 2 << 8; // enable RTS change
        if rts {
            val |= 2 << 0;
        }
        let channel = (self.port_idx + 1) as u16;
        self.vendor_out(FTDI_SIO_SET_MODEM_CTRL, val, channel).await
    }

    /// Configure hardware or software flow control.
    pub async fn set_flow_control(&mut self, flow: FlowControl) -> Result<(), FtdiError> {
        let mode: u16 = match flow {
            FlowControl::None => 0,
            FlowControl::RtsCts => 1,
            FlowControl::DtrDsr => 2,
            FlowControl::XonXoff => 4,
        };
        let channel = (self.port_idx + 1) as u16;
        let idx = (mode << 8) | channel;
        self.vendor_out(FTDI_SIO_SET_FLOW_CTRL, 0, idx).await
    }

    /// Set latency timer in milliseconds (1 to 255 ms, default 16 ms).
    pub async fn set_latency_timer(&mut self, latency_ms: u8) -> Result<(), FtdiError> {
        if latency_ms == 0 {
            return Err(FtdiError::InvalidArgument);
        }
        let channel = (self.port_idx + 1) as u16;
        self.vendor_out(FTDI_SIO_SET_LATENCY_TIMER, latency_ms as u16, channel)
            .await
    }

    /// Query the current latency timer value.
    pub async fn get_latency_timer(&mut self) -> Result<u8, FtdiError> {
        let channel = (self.port_idx + 1) as u16;
        let mut buf = [0u8; 1];
        self.vendor_in(FTDI_SIO_GET_LATENCY_TIMER, 0, channel, &mut buf).await?;
        Ok(buf[0])
    }

    /// Query current modem and line status.
    pub async fn get_modem_status(&mut self) -> Result<(ModemStatus, LineStatus), FtdiError> {
        let channel = (self.port_idx + 1) as u16;
        let mut buf = [0u8; 2];
        self.vendor_in(FTDI_SIO_GET_MODEM_STATUS, 0, channel, &mut buf).await?;
        Ok((
            ModemStatus::from_bits_truncate(buf[0]),
            LineStatus::from_bits_truncate(buf[1]),
        ))
    }

    /// Read data from the UART, automatically stripping the 2-byte FTDI status header.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, FtdiError> {
        self.reader.read(buf).await
    }

    /// Write data to the UART.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, FtdiError> {
        self.writer.write(data).await
    }

    /// Split into independent reader and writer halves.
    pub fn split(self) -> (Reader<'dev, 'd, A, M>, Writer<'dev, 'd, A, M>) {
        (self.reader, self.writer)
    }

    /// Last observed modem status from received packets.
    pub fn last_modem_status(&self) -> ModemStatus {
        self.reader.last_modem_status()
    }

    /// Last observed line status from received packets.
    pub fn last_line_status(&self) -> LineStatus {
        self.reader.last_line_status()
    }
}

/// Port reader half.
pub struct Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
    rx_buf: [u8; 512],
    rx_pos: usize,
    rx_len: usize,
    modem_status: ModemStatus,
    line_status: LineStatus,
    _phantom: PhantomData<(&'dev (), M)>,
}

impl<'dev, 'd, A, M> Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Read data from the UART, automatically stripping the 2-byte FTDI status header.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, FtdiError> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            if self.rx_pos < self.rx_len {
                let available = self.rx_len - self.rx_pos;
                let to_copy = available.min(buf.len());
                buf[..to_copy].copy_from_slice(&self.rx_buf[self.rx_pos..self.rx_pos + to_copy]);
                self.rx_pos += to_copy;
                if self.rx_pos == self.rx_len {
                    self.rx_pos = 0;
                    self.rx_len = 0;
                }
                return Ok(to_copy);
            }

            let n = self.in_ch.request_in(&mut self.rx_buf).await?;
            if n < 2 {
                continue;
            }

            self.modem_status = ModemStatus::from_bits_truncate(self.rx_buf[0]);
            self.line_status = LineStatus::from_bits_truncate(self.rx_buf[1]);

            if n == 2 {
                // Keepalive/status-only packet with no UART payload
                continue;
            }

            let payload_len = n - 2;
            let to_copy = payload_len.min(buf.len());
            buf[..to_copy].copy_from_slice(&self.rx_buf[2..2 + to_copy]);

            if to_copy < payload_len {
                self.rx_pos = 2 + to_copy;
                self.rx_len = n;
            } else {
                self.rx_pos = 0;
                self.rx_len = 0;
            }

            return Ok(to_copy);
        }
    }

    /// Last observed modem status from received packets.
    pub fn last_modem_status(&self) -> ModemStatus {
        self.modem_status
    }

    /// Last observed line status from received packets.
    pub fn last_line_status(&self) -> LineStatus {
        self.line_status
    }
}

/// Port writer half.
pub struct Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
    _phantom: PhantomData<(&'dev (), M)>,
}

impl<'dev, 'd, A, M> Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Write data to the UART.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, FtdiError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::ErrorType for FtdiPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    type Error = FtdiError;
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Read for FtdiPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        FtdiPort::read(self, buf).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Write for FtdiPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        FtdiPort::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::ErrorType for Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    type Error = FtdiError;
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Read for Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Reader::read(self, buf).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::ErrorType for Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    type Error = FtdiError;
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Write for Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Writer::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baud_divisor() {
        // 3 Mbaud -> divisor = 0, port A index = 1
        let (val, idx) = calculate_baud_divisor(3_000_000, 0).unwrap();
        assert_eq!(val, 0);
        assert_eq!(idx, 1);

        // 2 Mbaud -> divisor = 1, port B index = 2
        let (val, idx) = calculate_baud_divisor(2_000_000, 1).unwrap();
        assert_eq!(val, 1);
        assert_eq!(idx, 2);

        // 115200 baud -> 48MHz / 16 / 115200 = 26.0416 => divisor 26 (0x001a)
        let (val, idx) = calculate_baud_divisor(115200, 0).unwrap();
        assert_eq!(val, 26);
        assert_eq!(idx, 1);

        // 9600 baud -> 3000000 / 9600 = 312.5 => divisor 312 (0x0138) with frac 0.5 (code 1 << 14 = 0x4000)
        let (val, idx) = calculate_baud_divisor(9600, 0).unwrap();
        assert_eq!(val, 0x4138);
        assert_eq!(idx, 1);

        // Out of range baud rates
        assert!(calculate_baud_divisor(100, 0).is_err());
        assert!(calculate_baud_divisor(20_000_000, 0).is_err());
    }

    #[test]
    fn test_matcher() {
        let iface = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 2,
            interface_class: 0xFF,
            interface_subclass: 0xFF,
            interface_protocol: 0xFF,
            interface_name: 0,
        };
        assert!(FtdiMatcher::matches(&iface));
    }
}
