//! USB Printer Class 1.1 Host implementation.
//!
//! Connects to USB printers (such as ESC/POS thermal printers and receipt printers).
//! Transmits print data to the printer via [`embedded_io_async::Write`] or [`PrinterHost::write`].

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use super::{
    PRINTER_SUBCLASS_PRINTER, PortStatus, REQ_GET_DEVICE_ID, REQ_GET_PORT_STATUS, REQ_SOFT_RESET, USB_CLASS_PRINTER,
};
use crate::host::class::ClassMatch;
use crate::host::control::SetupPacket;
use crate::host::descriptor::{ClassCode, ConfigurationDescriptorChain, InterfaceDescriptor};
use crate::host::handler::EnumerationInfo;

/// Matcher for USB Printer class interfaces.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PrinterMatcher;

impl ClassMatch for PrinterMatcher {
    const NAME: &'static str = "Printer";
    const CODE: Option<ClassCode> = Some(ClassCode::new(0x07, 0x01, 0x02));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == USB_CLASS_PRINTER && iface.interface_subclass == PRINTER_SUBCLASS_PRINTER
    }
}

/// USB printer host error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PrinterError {
    /// Pipe transfer error.
    Transfer(PipeError),
    /// No USB printer interface found.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
    /// Device response had an unexpected length or invalid format.
    InvalidResponse,
    /// Invalid argument.
    InvalidArgument,
    /// Attempted to read from a unidirectional printer.
    Unidirectional,
}

impl From<PipeError> for PrinterError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for PrinterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No printer interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
            Self::Unidirectional => write!(f, "Printer is unidirectional"),
        }
    }
}

impl core::error::Error for PrinterError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for PrinterError {
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
            Self::Unidirectional => embedded_io_async::ErrorKind::Unsupported,
        }
    }
}

/// Descriptor-located info for a USB printer.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PrinterInfo {
    /// USB interface number.
    pub interface: u8,
    /// Interface protocol (1 = unidirectional, 2 = bidirectional).
    pub protocol: u8,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT max packet size.
    pub bulk_out_mps: u16,
    /// Optional Bulk IN endpoint address (for bidirectional printers).
    pub bulk_in_ep: Option<u8>,
    /// Optional Bulk IN max packet size.
    pub bulk_in_mps: Option<u16>,
}

/// Find a USB printer interface in the configuration descriptors.
pub fn find_printer(config_desc: &[u8]) -> Option<PrinterInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;

    for iface in cfg.iter_interface() {
        if iface.interface_class != USB_CLASS_PRINTER || iface.interface_subclass != PRINTER_SUBCLASS_PRINTER {
            continue;
        }

        let mut bulk_out = None;
        let mut bulk_in = None;

        for ep in iface.iter_endpoints() {
            if ep.ep_type() == EndpointType::Bulk {
                if ep.is_in() {
                    bulk_in = Some((ep.endpoint_address, ep.max_packet_size));
                } else {
                    bulk_out = Some((ep.endpoint_address, ep.max_packet_size));
                }
            }
        }

        if let Some((out_ep, out_mps)) = bulk_out {
            let (in_ep, in_mps) = match bulk_in {
                Some((ep, mps)) => (Some(ep), Some(mps)),
                None => (None, None),
            };

            return Some(PrinterInfo {
                interface: iface.interface_number,
                protocol: iface.interface_protocol,
                bulk_out_ep: out_ep,
                bulk_out_mps: out_mps,
                bulk_in_ep: in_ep,
                bulk_in_mps: in_mps,
            });
        }
    }

    None
}

/// USB Printer host driver.
pub struct PrinterHost<'d, A: UsbHostAllocator<'d>> {
    ctrl: A::Pipe<pipe::Control, pipe::InOut>,
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
    in_ch: Option<A::Pipe<pipe::Bulk, pipe::In>>,
    info: PrinterInfo,
}

