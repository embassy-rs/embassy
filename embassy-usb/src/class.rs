use core::future::Future;

use crate::control::Request;
use crate::driver::{ControlPipe, Driver};

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
pub trait UsbClass<'d, D: Driver<'d>> {
    type ControlOutFuture<'a>: Future<Output = RequestStatus> + 'a
    where
        Self: 'a,
        'd: 'a,
        D: 'a;

    type ControlInFuture<'a>: Future<Output = ControlInRequestStatus> + 'a
    where
        Self: 'a,
        'd: 'a,
        D: 'a;

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
    fn control_out<'a>(&'a mut self, req: Request, data: &'a [u8]) -> Self::ControlOutFuture<'a>
    where
        'd: 'a,
        D: 'a;

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
        &'a mut self,
        req: Request,
        control: ControlIn<'a, 'd, D>,
    ) -> Self::ControlInFuture<'a>
    where
        'd: 'a;
}

impl<'d, D: Driver<'d>> UsbClass<'d, D> for () {
    type ControlOutFuture<'a> = impl Future<Output = RequestStatus> + 'a where Self: 'a, 'd: 'a, D: 'a;
    type ControlInFuture<'a> = impl Future<Output = ControlInRequestStatus> + 'a where Self: 'a, 'd: 'a, D: 'a;

    fn control_out<'a>(&'a mut self, _req: Request, _data: &'a [u8]) -> Self::ControlOutFuture<'a>
    where
        'd: 'a,
        D: 'a,
    {
        async move { RequestStatus::default() }
    }

    fn control_in<'a>(
        &'a mut self,
        _req: Request,
        control: ControlIn<'a, 'd, D>,
    ) -> Self::ControlInFuture<'a>
    where
        'd: 'a,
        D: 'a,
    {
        async move { control.ignore() }
    }
}

impl<'d, D: Driver<'d>, Head, Tail> UsbClass<'d, D> for (Head, Tail)
where
    Head: UsbClass<'d, D>,
    Tail: UsbClass<'d, D>,
{
    type ControlOutFuture<'a> = impl Future<Output = RequestStatus> + 'a where Self: 'a, 'd: 'a, D: 'a;
    type ControlInFuture<'a> = impl Future<Output = ControlInRequestStatus> + 'a where Self: 'a, 'd: 'a, D: 'a;

    fn control_out<'a>(&'a mut self, req: Request, data: &'a [u8]) -> Self::ControlOutFuture<'a>
    where
        'd: 'a,
        D: 'a,
    {
        async move {
            match self.0.control_out(req, data).await {
                RequestStatus::Unhandled => self.1.control_out(req, data).await,
                status => status,
            }
        }
    }

    fn control_in<'a>(
        &'a mut self,
        req: Request,
        control: ControlIn<'a, 'd, D>,
    ) -> Self::ControlInFuture<'a>
    where
        'd: 'a,
    {
        async move {
            match self
                .0
                .control_in(req, ControlIn::new(control.control))
                .await
            {
                ControlInRequestStatus(RequestStatus::Unhandled) => {
                    self.1.control_in(req, control).await
                }
                status => status,
            }
        }
    }
}

/// Handle for a control IN transfer. When implementing a class, use the methods of this object to
/// response to the transfer with either data or an error (STALL condition). To ignore the request
/// and pass it on to the next class, call [`Self::ignore()`].
pub struct ControlIn<'a, 'd: 'a, D: Driver<'d>> {
    control: &'a mut D::ControlPipe,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControlInRequestStatus(pub(crate) RequestStatus);

impl ControlInRequestStatus {
    pub fn status(self) -> RequestStatus {
        self.0
    }
}

impl<'a, 'd: 'a, D: Driver<'d>> ControlIn<'a, 'd, D> {
    pub(crate) fn new(control: &'a mut D::ControlPipe) -> Self {
        ControlIn { control }
    }

    /// Ignores the request and leaves it unhandled.
    pub fn ignore(self) -> ControlInRequestStatus {
        ControlInRequestStatus(RequestStatus::Unhandled)
    }

    /// Accepts the transfer with the supplied buffer.
    pub async fn accept(self, data: &[u8]) -> ControlInRequestStatus {
        self.control.accept_in(data).await;

        ControlInRequestStatus(RequestStatus::Accepted)
    }

    /// Rejects the transfer by stalling the pipe.
    pub fn reject(self) -> ControlInRequestStatus {
        self.control.reject();
        ControlInRequestStatus(RequestStatus::Rejected)
    }
}
