use alloc::vec::Vec;

use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
#[cfg(feature = "nightly")]
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

extern crate alloc;

pub(crate) struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> {
    pub mem: [u8; SIZE],
    pub writes: Vec<(u32, usize)>,
    pub erases: Vec<(u32, u32)>,
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE> {
    #[allow(unused)]
    pub const fn new(fill: u8) -> Self {
        Self {
            mem: [fill; SIZE],
            writes: Vec::new(),
            erases: Vec::new(),
        }
    }

    fn read(&mut self, offset: u32, bytes: &mut [u8]) {
        let len = bytes.len();
        bytes.copy_from_slice(&self.mem[offset as usize..offset as usize + len]);
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) {
        self.writes.push((offset, bytes.len()));
        let offset = offset as usize;
        assert_eq!(0, bytes.len() % WRITE_SIZE);
        assert_eq!(0, offset % WRITE_SIZE);
        assert!(offset + bytes.len() <= SIZE);

        self.mem[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    fn erase(&mut self, from: u32, to: u32) {
        self.erases.push((from, to));
        let from = from as usize;
        let to = to as usize;
        assert_eq!(0, from % ERASE_SIZE);
        assert_eq!(0, to % ERASE_SIZE);
        self.mem[from..to].fill(0xff);
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> Default
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    fn default() -> Self {
        Self::new(0xff)
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ErrorType
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    type Error = core::convert::Infallible;
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ReadNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes);
        Ok(())
    }

    fn capacity(&self) -> usize {
        SIZE
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> NorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes);
        Ok(())
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to);
        Ok(())
    }
}

#[cfg(feature = "nightly")]
impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes);
        Ok(())
    }

    fn capacity(&self) -> usize {
        SIZE
    }
}

#[cfg(feature = "nightly")]
impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes);
        Ok(())
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to);
        Ok(())
    }
}
