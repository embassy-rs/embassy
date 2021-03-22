use core::future::Future;
use core::pin::Pin;

pub trait Delay {
    type DelayFuture<'a>: Future<Output = ()> + 'a;

    /// Future that completes after now + millis
    fn delay_ms<'a>(self: Pin<&'a mut Self>, millis: u64) -> Self::DelayFuture<'a>;

    /// Future that completes after now + micros
    fn delay_us<'a>(self: Pin<&'a mut Self>, micros: u64) -> Self::DelayFuture<'a>;
}
