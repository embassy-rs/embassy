//! USB Printer Class 1.1 Device implementation.
//!
//! Exposes an embedded system as a USB printer (such as a thermal receipt printer,
//! label printer, or virtual print logger). Print data sent from the host is read
//! via [`embedded_io_async::Read`] or [`PrinterClass::read`].

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_usb_driver::{Driver, EndpointError, EndpointIn, EndpointOut};

use super::{
    PRINTER_SUBCLASS_PRINTER, PROTOCOL_BIDIRECTIONAL, PortStatus, REQ_GET_DEVICE_ID, REQ_GET_PORT_STATUS,
    REQ_SOFT_RESET, USB_CLASS_PRINTER,
};
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler, InterfaceHandler};

/// Configuration options for the USB printer device.
pub struct Config<'a> {
    /// IEEE 1284 Device ID string (e.g. `"MFG:Generic;MDL:ThermalPrinter;CMD:ESC/POS;CLS:PRINTER;"`).
    pub device_id: Option<&'a str>,
    /// Initial port status byte.
    pub initial_status: PortStatus,
    /// Maximum packet size for bulk endpoints (typically 64 for full speed, 512 for high speed).
    pub max_packet_size: u16,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            device_id: None,
            initial_status: PortStatus::default(),
            max_packet_size: 64,
        }
    }
}

/// State for the USB printer device class.
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    shared: ControlShared,
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared {
                status: AtomicU8::new(0x18),
                reset_notified: AtomicBool::new(false),
            },
        }
    }
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

struct ControlShared {
    status: AtomicU8,
    reset_notified: AtomicBool,
}

struct Control<'a> {
    comm_if: InterfaceNumber,
    device_id: Option<&'a str>,
    shared: &'a ControlShared,
}

impl<'d> Handler for Control<'d> {
    fn control_out(&mut self, req: Request, _data: &[u8]) -> Option<OutResponse> {
        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            return None;
        }

        let iface_idx = self.comm_if.0 as u16;
        if req.index != iface_idx && (req.index >> 8) != iface_idx {
            return None;
        }

        match req.request {
            REQ_SOFT_RESET => {
                self.shared.reset_notified.store(true, Ordering::Relaxed);
                Some(OutResponse::Accepted)
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            return None;
        }

        let iface_idx = self.comm_if.0 as u16;
        if (req.index & 0xFF) != iface_idx && (req.index >> 8) != iface_idx {
            return None;
        }

        match req.request {
            REQ_GET_DEVICE_ID => {
                let id_str = self.device_id.unwrap_or("");
                let total_len = (id_str.len() + 2) as u16;
                let copy_len = (buf.len()).min(total_len as usize).min(req.length as usize);

                if copy_len >= 2 {
                    buf[0] = (total_len >> 8) as u8;
                    buf[1] = (total_len & 0xFF) as u8;
                    let str_bytes = id_str.as_bytes();
                    let str_len = (copy_len - 2).min(str_bytes.len());
                    buf[2..2 + str_len].copy_from_slice(&str_bytes[..str_len]);
                } else if copy_len == 1 {
                    buf[0] = (total_len >> 8) as u8;
                }

                Some(InResponse::Accepted(&buf[..copy_len]))
            }
            REQ_GET_PORT_STATUS => {
                if buf.is_empty() {
                    return Some(InResponse::Rejected);
                }
                buf[0] = self.shared.status.load(Ordering::Relaxed);
                Some(InResponse::Accepted(&buf[..1]))
            }
            _ => Some(InResponse::Rejected),
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.comm_if
    }
}

/// USB Printer Device class.
pub struct PrinterClass<'d, D: Driver<'d>> {
    _comm_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    shared: &'d ControlShared,
}

impl<'d, D: Driver<'d>> PrinterClass<'d, D> {
    /// Create a new USB Printer device class.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        state
            .shared
            .status
            .store(config.initial_status.bits(), Ordering::Relaxed);
        state.shared.reset_notified.store(false, Ordering::Relaxed);

        let mut func = builder.function(USB_CLASS_PRINTER, PRINTER_SUBCLASS_PRINTER, PROTOCOL_BIDIRECTIONAL);
        let mut iface = func.interface();
        let comm_if = iface.interface_number();
        let mut alt = iface.alt_setting(
            USB_CLASS_PRINTER,
            PRINTER_SUBCLASS_PRINTER,
            PROTOCOL_BIDIRECTIONAL,
            None,
        );

        let read_ep = alt.endpoint_bulk_out(None, config.max_packet_size);
        let write_ep = alt.endpoint_bulk_in(None, config.max_packet_size);

        drop(func);

        let control = state.control.write(Control {
            comm_if,
            device_id: config.device_id,
            shared: &state.shared,
        });
        builder.interface_handler(control);

        Self {
            _comm_if: comm_if,
            read_ep,
            write_ep,
            shared: &state.shared,
        }
    }

    /// Read incoming print job data sent by the USB host.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(buf).await
    }

    /// Write status or response data back to the USB host.
    pub async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(buf, true).await
    }

    /// Update the printer port status returned by `GET_PORT_STATUS`.
    pub fn set_port_status(&self, status: PortStatus) {
        self.shared.status.store(status.bits(), Ordering::Relaxed);
    }

    /// Check if the host issued a `SOFT_RESET` request, clearing the flag.
    pub fn check_soft_reset(&self) -> bool {
        // `swap` requires atomic CAS, which isn't available on all supported
        // targets (e.g. thumbv6m), so use load/store like the other classes do.
        if self.shared.reset_notified.load(Ordering::Relaxed) {
            self.shared.reset_notified.store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Split into independent reader and writer halves.
    pub fn split(self) -> (PrinterReader<'d, D>, PrinterWriter<'d, D>) {
        (
            PrinterReader { read_ep: self.read_ep },
            PrinterWriter {
                write_ep: self.write_ep,
            },
        )
    }
}

/// Printer reader half (receives print data from host).
pub struct PrinterReader<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> PrinterReader<'d, D> {
    /// Read incoming print job data sent by the USB host.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(buf).await
    }
}

/// Printer writer half (sends responses back to host).
pub struct PrinterWriter<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> PrinterWriter<'d, D> {
    /// Write status or response data back to the USB host.
    pub async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(buf, true).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for PrinterClass<'d, D> {
    type Error = EndpointError;
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Read for PrinterClass<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        PrinterClass::read(self, buf).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Write for PrinterClass<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        PrinterClass::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for PrinterReader<'d, D> {
    type Error = EndpointError;
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Read for PrinterReader<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        PrinterReader::read(self, buf).await
    }
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for PrinterWriter<'d, D> {
    type Error = EndpointError;
}

#[cfg(feature = "embedded-io")]
impl<'d, D: Driver<'d>> embedded_io_async::Write for PrinterWriter<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        PrinterWriter::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
