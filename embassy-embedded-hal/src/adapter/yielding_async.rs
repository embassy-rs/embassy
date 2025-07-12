use embassy_futures::yield_now;

/// Wrapper that yields for each operation to the wrapped instance
///
/// This can be used in combination with BlockingAsync<T> to enforce yields
/// between long running blocking operations.
pub struct YieldingAsync<T> {
    wrapped: T,
}

impl<T> YieldingAsync<T> {
    /// Create a new instance of a wrapper that yields after each operation.
    pub fn new(wrapped: T) -> Self {
        Self { wrapped }
    }
}

//
// I2C implementations
//
impl<T> embedded_hal_1::i2c::ErrorType for YieldingAsync<T>
where
    T: embedded_hal_1::i2c::ErrorType,
{
    type Error = T::Error;
}

impl<T> embedded_hal_async::i2c::I2c for YieldingAsync<T>
where
    T: embedded_hal_async::i2c::I2c,
{
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, read).await?;
        yield_now().await;
        Ok(())
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(address, write).await?;
        yield_now().await;
        Ok(())
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.write_read(address, write, read).await?;
        yield_now().await;
        Ok(())
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.wrapped.transaction(address, operations).await?;
        yield_now().await;
        Ok(())
    }
}

//
// SPI implementations
//

impl<T> embedded_hal_async::spi::ErrorType for YieldingAsync<T>
where
    T: embedded_hal_async::spi::ErrorType,
{
    type Error = T::Error;
}

impl<T, Word: 'static + Copy> embedded_hal_async::spi::SpiBus<Word> for YieldingAsync<T>
where
    T: embedded_hal_async::spi::SpiBus<Word>,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.wrapped.flush().await?;
        yield_now().await;
        Ok(())
    }

    async fn write(&mut self, data: &[Word]) -> Result<(), Self::Error> {
        self.wrapped.write(data).await?;
        yield_now().await;
        Ok(())
    }

    async fn read(&mut self, data: &mut [Word]) -> Result<(), Self::Error> {
        self.wrapped.read(data).await?;
        yield_now().await;
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [Word], write: &[Word]) -> Result<(), Self::Error> {
        self.wrapped.transfer(read, write).await?;
        yield_now().await;
        Ok(())
    }

    async fn transfer_in_place(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
        self.wrapped.transfer_in_place(words).await?;
        yield_now().await;
        Ok(())
    }
}

///
/// NOR flash implementations
///
impl<T: embedded_storage::nor_flash::ErrorType> embedded_storage::nor_flash::ErrorType for YieldingAsync<T> {
    type Error = T::Error;
}

impl<T: embedded_storage_async::nor_flash::ReadNorFlash> embedded_storage_async::nor_flash::ReadNorFlash
    for YieldingAsync<T>
{
    const READ_SIZE: usize = T::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(offset, bytes).await?;
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.wrapped.capacity()
    }
}

impl<T: embedded_storage_async::nor_flash::NorFlash> embedded_storage_async::nor_flash::NorFlash for YieldingAsync<T> {
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const ERASE_SIZE: usize = T::ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(offset, bytes).await?;
        yield_now().await;
        Ok(())
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        // Yield between each actual erase
        for from in (from..to).step_by(T::ERASE_SIZE) {
            let to = core::cmp::min(from + T::ERASE_SIZE as u32, to);
            self.wrapped.erase(from, to).await?;
            yield_now().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_storage_async::nor_flash::NorFlash;

    use super::*;
    use crate::flash::mem_flash::MemFlash;

    #[futures_test::test]
    async fn can_erase() {
        let flash = MemFlash::<1024, 128, 4>::new(0x00);
        let mut yielding = YieldingAsync::new(flash);

        yielding.erase(0, 256).await.unwrap();

        let flash = yielding.wrapped;
        assert_eq!(2, flash.erases.len());
        assert_eq!((0, 128), flash.erases[0]);
        assert_eq!((128, 256), flash.erases[1]);
    }
}
