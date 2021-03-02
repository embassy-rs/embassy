use core::future::Future;
use core::pin::Pin;

pub trait Delay {
    type DelayFuture<'a>: Future<Output = ()> + 'a;

    fn delay_ms<'a>(self: Pin<&'a mut Self>, millis: u64) -> Self::DelayFuture<'a>;
    fn delay_us<'a>(self: Pin<&'a mut Self>, micros: u64) -> Self::DelayFuture<'a>;
}
