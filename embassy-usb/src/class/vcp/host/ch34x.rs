//! WCH CH34x (CH340 / CH341) USB ↔ UART bridge driver.
//!
//! Driver for Winchiphead / WCH USB-to-UART bridge ICs (CH340G, CH340C, CH340N, CH340K, CH341A).
//! Transports serial data over bulk endpoints and performs line configuration, baud rate, and
//! modem control via vendor-specific control transfers.

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

/// WCH VID and common CH34x PIDs.
pub mod id {
    /// Winchiphead (WCH) Vendor ID.
    pub const VID_WCH: u16 = 0x1A86;
    /// Standard CH340 (CH340G, CH340C, CH340N, CH340E, CH341A) PID.
    pub const PID_CH340: u16 = 0x7523;
    /// CH340K alternate PID.
    pub const PID_CH340K: u16 = 0x7522;
    /// CH341 alternate PID.
    pub const PID_CH341_ALT: u16 = 0x5523;
}

/// Matcher for the vendor-specific CH34x interface.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ch34xMatcher;

impl ClassMatch for Ch34xMatcher {
    const NAME: &'static str = "CH34x";
    const CODE: Option<ClassCode> = Some(ClassCode::new(0xFF, 0x01, 0x02));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == 0xFF
    }
}

// Vendor request codes
const CH341_REQ_READ_VERSION: u8 = 0x5F;
const CH341_REQ_WRITE_REG: u8 = 0x9A;
#[allow(dead_code)]
const CH341_REQ_READ_REG: u8 = 0x95;
const CH341_REQ_SERIAL_INIT: u8 = 0xA1;
const CH341_REQ_MODEM_CTRL: u8 = 0xA4;

const CH341_REG_DIVISOR: u8 = 0x13;
const CH341_REG_PRESCALER: u8 = 0x12;
const CH341_REG_LCR: u8 = 0x18;
const CH341_REG_LCR2: u8 = 0x25;

const VENDOR_CLASS: u8 = 0xFF;

/// CH34x host driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ch34xError {
    /// Transfer error.
    Transfer(PipeError),
    /// No vendor-class interface at `interface_idx` with a bulk IN/OUT pair.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
    /// Device response had an unexpected length or out-of-range field.
    InvalidResponse,
    /// Argument was out of range for the CH34x protocol.
    InvalidArgument,
}

impl From<PipeError> for Ch34xError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for Ch34xError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No CH34x interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

