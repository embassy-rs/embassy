use core::mem;

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

/// A trait for implementing USB classes.
///
/// All methods are optional callbacks that will be called by
/// [`UsbDevice::run()`](crate::UsbDevice::run)
pub trait ControlHandler {
    /// Called after a USB reset after the bus reset sequence is complete.
    fn reset(&mut self) {}

    /// Called when a control request is received with direction HostToDevice.
    ///
    /// All requests are passed to classes in turn, which can choose to accept, ignore or report an
    /// error. Classes can even choose to override standard requests, but doing that is rarely
    /// necessary.
    ///
    /// When implementing your own class, you should ignore any requests that are not meant for your
    /// class so that any other classes in the composite device can process them.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `data` - The data from the request.
    fn control_out(&mut self, req: Request, data: &[u8]) -> OutResponse {
        OutResponse::Rejected
    }

    /// Called when a control request is received with direction DeviceToHost.
    ///
    /// All requests are passed to classes in turn, which can choose to accept, ignore or report an
    /// error. Classes can even choose to override standard requests, but doing that is rarely
    /// necessary.
    ///
    /// See [`ControlIn`] for how to respond to the transfer.
    ///
    /// When implementing your own class, you should ignore any requests that are not meant for your
    /// class so that any other classes in the composite device can process them.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `control` - The control pipe.
    fn control_in<'a>(&mut self, req: Request, control: ControlIn<'a>) -> InResponse<'a> {
        control.reject()
    }
}

/// Handle for a control IN transfer. When implementing a class, use the methods of this object to
/// response to the transfer with either data or an error (STALL condition). To ignore the request
/// and pass it on to the next class, call [`Self::ignore()`].
pub struct ControlIn<'a> {
    buf: &'a mut [u8],
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InResponse<'a> {
    pub(crate) response: OutResponse,
    pub(crate) data: &'a [u8],
}

impl<'a> InResponse<'a> {
    pub fn status(&self) -> OutResponse {
        self.response
    }
}

impl<'a> ControlIn<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        ControlIn { buf }
    }

    /// Accepts the transfer with the supplied buffer.
    pub fn accept(self, data: &[u8]) -> InResponse<'a> {
        assert!(data.len() < self.buf.len());

        let buf = &mut self.buf[0..data.len()];
        buf.copy_from_slice(data);

        InResponse {
            response: OutResponse::Accepted,
            data: buf,
        }
    }

    /// Rejects the transfer by stalling the pipe.
    pub fn reject(self) -> InResponse<'a> {
        InResponse {
            response: OutResponse::Rejected,
            data: &[],
        }
    }
}
