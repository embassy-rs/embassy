use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage::nor_flash::ErrorType;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};

use super::Error;

/// A logical partition of an underlying shared flash
///
/// A partition holds an offset and a size of the flash,
/// and is restricted to operate with that range.
/// There is no guarantee that muliple partitions on the same flash
/// operate on mutually exclusive ranges - such a separation is up to
/// the user to guarantee.
pub struct Partition<'a, M: RawMutex, T: NorFlash> {
    flash: &'a Mutex<M, T>,
    offset: u32,
    size: u32,
}

impl<'a, M: RawMutex, T: NorFlash> Partition<'a, M, T> {
    /// Create a new partition
    pub const fn new(flash: &'a Mutex<M, T>, offset: u32, size: u32) -> Self {
        if offset % T::READ_SIZE as u32 != 0 || offset % T::WRITE_SIZE as u32 != 0 || offset % T::ERASE_SIZE as u32 != 0
        {
            panic!("Partition offset must be a multiple of read, write and erase size");
        }
        if size % T::READ_SIZE as u32 != 0 || size % T::WRITE_SIZE as u32 != 0 || size % T::ERASE_SIZE as u32 != 0 {
            panic!("Partition size must be a multiple of read, write and erase size");
        }
        Self { flash, offset, size }
    }

    /// Get the partition offset within the flash
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Get the partition size
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl<M: RawMutex, T: NorFlash> ErrorType for Partition<'_, M, T> {
    type Error = Error<T::Error>;
}

impl<M: RawMutex, T: NorFlash> ReadNorFlash for Partition<'_, M, T> {
    const READ_SIZE: usize = T::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset + bytes.len() as u32 > self.size {
            return Err(Error::OutOfBounds);
        }

        let mut flash = self.flash.lock().await;
        flash.read(self.offset + offset, bytes).await.map_err(Error::Flash)
    }

    fn capacity(&self) -> usize {
        self.size as usize
    }
}

impl<M: RawMutex, T: NorFlash> NorFlash for Partition<'_, M, T> {
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const ERASE_SIZE: usize = T::ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset + bytes.len() as u32 > self.size {
            return Err(Error::OutOfBounds);
        }

        let mut flash = self.flash.lock().await;
        flash.write(self.offset + offset, bytes).await.map_err(Error::Flash)
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to > self.size {
            return Err(Error::OutOfBounds);
        }

        let mut flash = self.flash.lock().await;
        flash
            .erase(self.offset + from, self.offset + to)
            .await
            .map_err(Error::Flash)
    }
}

#[cfg(test)]
mod tests {
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;

    use super::*;
    use crate::flash::mem_flash::MemFlash;

    #[futures_test::test]
    async fn can_read() {
        let mut flash = MemFlash::<1024, 128, 4>::default();
        flash.mem[132..132 + 8].fill(0xAA);

        let flash = Mutex::<NoopRawMutex, _>::new(flash);
        let mut partition = Partition::new(&flash, 128, 256);

        let mut read_buf = [0; 8];
        partition.read(4, &mut read_buf).await.unwrap();

        assert!(read_buf.iter().position(|&x| x != 0xAA).is_none());
    }

    #[futures_test::test]
    async fn can_write() {
        let flash = MemFlash::<1024, 128, 4>::default();

        let flash = Mutex::<NoopRawMutex, _>::new(flash);
        let mut partition = Partition::new(&flash, 128, 256);

        let write_buf = [0xAA; 8];
        partition.write(4, &write_buf).await.unwrap();

        let flash = flash.try_lock().unwrap();
        assert!(flash.mem[132..132 + 8].iter().position(|&x| x != 0xAA).is_none());
    }

    #[futures_test::test]
    async fn can_erase() {
        let flash = MemFlash::<1024, 128, 4>::new(0x00);

        let flash = Mutex::<NoopRawMutex, _>::new(flash);
        let mut partition = Partition::new(&flash, 128, 256);

        partition.erase(0, 128).await.unwrap();

        let flash = flash.try_lock().unwrap();
        assert!(flash.mem[128..256].iter().position(|&x| x != 0xFF).is_none());
    }
}
