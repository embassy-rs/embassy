use crate::control::Request;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RequestStatus {
    Unhandled,
    Accepted,
    Rejected,
}

impl Default for RequestStatus {
    fn default() -> Self {
        RequestStatus::Unhandled
    }
}

/// A trait for implementing USB classes.
///
/// All methods are optional callbacks that will be called by
/// [`UsbDevice::run()`](crate::UsbDevice::run)
pub trait UsbClass {
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
    fn control_out(&mut self, req: Request, data: &[u8]) -> RequestStatus;

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
    fn control_in<'a>(
        &mut self,
        req: Request,
        control: ControlIn<'a>,
    ) -> ControlInRequestStatus<'a>;
}

/// Handle for a control IN transfer. When implementing a class, use the methods of this object to
/// response to the transfer with either data or an error (STALL condition). To ignore the request
/// and pass it on to the next class, call [`Self::ignore()`].
pub struct ControlIn<'a> {
    buf: &'a mut [u8],
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControlInRequestStatus<'a> {
    pub(crate) status: RequestStatus,
    pub(crate) data: &'a [u8],
}

impl<'a> ControlInRequestStatus<'a> {
    pub fn status(&self) -> RequestStatus {
        self.status
    }
}

impl<'a> ControlIn<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        ControlIn { buf }
    }

    /// Ignores the request and leaves it unhandled.
    pub fn ignore(self) -> ControlInRequestStatus<'a> {
        ControlInRequestStatus {
            status: RequestStatus::Unhandled,
            data: &[],
        }
    }

    /// Accepts the transfer with the supplied buffer.
    pub fn accept(self, data: &[u8]) -> ControlInRequestStatus<'a> {
        assert!(data.len() < self.buf.len());

        let buf = &mut self.buf[0..data.len()];
        buf.copy_from_slice(data);

        ControlInRequestStatus {
            status: RequestStatus::Accepted,
            data: buf,
        }
    }

    /// Rejects the transfer by stalling the pipe.
    pub fn reject(self) -> ControlInRequestStatus<'a> {
        ControlInRequestStatus {
            status: RequestStatus::Unhandled,
            data: &[],
        }
    }
}
