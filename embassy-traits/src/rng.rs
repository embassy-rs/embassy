use core::future::Future;

/// Random-number Generator
pub trait Rng {
    type Error;

    #[rustfmt::skip]
    type RngFuture<'a>: Future<Output = Result<(), Self::Error> > + 'a
    where
    Self: 'a;

    /// Completely fill the provided buffer with random bytes.
    ///
    /// May result in delays if entropy is exhausted prior to completely
    /// filling the buffer. Upon completion, the buffer will be completely
    /// filled or an error will have been reported.
    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a>;
}

pub struct Random<T: Rng> {
    rng: T,
}

impl<T: Rng> Random<T> {
    pub fn new(rng: T) -> Self {
        Self { rng }
    }

    pub async fn next_u8<'a>(&'a mut self, range: u8) -> Result<u8, T::Error> {
        // Lemire's method
        let t = (-(range as i8) % (range as i8)) as u8;
        loop {
            let mut buf = [0; 1];
            self.rng.fill_bytes(&mut buf).await?;
            let x = u8::from_le_bytes(buf);
            let m = x as u16 * range as u16;
            let l = m as u8;
            if l < t {
                continue;
            }
            return Ok((m >> 8) as u8);
        }
    }

    pub async fn next_u16<'a>(&'a mut self, range: u16) -> Result<u16, T::Error> {
        // Lemire's method
        let t = (-(range as i16) % (range as i16)) as u16;
        loop {
            let mut buf = [0; 2];
            self.rng.fill_bytes(&mut buf).await?;
            let x = u16::from_le_bytes(buf);
            let m = x as u32 * range as u32;
            let l = m as u16;
            if l < t {
                continue;
            }
            return Ok((m >> 16) as u16);
        }
    }

    pub async fn next_u32<'a>(&'a mut self, range: u32) -> Result<u32, T::Error> {
        // Lemire's method
        let t = (-(range as i32) % (range as i32)) as u32;
        loop {
            let mut buf = [0; 4];
            self.rng.fill_bytes(&mut buf).await?;
            let x = u32::from_le_bytes(buf);
            let m = x as u64 * range as u64;
            let l = m as u32;
            if l < t {
                continue;
            }
            return Ok((m >> 32) as u32);
        }
    }
}
