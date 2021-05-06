use core::future::Future;

/// Random-number Generator
pub trait Rng {
    type Error;

    type RngFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Completely fill the provided buffer with random bytes.
    ///
    /// May result in delays if entropy is exhausted prior to completely
    /// filling the buffer. Upon completion, the buffer will be completely
    /// filled or an error will have been reported.
    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a>;
}
