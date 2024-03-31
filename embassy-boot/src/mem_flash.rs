#![allow(unused)]

use core::ops::{Bound, Range, RangeBounds};

use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

pub struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> {
    pub mem: [u8; SIZE],
    pub pending_write_successes: Option<usize>,
}

#[derive(Debug)]
pub struct MemFlashError;

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE> {
    pub const fn new(fill: u8) -> Self {
        Self {
            mem: [fill; SIZE],
            pending_write_successes: None,
        }
    }

    #[cfg(test)]
    pub fn random() -> Self {
        let mut mem = [0; SIZE];
        for byte in mem.iter_mut() {
            *byte = rand::random::<u8>();
        }
        Self {
            mem,
            pending_write_successes: None,
        }
    }

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), MemFlashError> {
        let len = bytes.len();
        bytes.copy_from_slice(&self.mem[offset as usize..offset as usize + len]);
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), MemFlashError> {
        let offset = offset as usize;
        assert!(bytes.len() % WRITE_SIZE == 0);
        assert!(offset % WRITE_SIZE == 0);
        assert!(offset + bytes.len() <= SIZE);

        if let Some(pending_successes) = self.pending_write_successes {
            if pending_successes > 0 {
                self.pending_write_successes = Some(pending_successes - 1);
            } else {
                return Err(MemFlashError);
            }
        }

        for ((offset, mem_byte), new_byte) in self
            .mem
            .iter_mut()
            .enumerate()
            .skip(offset)
            .take(bytes.len())
            .zip(bytes)
        {
            assert_eq!(0xFF, *mem_byte, "Offset {} is not erased", offset);
            *mem_byte = *new_byte;
        }

        Ok(())
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), MemFlashError> {
        let from = from as usize;
        let to = to as usize;
        assert!(from % ERASE_SIZE == 0);
        assert!(to % ERASE_SIZE == 0, "To: {}, erase size: {}", to, ERASE_SIZE);
        for i in from..to {
            self.mem[i] = 0xFF;
        }
        Ok(())
    }

    pub fn program(&mut self, offset: u32, bytes: &[u8]) -> Result<(), MemFlashError> {
        let offset = offset as usize;
        assert!(bytes.len() % WRITE_SIZE == 0);
        assert!(offset % WRITE_SIZE == 0);
        assert!(offset + bytes.len() <= SIZE);

        self.mem[offset..offset + bytes.len()].copy_from_slice(bytes);

        Ok(())
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> Default
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    fn default() -> Self {
        Self::new(0xFF)
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ErrorType
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    type Error = MemFlashError;
}

impl NorFlashError for MemFlashError {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ReadNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
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
        self.write(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to)
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        SIZE
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes)
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to)
    }
}