impl core::error::Error for Ch34xError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for Ch34xError {
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

/// Descriptor-located info for a single CH34x interface.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ch34xInfo {
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
pub fn find_ch34x(config_desc: &[u8], interface_idx: u8) -> Option<Ch34xInfo> {
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
                return Some(Ch34xInfo {
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

/// Calculate the CH340 prescaler and divisor register value for a target baud rate.
pub fn calculate_baud_divisor(baud: u32) -> Result<u16, Ch34xError> {
    if baud < 300 || baud > 2_000_000 {
        return Err(Ch34xError::InvalidArgument);
    }

    const CLKRATE: u32 = 48_000_000;
    const MIN_RATES: [u32; 4] = [
        CLKRATE / (4096 * 255),
        CLKRATE / (512 * 255),
        CLKRATE / (64 * 255),
        CLKRATE / (8 * 255),
    ];

    let mut ps = 0u32;
    for i in (0..4).rev() {
        if baud > MIN_RATES[i] {
            ps = i as u32;
            break;
        }
    }

    let mut fact = 1u32;
    let mut clk_div = 1 << (12 - 3 * ps - fact);
    let mut div = CLKRATE / (clk_div * baud);

    if div < 9 || div > 255 {
        div /= 2;
        clk_div *= 2;
        fact = 0;
    }

    if div < 2 {
        return Err(Ch34xError::InvalidArgument);
    }

    let err_curr = (CLKRATE / (clk_div * div)).abs_diff(baud);
    let err_next = (CLKRATE / (clk_div * (div + 1))).abs_diff(baud);
    if err_next < err_curr {
        div += 1;
    }

    if fact == 1 && div % 2 == 0 {
        div /= 2;
        fact = 0;
    }

    let val = ((0x100 - div) as u16) << 8 | ((fact as u16) << 2) | (ps as u16) | 0x80;
    Ok(val)
}

/// CH34x device — owns the shared control pipe on endpoint 0.
pub struct Ch34xDevice<'d, A, M = NoopRawMutex>
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

impl<'d, A> Ch34xDevice<'d, A, NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
{
    /// Allocate the device-level control pipe using a [`NoopRawMutex`].
    pub fn new(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, Ch34xError> {
        Self::new_with_raw_mutex(alloc, enum_info)
    }
}

impl<'d, A, M> Ch34xDevice<'d, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Allocate the device-level control pipe using a caller-provided raw mutex.
    pub fn new_with_raw_mutex(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, Ch34xError> {
        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.max_packet_size0(),
            interval_ms: 0,
            max_burst: 0,
        };

        let device_address = enum_info.device_address;
        let split = enum_info.split();

        let ctrl = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| Ch34xError::NoPipe)?;

        Ok(Self {
            alloc: alloc.clone(),
            ctrl: Mutex::new(ctrl),
            device_address,
            split,
            _phantom: PhantomData,
        })
    }

    /// Open a UART port on the device.
    pub fn port<'dev>(
        &'dev self,
        config_desc: &[u8],
        interface_idx: u8,
    ) -> Result<Ch34xPort<'dev, 'd, A, M>, Ch34xError> {
        let info = find_ch34x(config_desc, interface_idx).ok_or(Ch34xError::NoInterface)?;

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_in_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            interval_ms: 0,
            max_burst: 0,
        };
        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_out_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval_ms: 0,
            max_burst: 0,
        };

        let in_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(self.device_address, &in_ep_info, self.split)
            .map_err(|_| Ch34xError::NoPipe)?;
        let out_ch = self
            .alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(self.device_address, &out_ep_info, self.split)
            .map_err(|_| Ch34xError::NoPipe)?;

        Ok(Ch34xPort {
            device: self,
            in_ch,
            out_ch,
            interface: info.interface,
        })
    }
}

/// An open CH34x UART port.
pub struct Ch34xPort<'dev, 'd, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    device: &'dev Ch34xDevice<'d, A, M>,
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
    interface: u8,
}