impl<'d, A: UsbHostAllocator<'d>> PrinterHost<'d, A> {
    /// Create a new USB printer host instance.
    pub fn new(alloc: &A, enum_info: &EnumerationInfo, config_desc: &[u8]) -> Result<Self, PrinterError> {
        let info = find_printer(config_desc).ok_or(PrinterError::NoInterface)?;

        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.device_desc.max_packet_size0 as u16,
            interval: 0,
            max_burst: 0,
        };
        let ctrl = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &ctrl_ep_info, enum_info.split())
            .map_err(|_| PrinterError::NoPipe)?;

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_out_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval: 0,
            max_burst: 0,
        };
        let out_ch = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(enum_info.device_address, &out_ep_info, enum_info.split())
            .map_err(|_| PrinterError::NoPipe)?;

        let in_ch = if let (Some(in_ep), Some(in_mps)) = (info.bulk_in_ep, info.bulk_in_mps) {
            let in_ep_info = EndpointInfo {
                addr: EndpointAddress::from(in_ep),
                ep_type: EndpointType::Bulk,
                max_packet_size: in_mps,
                interval: 0,
                max_burst: 0,
            };
            Some(
                alloc
                    .alloc_pipe::<pipe::Bulk, pipe::In>(enum_info.device_address, &in_ep_info, enum_info.split())
                    .map_err(|_| PrinterError::NoPipe)?,
            )
        } else {
            None
        };

        Ok(Self {
            ctrl,
            out_ch,
            in_ch,
            info,
        })
    }

    /// Return descriptor-located info.
    pub fn info(&self) -> &PrinterInfo {
        &self.info
    }

    /// Write print data (e.g. ESC/POS, text, graphics) to the printer.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, PrinterError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }

    /// Read data from a bi-directional printer.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, PrinterError> {
        let in_ch = self.in_ch.as_mut().ok_or(PrinterError::Unidirectional)?;
        Ok(in_ch.request_in(buf).await?)
    }

    /// Query the IEEE 1284 Device ID string from the printer.
    ///
    /// Copies the ASCII string into `buf` and returns the string length.
    pub async fn get_device_id(&mut self, buf: &mut [u8]) -> Result<usize, PrinterError> {
        let mut raw = [0u8; 258];
        let idx = (self.info.interface as u16) << 8;
        let setup = SetupPacket::class_interface_in(REQ_GET_DEVICE_ID, 0, idx, raw.len() as u16);
        let n = self.ctrl.control_in(&setup.to_bytes(), &mut raw).await?;

        if n < 2 {
            return Err(PrinterError::InvalidResponse);
        }

        let total_len = u16::from_be_bytes([raw[0], raw[1]]) as usize;
        if total_len < 2 {
            return Err(PrinterError::InvalidResponse);
        }

        let str_len = (total_len - 2).min(n - 2);
        let copy_len = str_len.min(buf.len());
        buf[..copy_len].copy_from_slice(&raw[2..2 + copy_len]);
        Ok(copy_len)
    }

    /// Query current port status flags (error, selected/online, paper empty).
    pub async fn get_port_status(&mut self) -> Result<PortStatus, PrinterError> {
        let mut buf = [0u8; 1];
        let setup = SetupPacket::class_interface_in(REQ_GET_PORT_STATUS, 0, self.info.interface as u16, 1);
        let n = self.ctrl.control_in(&setup.to_bytes(), &mut buf).await?;
        if n != 1 {
            return Err(PrinterError::InvalidResponse);
        }
        Ok(PortStatus::from_bits_truncate(buf[0]))
    }

    /// Issue a soft reset request to the printer.
    pub async fn soft_reset(&mut self) -> Result<(), PrinterError> {
        let setup = SetupPacket::class_interface_out(REQ_SOFT_RESET, 0, self.info.interface as u16, 0);
        self.ctrl.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::ErrorType for PrinterHost<'d, A> {
    type Error = PrinterError;
}

#[cfg(feature = "embedded-io")]
impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::Write for PrinterHost<'d, A> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        PrinterHost::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::Read for PrinterHost<'d, A> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        PrinterHost::read(self, buf).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printer_matcher() {
        let iface = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 2,
            interface_class: 0x07,
            interface_subclass: 0x01,
            interface_protocol: 0x02,
            interface_name: 0,
        };
        assert!(PrinterMatcher::matches(&iface));

        let non_printer = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 2,
            interface_class: 0x03, // HID
            interface_subclass: 0x01,
            interface_protocol: 0x01,
            interface_name: 0,
        };
        assert!(!PrinterMatcher::matches(&non_printer));
    }
}
