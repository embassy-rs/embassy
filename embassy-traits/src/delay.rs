use core::future::Future;

pub trait Delay {
    type DelayFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// Future that completes after now + millis
    fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture<'_>;

    /// Future that completes after now + micros
    fn delay_us(&mut self, micros: u64) -> Self::DelayFuture<'_>;
}
