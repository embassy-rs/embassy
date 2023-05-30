use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

use super::Error;

/// A logical partition of an underlying shared flash
///
/// A partition holds an offset and a size of the flash,
/// and is restricted to operate with that range.
/// There is no guarantee that muliple partitions on the same flash
/// operate on mutually exclusive ranges - such a separation is up to
/// the user to guarantee.
pub struct BlockingPartition<'a, M: RawMutex, T: NorFlash> {
    flash: &'a Mutex<M, RefCell<T>>,
    offset: u32,
    size: u32,
}

impl<'a, M: RawMutex, T: NorFlash> BlockingPartition<'a, M, T> {
    /// Create a new partition
    pub const fn new(flash: &'a Mutex<M, RefCell<T>>, offset: u32, size: u32) -> Self {
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

impl<M: RawMutex, T: NorFlash> ErrorType for BlockingPartition<'_, M, T> {
    type Error = Error<T::Error>;
}

impl<M: RawMutex, T: NorFlash> ReadNorFlash for BlockingPartition<'_, M, T> {
    const READ_SIZE: usize = T::READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset + bytes.len() as u32 > self.size {
            return Err(Error::OutOfBounds);
        }

        self.flash.lock(|flash| {
            flash
                .borrow_mut()
                .read(self.offset + offset, bytes)
                .map_err(Error::Flash)
        })
    }

    fn capacity(&self) -> usize {
        self.size as usize
    }
}

impl<M: RawMutex, T: NorFlash> NorFlash for BlockingPartition<'_, M, T> {
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const ERASE_SIZE: usize = T::ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset + bytes.len() as u32 > self.size {
            return Err(Error::OutOfBounds);
        }

        self.flash.lock(|flash| {
            flash
                .borrow_mut()
                .write(self.offset + offset, bytes)
                .map_err(Error::Flash)
        })
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to > self.size {
            return Err(Error::OutOfBounds);
        }

        self.flash.lock(|flash| {
            flash
                .borrow_mut()
                .erase(self.offset + from, self.offset + to)
                .map_err(Error::Flash)
        })
    }
}

#[cfg(test)]
mod tests {
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;

    use super::*;
    use crate::flash::mem_flash::MemFlash;

    #[test]
    fn can_read() {
        let mut flash = MemFlash::<1024, 128, 4>::default();
        flash.mem[132..132 + 8].fill(0xAA);

        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(flash));
        let mut partition = BlockingPartition::new(&flash, 128, 256);

        let mut read_buf = [0; 8];
        partition.read(4, &mut read_buf).unwrap();

        assert!(read_buf.iter().position(|&x| x != 0xAA).is_none());
    }

    #[test]
    fn can_write() {
        let flash = MemFlash::<1024, 128, 4>::default();

        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(flash));
        let mut partition = BlockingPartition::new(&flash, 128, 256);

        let write_buf = [0xAA; 8];
        partition.write(4, &write_buf).unwrap();

        let flash = flash.into_inner().take();
        assert!(flash.mem[132..132 + 8].iter().position(|&x| x != 0xAA).is_none());
    }

    #[test]
    fn can_erase() {
        let flash = MemFlash::<1024, 128, 4>::new(0x00);

        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(flash));
        let mut partition = BlockingPartition::new(&flash, 128, 256);

        partition.erase(0, 128).unwrap();

        let flash = flash.into_inner().take();
        assert!(flash.mem[128..256].iter().position(|&x| x != 0xFF).is_none());
    }
}
