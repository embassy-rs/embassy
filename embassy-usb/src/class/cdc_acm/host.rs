//! CDC ACM (Serial over USB) host class driver.
//!
//! This driver can communicate with USB CDC ACM devices (virtual serial ports).

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use super::{
    CDC_PROTOCOL_NONE, CDC_SUBCLASS_ACM, LineCoding, REQ_SET_CONTROL_LINE_STATE, REQ_SET_LINE_CODING, USB_CLASS_CDC,
    USB_CLASS_CDC_DATA,
};
use crate::host::class::{ClassError, HostClass};
use crate::host::control::SetupPacket;
use crate::host::descriptor::{ClassCode, ClassMatch, ConfigurationDescriptorChain, InterfaceDescriptor};
use crate::host::handler::EnumerationInfo;

/// CDC ACM host class driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CdcAcmError {
    /// Transfer error.
    Transfer(PipeError),
    /// No matching CDC ACM interface found in the device.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
}

impl From<PipeError> for CdcAcmError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for CdcAcmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No CDC ACM interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
        }
    }
}

impl core::error::Error for CdcAcmError {}

impl From<CdcAcmError> for ClassError {
    fn from(e: CdcAcmError) -> Self {
        match e {
            CdcAcmError::Transfer(e) => Self::Transfer(e),
            CdcAcmError::NoInterface => Self::NoMatch,
            CdcAcmError::NoPipe => Self::NoPipe,
        }
    }
}

impl embedded_io_async::Error for CdcAcmError {
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
        }
    }
}

/// Information about a CDC ACM interface found in a configuration descriptor.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CdcAcmInfo {
    /// CDC communication interface number.
    pub comm_interface: u8,
    /// CDC data interface number.
    pub data_interface: u8,
    /// Bulk IN endpoint address.
    pub bulk_in_ep: u8,
    /// Bulk IN max packet size.
    pub bulk_in_mps: u16,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT max packet size.
    pub bulk_out_mps: u16,
}

/// Matches the CDC ACM communication interface.
///
/// A CDC ACM function is split across a communication interface and a data
/// interface, so this matcher only identifies the former. [`find_cdc_acm`]
/// finds the associated data interface and endpoints.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CdcAcmMatcher;

impl ClassMatch for CdcAcmMatcher {
    const NAME: &'static str = "CDC ACM";
    const CODE: Option<ClassCode> = Some(ClassCode::new(USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == USB_CLASS_CDC && iface.interface_subclass == CDC_SUBCLASS_ACM
    }
}

/// Find CDC ACM interfaces in a configuration descriptor.
pub fn find_cdc_acm(config_desc: &[u8]) -> Option<CdcAcmInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;

    // Pick the last matching communication interface, preserving the behavior
    // of the original scan when a device exposes several CDC ACM functions.
    let comm_interface = cfg.find_interfaces::<CdcAcmMatcher>().last()?.interface_number;

    let mut data_iface: Option<u8> = None;
    let mut bulk_in: Option<(u8, u16)> = None;
    let mut bulk_out: Option<(u8, u16)> = None;

    for iface in cfg.iter_interface() {
        if iface.interface_class == USB_CLASS_CDC_DATA {
            data_iface = Some(iface.interface_number);
            for ep in iface.iter_endpoints() {
                if ep.transfer_type() == 0x02 {
                    // Bulk
                    if ep.is_in() {
                        bulk_in = Some((ep.endpoint_address, ep.max_packet_size));
                    } else {
                        bulk_out = Some((ep.endpoint_address, ep.max_packet_size));
                    }
                }
            }
        }
    }

    let (in_ep, in_mps) = bulk_in?;
    let (out_ep, out_mps) = bulk_out?;

    Some(CdcAcmInfo {
        comm_interface,
        data_interface: data_iface?,
        bulk_in_ep: in_ep,
        bulk_in_mps: in_mps,
        bulk_out_ep: out_ep,
        bulk_out_mps: out_mps,
    })
}

/// CDC ACM host driver.
///
/// Provides read/write access to a CDC ACM (virtual serial port) USB device.
pub struct CdcAcmHost<'d, A: UsbHostAllocator<'d>> {
    ctrl_ch: A::Pipe<pipe::Control, pipe::InOut>,
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
    comm_interface: u8,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, A: UsbHostAllocator<'d>> CdcAcmHost<'d, A> {
    /// Create a new CDC ACM host driver.
    ///
    /// Parses the config descriptor to find CDC ACM endpoints and allocates channels.
    pub fn new(alloc: &A, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, CdcAcmError> {
        let info = find_cdc_acm(config_desc).ok_or(CdcAcmError::NoInterface)?;
        Self::new_with_info(alloc, info, enum_info)
    }

    /// Create a CDC ACM host driver from an already-probed [`CdcAcmInfo`].
    ///
    /// This is the allocation half of [`HostClass::open`]; it does not parse the
    /// configuration descriptor again.
    pub fn new_with_info(alloc: &A, info: CdcAcmInfo, enum_info: &EnumerationInfo) -> Result<Self, CdcAcmError> {
        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.device_desc.max_packet_size0 as u16,
            interval_ms: 0,
        };

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

        let device_address = enum_info.device_address;
        let split = enum_info.split();

        let ctrl_ch = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| CdcAcmError::NoPipe)?;
        let in_ch = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(device_address, &in_ep_info, split)
            .map_err(|_| CdcAcmError::NoPipe)?;
        let out_ch = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(device_address, &out_ep_info, split)
            .map_err(|_| CdcAcmError::NoPipe)?;

        Ok(Self {
            ctrl_ch,
            in_ch,
            out_ch,
            comm_interface: info.comm_interface,
            _phantom: core::marker::PhantomData,
        })
    }

