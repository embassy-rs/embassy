//! CDC ACM (Serial over USB) host class driver.
//!
//! This driver can communicate with USB CDC ACM devices (virtual serial ports).

use embassy_usb_driver::host::{PipeError, UsbHostDriver, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::control::SetupPacket;
use crate::descriptor::ConfigurationDescriptor;
use crate::handler::EnumerationInfo;

/// CDC class code.
const USB_CLASS_CDC: u8 = 0x02;
/// CDC Data class code.
const USB_CLASS_CDC_DATA: u8 = 0x0A;
/// CDC ACM subclass.
const CDC_SUBCLASS_ACM: u8 = 0x02;

/// CDC ACM class request: SET_LINE_CODING.
const REQ_SET_LINE_CODING: u8 = 0x20;
/// CDC ACM class request: SET_CONTROL_LINE_STATE.
const REQ_SET_CONTROL_LINE_STATE: u8 = 0x22;

/// USB line coding (serial parameters).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LineCoding {
    /// Baud rate in bits per second.
    pub baud_rate: u32,
    /// Stop bits: 0=1, 1=1.5, 2=2.
    pub stop_bits: u8,
    /// Parity: 0=None, 1=Odd, 2=Even.
    pub parity: u8,
    /// Data bits (5, 6, 7, 8).
    pub data_bits: u8,
}

impl Default for LineCoding {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            stop_bits: 0,
            parity: 0,
            data_bits: 8,
        }
    }
}

impl LineCoding {
    fn to_bytes(&self) -> [u8; 7] {
        let baud = self.baud_rate.to_le_bytes();
        [
            baud[0],
            baud[1],
            baud[2],
            baud[3],
            self.stop_bits,
            self.parity,
            self.data_bits,
        ]
    }
}

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
#[derive(Clone, Debug)]
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

/// Find CDC ACM interfaces in a configuration descriptor.
pub fn find_cdc_acm(config_desc: &[u8]) -> Option<CdcAcmInfo> {
    let cfg = ConfigurationDescriptor::try_from_slice(config_desc).ok()?;

    let mut comm_iface: Option<u8> = None;
    let mut data_iface: Option<u8> = None;
    let mut bulk_in: Option<(u8, u16)> = None;
    let mut bulk_out: Option<(u8, u16)> = None;

    for iface in cfg.iter_interface() {
        if iface.interface_class == USB_CLASS_CDC && iface.interface_subclass == CDC_SUBCLASS_ACM {
            comm_iface = Some(iface.interface_number);
        } else if iface.interface_class == USB_CLASS_CDC_DATA {
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

    if let (Some(comm), Some(data), Some((in_ep, in_mps)), Some((out_ep, out_mps))) =
        (comm_iface, data_iface, bulk_in, bulk_out)
    {
        Some(CdcAcmInfo {
            comm_interface: comm,
            data_interface: data,
            bulk_in_ep: in_ep,
            bulk_in_mps: in_mps,
            bulk_out_ep: out_ep,
            bulk_out_mps: out_mps,
        })
    } else {
        None
    }
}

/// CDC ACM host driver.
///
/// Provides read/write access to a CDC ACM (virtual serial port) USB device.
pub struct CdcAcmHost<'d, D: UsbHostDriver<'d>> {
    ctrl_ch: D::Pipe<pipe::Control, pipe::InOut>,
    in_ch: D::Pipe<pipe::Bulk, pipe::In>,
    out_ch: D::Pipe<pipe::Bulk, pipe::Out>,
    comm_interface: u8,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, D: UsbHostDriver<'d>> CdcAcmHost<'d, D> {
    /// Create a new CDC ACM host driver.
    ///
    /// Parses the config descriptor to find CDC ACM endpoints and allocates channels.
    pub fn new(driver: &D, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, CdcAcmError> {
        let info = find_cdc_acm(config_desc).ok_or(CdcAcmError::NoInterface)?;

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

        let ctrl_ch = driver
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| CdcAcmError::NoPipe)?;
        let in_ch = driver
            .alloc_pipe::<pipe::Bulk, pipe::In>(device_address, &in_ep_info, split)
            .map_err(|_| CdcAcmError::NoPipe)?;
        let out_ch = driver
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

impl<'d, D: UsbHostDriver<'d>> embedded_io_async::ErrorType for CdcAcmHost<'d, D> {
    type Error = CdcAcmError;
}

impl<'d, D: UsbHostDriver<'d>> embedded_io_async::Read for CdcAcmHost<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::read(self, buf).await
    }
}

impl<'d, D: UsbHostDriver<'d>> embedded_io_async::Write for CdcAcmHost<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        // USB bulk transfers are flushed immediately
        Ok(())
    }
}
