use core::future::Future;

/// Random-number Generator
pub trait Rng {
    type Error;

    #[rustfmt::skip]
    type RngFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Completely fill the provided buffer with random bytes.
    ///
    /// May result in delays if entropy is exhausted prior to completely
    /// filling the buffer. Upon completion, the buffer will be completely
    /// filled or an error will have been reported.
    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a>;

    #[rustfmt::skip]
    type NextFuture<'a>: Future<Output = Result<u32, Self::Error>> + 'a
    where
        Self: 'a;

    fn next<'a>(&'a mut self, range: u32) -> Self::NextFuture<'a>;
}
