use core::future::Future;

pub trait Delay {
    type DelayFuture<'a>: Future<Output = ()> + 'a;

    /// Future that completes after now + millis
    fn delay_ms<'a>(&'a mut self, millis: u64) -> Self::DelayFuture<'a>;

    /// Future that completes after now + micros
    fn delay_us<'a>(&'a mut self, micros: u64) -> Self::DelayFuture<'a>;
}
