use core::future::Future;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    Other,
}

pub trait Uart {
    type ReceiveFuture<'a>: Future<Output = Result<(), Error>>;
    type SendFuture<'a>: Future<Output = Result<(), Error>>;
    /// Receive into the buffer until the buffer is full
    fn receive<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReceiveFuture<'a>;
    /// Send the specified buffer, and return when the transmission has completed
    fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a>;
}

pub trait IdleUart {
    type ReceiveFuture<'a>: Future<Output = Result<usize, Error>>;
    /// Receive into the buffer until the buffer is full or the line is idle after some bytes are received
    /// Return the number of bytes received
    fn receive_until_idle<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReceiveFuture<'a>;
}