    /// Set the line coding (baud rate, data bits, parity, stop bits).
    pub async fn set_line_coding(&mut self, coding: &LineCoding) -> Result<(), CdcAcmError> {
        let data = coding.to_bytes();
        let setup =
            SetupPacket::class_interface_out(REQ_SET_LINE_CODING, 0, self.comm_interface as u16, data.len() as u16);
        self.ctrl_ch.control_out(&setup.to_bytes(), &data).await?;
        Ok(())
    }

    /// Set the control line state (DTR, RTS).
    pub async fn set_control_line_state(&mut self, dtr: bool, rts: bool) -> Result<(), CdcAcmError> {
        let value = (dtr as u16) | ((rts as u16) << 1);
        let setup = SetupPacket::class_interface_out(REQ_SET_CONTROL_LINE_STATE, value, self.comm_interface as u16, 0);
        self.ctrl_ch.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    /// Read data from the CDC ACM device.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, CdcAcmError> {
        let n = self.in_ch.request_in(buf).await?;
        Ok(n)
    }

    /// Write data to the CDC ACM device.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, CdcAcmError> {
        self.out_ch.request_out(data, false).await?;
        Ok(data.len())
    }
}

impl<'d, A: UsbHostAllocator<'d>> HostClass<'d, A> for CdcAcmHost<'d, A> {
    const NAME: &'static str = "CDC ACM";
    type Info = CdcAcmInfo;

    fn probe(_enum_info: &EnumerationInfo, cfg: &[u8]) -> Option<Self::Info> {
        find_cdc_acm(cfg)
    }

    async fn open(alloc: &A, _cfg: &[u8], info: Self::Info, enum_info: &EnumerationInfo) -> Result<Self, ClassError> {
        Self::new_with_info(alloc, info, enum_info).map_err(Into::into)
    }
}

impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::ErrorType for CdcAcmHost<'d, A> {
    type Error = CdcAcmError;
}

impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::Read for CdcAcmHost<'d, A> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::read(self, buf).await
    }
}

impl<'d, A: UsbHostAllocator<'d>> embedded_io_async::Write for CdcAcmHost<'d, A> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        // USB bulk transfers are flushed immediately
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    const CONFIG: &[u8] = &[
        // Configuration descriptor (9 bytes, total 56).
        0x09, 0x02, 0x38, 0x00, 0x02, 0x01, 0x00, 0x80, 0x32,
        // Interface Association Descriptor (8 bytes).
        0x08, 0x0b, 0x00, 0x02, 0x02, 0x02, 0x00, 0x00,
        // Interface 0: CDC ACM communication (9 bytes, 1 endpoint).
        0x09, 0x04, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00,
        // Interrupt IN endpoint (7 bytes).
        0x07, 0x05, 0x81, 0x03, 0x08, 0x00, 0x0a,
        // Interface 1: CDC data (9 bytes, 2 endpoints).
        0x09, 0x04, 0x01, 0x00, 0x02, 0x0a, 0x00, 0x00, 0x00,
        // Bulk IN endpoint (7 bytes).
        0x07, 0x05, 0x82, 0x02, 0x40, 0x00, 0x00,
        // Bulk OUT endpoint (7 bytes).
        0x07, 0x05, 0x02, 0x02, 0x40, 0x00, 0x00,
    ];

    #[test]
    fn find_cdc_acm_parses_interfaces_and_endpoints() {
        let info = find_cdc_acm(CONFIG).expect("CDC ACM should be found");
        assert_eq!(info.comm_interface, 0);
        assert_eq!(info.data_interface, 1);
        assert_eq!(info.bulk_in_ep, 0x82);
        assert_eq!(info.bulk_in_mps, 64);
        assert_eq!(info.bulk_out_ep, 0x02);
        assert_eq!(info.bulk_out_mps, 64);
    }

    #[test]
    fn matcher_matches_only_the_communication_interface() {
        let cfg = ConfigurationDescriptorChain::try_from_slice(CONFIG).unwrap();
        let comm = cfg.find_interface::<CdcAcmMatcher>().expect("communication interface");
        assert_eq!(comm.interface_number, 0);
        assert!(CdcAcmMatcher::matches(&comm.descriptor));

        let data = cfg.iter_interface().nth(1).expect("data interface");
        assert!(!CdcAcmMatcher::matches(&data.descriptor));
    }

    #[test]
    fn iad_group_is_found_for_member_interfaces() {
        let cfg = ConfigurationDescriptorChain::try_from_slice(CONFIG).unwrap();
        let iad = cfg.find_iad_group(1).expect("IAD should cover interface 1");
        assert_eq!(iad.first_interface, 0);
        assert_eq!(iad.interface_count, 2);
        assert_eq!(iad.function_class, USB_CLASS_CDC);
    }
}
