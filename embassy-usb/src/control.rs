use core::mem;

use crate::descriptor::DescriptorWriter;
use crate::driver::{self, EndpointError};

use super::types::*;

/// Control request type.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RequestType {
    /// Request is a USB standard request. Usually handled by
    /// [`UsbDevice`](crate::device::UsbDevice).
    Standard = 0,
    /// Request is intended for a USB class.
    Class = 1,
    /// Request is vendor-specific.
    Vendor = 2,
    /// Reserved.
    Reserved = 3,
}

/// Control request recipient.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Recipient {
    /// Request is intended for the entire device.
    Device = 0,
    /// Request is intended for an interface. Generally, the `index` field of the request specifies
    /// the interface number.
    Interface = 1,
    /// Request is intended for an endpoint. Generally, the `index` field of the request specifies
    /// the endpoint address.
    Endpoint = 2,
    /// None of the above.
    Other = 3,
    /// Reserved.
    Reserved = 4,
}

/// A control request read from a SETUP packet.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Request {
    /// Direction of the request.
    pub direction: UsbDirection,
    /// Type of the request.
    pub request_type: RequestType,
    /// Recipient of the request.
    pub recipient: Recipient,
    /// Request code. The meaning of the value depends on the previous fields.
    pub request: u8,
    /// Request value. The meaning of the value depends on the previous fields.
    pub value: u16,
    /// Request index. The meaning of the value depends on the previous fields.
    pub index: u16,
    /// Length of the DATA stage. For control OUT transfers this is the exact length of the data the
    /// host sent. For control IN transfers this is the maximum length of data the device should
    /// return.
    pub length: u16,
}

impl Request {
    /// Standard USB control request Get Status
    pub const GET_STATUS: u8 = 0;

    /// Standard USB control request Clear Feature
    pub const CLEAR_FEATURE: u8 = 1;

    /// Standard USB control request Set Feature
    pub const SET_FEATURE: u8 = 3;

    /// Standard USB control request Set Address
    pub const SET_ADDRESS: u8 = 5;

    /// Standard USB control request Get Descriptor
    pub const GET_DESCRIPTOR: u8 = 6;

    /// Standard USB control request Set Descriptor
    pub const SET_DESCRIPTOR: u8 = 7;

    /// Standard USB control request Get Configuration
    pub const GET_CONFIGURATION: u8 = 8;

    /// Standard USB control request Set Configuration
    pub const SET_CONFIGURATION: u8 = 9;

    /// Standard USB control request Get Interface
    pub const GET_INTERFACE: u8 = 10;

    /// Standard USB control request Set Interface
    pub const SET_INTERFACE: u8 = 11;

    /// Standard USB control request Synch Frame
    pub const SYNCH_FRAME: u8 = 12;

    /// Standard USB feature Endpoint Halt for Set/Clear Feature
    pub const FEATURE_ENDPOINT_HALT: u16 = 0;

    /// Standard USB feature Device Remote Wakeup for Set/Clear Feature
    pub const FEATURE_DEVICE_REMOTE_WAKEUP: u16 = 1;

    /// Parses a USB control request from a byte array.
    pub fn parse(buf: &[u8; 8]) -> Request {
        let rt = buf[0];
        let recipient = rt & 0b11111;

        Request {
            direction: rt.into(),
            request_type: unsafe { mem::transmute((rt >> 5) & 0b11) },
            recipient: if recipient <= 3 {
                unsafe { mem::transmute(recipient) }
            } else {
                Recipient::Reserved
            },
            request: buf[1],
            value: (buf[2] as u16) | ((buf[3] as u16) << 8),
            index: (buf[4] as u16) | ((buf[5] as u16) << 8),
            length: (buf[6] as u16) | ((buf[7] as u16) << 8),
        }
    }

