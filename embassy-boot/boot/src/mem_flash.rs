#![allow(unused)]

use core::ops::{Bound, Range, RangeBounds};

use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

use crate::Flash;

pub struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> {
    pub mem: [u8; SIZE],
    pub allow_same_write: bool,
    pub verify_erased_before_write: Range<usize>,
    pub pending_write_successes: Option<usize>,
}

#[derive(Debug)]
pub struct MemFlashError;

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE> {
    pub const fn new(fill: u8) -> Self {
        Self {
            mem: [fill; SIZE],
            allow_same_write: false,
            verify_erased_before_write: 0..SIZE,
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
            allow_same_write: false,
            verify_erased_before_write: 0..SIZE,
            pending_write_successes: None,
        }
    }

    #[must_use]
    pub fn allow_same_write(self, allow: bool) -> Self {
        Self {
            allow_same_write: allow,
            ..self
        }
    }

    #[must_use]
    pub fn with_limited_erase_before_write_verification<R: RangeBounds<usize>>(self, verified_range: R) -> Self {
        let start = match verified_range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match verified_range.end_bound() {
            Bound::Included(end) => *end - 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.mem.len(),
        };
        Self {
            verify_erased_before_write: start..end,
            ..self
        }
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> Default
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    fn default() -> Self {
        Self::new(0xFF)
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> Flash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const BLOCK_SIZE: usize = ERASE_SIZE;
    const ERASE_VALUE: u8 = 0xFF;
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
        let len = bytes.len();
        bytes.copy_from_slice(&self.mem[offset as usize..offset as usize + len]);
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

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let from = from as usize;
        let to = to as usize;
        assert!(from % ERASE_SIZE == 0);
        assert!(to % ERASE_SIZE == 0, "To: {}, erase size: {}", to, ERASE_SIZE);
        for i in from..to {
            self.mem[i] = 0xFF;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
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
            if self.allow_same_write && mem_byte == new_byte {
                // Write does not change the flash memory which is allowed
            } else {
                if self.verify_erased_before_write.contains(&offset) {
                    assert_eq!(0xFF, *mem_byte, "Offset {} is not erased", offset);
                }
                *mem_byte &= *new_byte;
            }
        }

        Ok(())
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        <Self as ReadNorFlash>::read(self, offset, bytes)
    }

    fn capacity(&self) -> usize {
        <Self as ReadNorFlash>::capacity(self)
    }
}

impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
    for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        <Self as NorFlash>::erase(self, from, to)
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        <Self as NorFlash>::write(self, offset, bytes)
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use embedded_storage::nor_flash::NorFlash;

    use super::MemFlash;

    #[test]
    fn writes_only_flip_bits_from_1_to_0() {
        let mut flash = MemFlash::<16, 16, 1>::default().with_limited_erase_before_write_verification(0..0);

        flash.write(0, &[0x55]).unwrap();
        flash.write(0, &[0xAA]).unwrap();

        assert_eq!(0x00, flash.mem[0]);
    }
}
