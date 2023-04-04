#![allow(unused)]

use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

use crate::Flash;

pub struct LargeErase<F, const ERASE_SIZE: usize>(pub F);

impl<F, const ERASE_SIZE: usize> LargeErase<F, ERASE_SIZE> {
    pub const fn new(flash: F) -> Self {
        Self(flash)
    }
}

impl<F: Flash, const ERASE_SIZE: usize> Flash for LargeErase<F, ERASE_SIZE> {
    const ERASE_VALUE: u8 = F::ERASE_VALUE;
}

impl<F: ErrorType, const ERASE_SIZE: usize> ErrorType for LargeErase<F, ERASE_SIZE> {
    type Error = F::Error;
}

impl<F: NorFlash, const ERASE_SIZE: usize> NorFlash for LargeErase<F, ERASE_SIZE> {
    const WRITE_SIZE: usize = F::ERASE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        assert!(ERASE_SIZE >= F::ERASE_SIZE);
        assert_eq!(0, ERASE_SIZE % F::ERASE_SIZE);
        self.0.erase(from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.write(offset, bytes)
    }
}

impl<F: ReadNorFlash, const ERASE_SIZE: usize> ReadNorFlash for LargeErase<F, ERASE_SIZE> {
    const READ_SIZE: usize = F::READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<F: AsyncNorFlash, const ERASE_SIZE: usize> AsyncNorFlash for LargeErase<F, ERASE_SIZE> {
    const WRITE_SIZE: usize = F::ERASE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        assert!(ERASE_SIZE >= F::ERASE_SIZE);
        assert_eq!(0, ERASE_SIZE % F::ERASE_SIZE);
        self.0.erase(from, to).await
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.write(offset, bytes).await
    }
}

impl<F: AsyncReadNorFlash, const ERASE_SIZE: usize> AsyncReadNorFlash for LargeErase<F, ERASE_SIZE> {
    const READ_SIZE: usize = F::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read(offset, bytes).await
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}