    /// Gets the descriptor type and index from the value field of a GET_DESCRIPTOR request.
    pub fn descriptor_type_index(&self) -> (u8, u8) {
        ((self.value >> 8) as u8, self.value as u8)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutResponse {
    Accepted,
    Rejected,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InResponse<'a> {
    Accepted(&'a [u8]),
    Rejected,
}

/// Handler for control requests.
///
/// All methods are optional callbacks that will be called by
/// [`UsbDevice::run()`](crate::UsbDevice::run)
pub trait ControlHandler {
    /// Called after a USB reset after the bus reset sequence is complete.
    fn reset(&mut self) {}

    fn set_alternate_setting(&mut self, alternate_setting: u8) {
        let _ = alternate_setting;
    }

    /// Called when a control request is received with direction HostToDevice.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `data` - The data from the request.
    fn control_out(&mut self, req: Request, data: &[u8]) -> OutResponse {
        let _ = (req, data);
        OutResponse::Rejected
    }

    /// Called when a control request is received with direction DeviceToHost.
    ///
    /// You should write the response somewhere (usually to `buf`, but you may use another buffer
    /// owned by yourself, or a static buffer), then return `InResponse::Accepted(data)`.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        let _ = (req, buf);
        InResponse::Rejected
    }

    /// Called when a GET DESCRIPTOR control request is received on the interface.
    ///
    /// You should write the response somewhere (usually to `buf`, but you may use another buffer
    /// owned by yourself, or a static buffer), then return `InResponse::Accepted(data)`.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    fn get_descriptor<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        let _ = (req, buf);
        InResponse::Rejected
    }

    /// Called when a GET_DESCRIPTOR STRING control request is received.
    ///
    /// Write the response string somewhere (usually to `buf`, but you may use another buffer
    /// owned by yourself, or a static buffer), then return it.
    fn get_string<'a>(
        &'a mut self,
        index: StringIndex,
        lang_id: u16,
        buf: &'a mut [u8],
    ) -> Option<&'a str> {
        let _ = (index, lang_id, buf);
        None
    }
}

/// Typestate representing a ControlPipe in the DATA IN stage
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct DataInStage {
    pub(crate) length: usize,
}

/// Typestate representing a ControlPipe in the DATA OUT stage
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct DataOutStage {
    length: usize,
}

/// Typestate representing a ControlPipe in the STATUS stage
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct StatusStage {}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum Setup {
    DataIn(Request, DataInStage),
    DataOut(Request, DataOutStage),
}

pub(crate) struct ControlPipe<C: driver::ControlPipe> {
    control: C,
}

impl<C: driver::ControlPipe> ControlPipe<C> {
    pub(crate) fn new(control: C) -> Self {
        ControlPipe { control }
    }

    pub(crate) async fn setup(&mut self) -> Setup {
        let req = self.control.setup().await;
        trace!("control request: {:02x}", req);

        match (req.direction, req.length) {
            (UsbDirection::Out, n) => Setup::DataOut(
                req,
                DataOutStage {
                    length: usize::from(n),
                },
            ),
            (UsbDirection::In, n) => Setup::DataIn(
                req,
                DataInStage {
                    length: usize::from(n),
                },
            ),
        }
    }

    pub(crate) async fn data_out<'a>(
        &mut self,
        buf: &'a mut [u8],
        stage: DataOutStage,
    ) -> Result<(&'a [u8], StatusStage), EndpointError> {
        if stage.length == 0 {
            Ok((&[], StatusStage {}))
        } else {
            let req_length = stage.length;
            let max_packet_size = self.control.max_packet_size();
            let mut total = 0;

            for chunk in buf.chunks_mut(max_packet_size) {
                let size = self.control.data_out(chunk).await?;
                total += size;
                if size < max_packet_size || total == req_length {
                    break;
                }
            }

            let res = &buf[0..total];
            #[cfg(feature = "defmt")]
            trace!("  control out data: {:02x}", res);
            #[cfg(not(feature = "defmt"))]
            trace!("  control out data: {:02x?}", res);

            Ok((res, StatusStage {}))
        }
    }

    pub(crate) async fn accept_in(&mut self, buf: &[u8], stage: DataInStage) {
        #[cfg(feature = "defmt")]
        trace!("  control in accept {:02x}", buf);
        #[cfg(not(feature = "defmt"))]
        trace!("  control in accept {:02x?}", buf);

        let req_len = stage.length;
        let len = buf.len().min(req_len);
        let max_packet_size = self.control.max_packet_size();
        let need_zlp = len != req_len && (len % usize::from(max_packet_size)) == 0;

        let mut chunks = buf[0..len]
            .chunks(max_packet_size)
            .chain(need_zlp.then(|| -> &[u8] { &[] }));

        while let Some(chunk) = chunks.next() {
            match self.control.data_in(chunk, chunks.size_hint().0 == 0).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("control accept_in failed: {:?}", e);
                    return;
                }
            }
        }
    }

    pub(crate) async fn accept_in_writer(
        &mut self,
        req: Request,
        stage: DataInStage,
        f: impl FnOnce(&mut DescriptorWriter),
    ) {
        let mut buf = [0; 256];
        let mut w = DescriptorWriter::new(&mut buf);
        f(&mut w);
        let pos = w.position().min(usize::from(req.length));
        self.accept_in(&buf[..pos], stage).await
    }

    pub(crate) fn accept(&mut self, _: StatusStage) {
        trace!("  control accept");
        self.control.accept();
    }

    pub(crate) fn reject(&mut self) {
        trace!("  control reject");
        self.control.reject();
    }
}
