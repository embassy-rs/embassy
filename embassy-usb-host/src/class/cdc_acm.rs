//! CDC ACM (Serial over USB) host class driver.
//!
//! This driver can communicate with USB CDC ACM devices (virtual serial ports).

use embassy_usb_host_driver::{
    ChannelAllocError, DeviceEndpoint, DeviceSpeed, Direction, EndpointType, HostBus, HostChannel, TransferError,
};

use crate::descriptor::{DescriptorIter, EndpointDescriptor, InterfaceDescriptor, descriptor_type};

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
    Transfer(TransferError),
    /// No matching CDC ACM interface found in the device.
    NoInterface,
    /// Failed to allocate a channel.
    NoChannel,
}

impl From<TransferError> for CdcAcmError {
    fn from(e: TransferError) -> Self {
        Self::Transfer(e)
    }
}

impl From<ChannelAllocError> for CdcAcmError {
    fn from(_: ChannelAllocError) -> Self {
        Self::NoChannel
    }
}

impl core::fmt::Display for CdcAcmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(e) => write!(f, "Transfer error: {}", e),
            Self::NoInterface => write!(f, "No CDC ACM interface found"),
            Self::NoChannel => write!(f, "No free channel"),
        }
    }
}

impl core::error::Error for CdcAcmError {}

impl embedded_io_async::Error for CdcAcmError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Transfer(e) => e.kind(),
            Self::NoInterface => embedded_io_async::ErrorKind::NotFound,
            Self::NoChannel => embedded_io_async::ErrorKind::OutOfMemory,
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
    let mut comm_iface: Option<u8> = None;
    let mut data_iface: Option<u8> = None;
    let mut bulk_in: Option<(u8, u16)> = None;
    let mut bulk_out: Option<(u8, u16)> = None;
    let mut current_iface_class: u8 = 0;

    for (desc_type, desc_data) in DescriptorIter::new(config_desc) {
        match desc_type {
            descriptor_type::INTERFACE => {
                if let Some(iface) = InterfaceDescriptor::parse(desc_data) {
                    current_iface_class = iface.interface_class;

                    if iface.interface_class == USB_CLASS_CDC && iface.interface_subclass == CDC_SUBCLASS_ACM {
                        comm_iface = Some(iface.interface_number);
                    } else if iface.interface_class == USB_CLASS_CDC_DATA {
                        data_iface = Some(iface.interface_number);
                    }
                }
            }
            descriptor_type::ENDPOINT => {
                if current_iface_class == USB_CLASS_CDC_DATA {
                    if let Some(ep) = EndpointDescriptor::parse(desc_data) {
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
            _ => {}
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
pub struct CdcAcmHost<B: HostBus> {
    ctrl_ch: B::Channel,
    in_ch: B::Channel,
    out_ch: B::Channel,
    comm_interface: u8,
}

impl<B: HostBus> CdcAcmHost<B> {
    /// Create a new CDC ACM host driver.
    ///
    /// Parses the config descriptor to find CDC ACM endpoints and allocates channels.
    pub fn new(bus: &B, config_desc: &[u8], device_address: u8, speed: DeviceSpeed) -> Result<Self, CdcAcmError> {
        let info = find_cdc_acm(config_desc).ok_or(CdcAcmError::NoInterface)?;

        let ctrl_ep = DeviceEndpoint {
            device_address,
            ep_number: 0,
            direction: Direction::In,
            ep_type: EndpointType::Control,
            max_packet_size: 64, // Assume 64 for FS
            speed,
        };

        let in_ep = DeviceEndpoint {
            device_address,
            ep_number: info.bulk_in_ep & 0x0F,
            direction: Direction::In,
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            speed,
        };

        let out_ep = DeviceEndpoint {
            device_address,
            ep_number: info.bulk_out_ep & 0x0F,
            direction: Direction::Out,
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            speed,
        };

        let ctrl_ch = bus.alloc_channel(&ctrl_ep)?;
        let in_ch = bus.alloc_channel(&in_ep)?;
        let out_ch = bus.alloc_channel(&out_ep)?;

        Ok(Self {
            ctrl_ch,
            in_ch,
            out_ch,
            comm_interface: info.comm_interface,
        })
    }

    /// Set the line coding (baud rate, data bits, parity, stop bits).
    pub async fn set_line_coding(&mut self, coding: &LineCoding) -> Result<(), CdcAcmError> {
        let mut data = coding.to_bytes();
        let setup =
            crate::control::class_interface_out_with_data(REQ_SET_LINE_CODING, 0, self.comm_interface as u16, 7);
        self.ctrl_ch.control_transfer(&setup, Direction::Out, &mut data).await?;
        Ok(())
    }

    /// Set the control line state (DTR, RTS).
    pub async fn set_control_line_state(&mut self, dtr: bool, rts: bool) -> Result<(), CdcAcmError> {
        let value = (dtr as u16) | ((rts as u16) << 1);
        let setup = crate::control::class_interface_out(REQ_SET_CONTROL_LINE_STATE, value, self.comm_interface as u16);
        self.ctrl_ch.control_transfer(&setup, Direction::Out, &mut []).await?;
        Ok(())
    }

    /// Read data from the CDC ACM device.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, CdcAcmError> {
        let n = self.in_ch.in_transfer(buf).await?;
        Ok(n)
    }

    /// Write data to the CDC ACM device.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, CdcAcmError> {
        let n = self.out_ch.out_transfer(data).await?;
        Ok(n)
    }
}

impl<B: HostBus> embedded_io_async::ErrorType for CdcAcmHost<B> {
    type Error = CdcAcmError;
}

impl<B: HostBus> embedded_io_async::Read for CdcAcmHost<B> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::read(self, buf).await
    }
}

impl<B: HostBus> embedded_io_async::Write for CdcAcmHost<B> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        CdcAcmHost::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        // USB bulk transfers are flushed immediately
        Ok(())
    }
}