impl<'dev, 'd, A, M> Ch34xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Gets the bound interface number.
    pub fn interface(&self) -> u8 {
        self.interface
    }

    async fn vendor_out(&mut self, request: u8, value: u16, index: u16) -> Result<(), Ch34xError> {
        let mut ctrl = self.device.ctrl.lock().await;
        let setup = SetupPacket::vendor_device_out(request, value, index, 0);
        ctrl.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    async fn vendor_in(&mut self, request: u8, value: u16, index: u16, buf: &mut [u8]) -> Result<(), Ch34xError> {
        let mut ctrl = self.device.ctrl.lock().await;
        let setup = SetupPacket::vendor_device_in(request, value, index, buf.len() as u16);
        let n = ctrl.control_in(&setup.to_bytes(), buf).await?;
        if n != buf.len() {
            return Err(Ch34xError::InvalidResponse);
        }
        Ok(())
    }

    /// Enable and initialize the CH34x chip.
    pub async fn enable(&mut self) -> Result<(), Ch34xError> {
        let mut version = [0u8; 2];
        let _ = self.vendor_in(CH341_REQ_READ_VERSION, 0, 0, &mut version).await;
        self.vendor_out(CH341_REQ_SERIAL_INIT, 0xC29C, 0xB834).await?;
        // Enable RX/TX via LCR2 register 0x2518
        self.vendor_out(CH341_REQ_WRITE_REG, 0x2518, 0x00C3).await?;
        self.set_baud_rate(9600).await?;
        self.set_control_line_state(true, true).await?;
        Ok(())
    }

    /// Set baud rate in bauds per second.
    pub async fn set_baud_rate(&mut self, baud: u32) -> Result<(), Ch34xError> {
        let val = calculate_baud_divisor(baud)?;
        let reg = (CH341_REG_DIVISOR as u16) << 8 | (CH341_REG_PRESCALER as u16);
        self.vendor_out(CH341_REQ_WRITE_REG, reg, val).await
    }

    /// Set line coding (baud rate, data bits, stop bits, parity).
    pub async fn set_line_coding(&mut self, coding: &LineCoding) -> Result<(), Ch34xError> {
        let mut lcr = match coding.data_bits() {
            5 => 0x00,
            6 => 0x01,
            7 => 0x02,
            8 => 0x03,
            _ => return Err(Ch34xError::InvalidArgument),
        };

        if coding.stop_bits() == StopBits::Two {
            lcr |= 0x04;
        }

        match coding.parity_type() {
            ParityType::None => {}
            ParityType::Odd => lcr |= 0x08,
            ParityType::Even => lcr |= 0x18,
            ParityType::Mark => lcr |= 0x28,
            ParityType::Space => lcr |= 0x38,
        }

        let reg = (CH341_REG_LCR2 as u16) << 8 | (CH341_REG_LCR as u16);
        self.vendor_out(CH341_REQ_WRITE_REG, reg, 0x00C0 | (lcr as u16)).await?;
        self.set_baud_rate(coding.data_rate()).await
    }

    /// Set DTR and RTS control line states.
    pub async fn set_control_line_state(&mut self, dtr: bool, rts: bool) -> Result<(), Ch34xError> {
        let mut val = 0u16;
        if !dtr {
            val |= 1 << 5;
        }
        if !rts {
            val |= 1 << 6;
        }
        self.vendor_out(CH341_REQ_MODEM_CTRL, val, 0).await
    }

    /// Read bytes from UART receive stream.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Ch34xError> {
        Ok(self.in_ch.request_in(buf).await?)
    }

    /// Write bytes to UART transmit stream.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Ch34xError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }

    /// Split port into independent reader and writer handles.
    pub fn split(self) -> (Reader<'dev, 'd, A, M>, Writer<'dev, 'd, A, M>) {
        (
            Reader {
                _device: self.device,
                in_ch: self.in_ch,
            },
            Writer {
                _device: self.device,
                out_ch: self.out_ch,
            },
        )
    }
}

/// Port reader half.
pub struct Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    _device: &'dev Ch34xDevice<'d, A, M>,
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
}

impl<'dev, 'd, A, M> Reader<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Read bytes from the port.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Ch34xError> {
        Ok(self.in_ch.request_in(buf).await?)
    }
}

/// Port writer half.
pub struct Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    _device: &'dev Ch34xDevice<'d, A, M>,
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
}

impl<'dev, 'd, A, M> Writer<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Write bytes to the port.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Ch34xError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::ErrorType for Ch34xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    type Error = Ch34xError;
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Read for Ch34xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ch34xPort::read(self, buf).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'dev, 'd, A, M> embedded_io_async::Write for Ch34xPort<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ch34xPort::write(self, buf).await
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
    type Error = Ch34xError;
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
    type Error = Ch34xError;
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
        assert_eq!(calculate_baud_divisor(115200).unwrap(), 0xCC83);
        assert_eq!(calculate_baud_divisor(9600).unwrap(), 0xB282);
        assert!(calculate_baud_divisor(100).is_err());
        assert!(calculate_baud_divisor(3_000_000).is_err());
    }

    #[test]
    fn test_matcher() {
        let iface = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 2,
            interface_class: 0xFF,
            interface_subclass: 0x01,
            interface_protocol: 0x02,
            interface_name: 0,
        };
        assert!(Ch34xMatcher::matches(&iface));
    }
}
