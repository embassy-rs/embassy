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
    fn receive<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReceiveFuture<'a>;
    fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a>;
}
