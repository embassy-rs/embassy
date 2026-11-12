use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use super::asynch::{REGION_ACCESS, op};
use super::common::{Op, blocking_op, check_range};
use super::{EEPROM_BASE, EEPROM_SIZE, Error, WRITE_SIZE};
use crate::mode::{Async, Blocking, Mode};
use crate::peripherals::FLASH;

// The EEPROM is programmed with the 32-bit word writes of the flash.
const _: () = core::assert!(WRITE_SIZE == 4);

/// Driver for the data EEPROM.
///
/// The EEPROM is written without erasing it first, at any offset and with any length. The
/// hardware erases and programs it in 32-bit words, which takes a few milliseconds per word: a
/// write rewrites every word it touches, except the ones that already have the value written.
/// While a word is programmed, the CPU stalls when it reads from the flash bank being written.
///
/// Erased EEPROM reads as `0x00`.
pub struct Eeprom<'d, M: Mode> {
    _flash: Peri<'d, FLASH>,
    _mode: PhantomData<M>,
}

impl<'d, M: Mode> Eeprom<'d, M> {
    pub(crate) fn new(flash: Peri<'d, FLASH>) -> Self {
        Self {
            _flash: flash,
            _mode: PhantomData,
        }
    }

    /// Size of the EEPROM in bytes, [`EEPROM_SIZE`].
    pub fn capacity(&self) -> usize {
        EEPROM_SIZE
    }

    /// Blocking read.
    ///
    /// NOTE: `offset` is an offset from the start of the EEPROM, [`EEPROM_BASE`].
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        check_range(offset, bytes.len(), EEPROM_SIZE as u32)?;
        let address = EEPROM_BASE as u32 + offset;
        let eeprom = unsafe { core::slice::from_raw_parts(address as *const u8, bytes.len()) };
        bytes.copy_from_slice(eeprom);
        Ok(())
    }
}

impl<'d> Eeprom<'d, Blocking> {
    /// Blocking write.
    ///
    /// NOTE: `offset` is an offset from the start of the EEPROM, [`EEPROM_BASE`].
    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        check_range(offset, bytes.len(), EEPROM_SIZE as u32)?;
        for (address, word) in words(offset, bytes) {
            critical_section::with(|_| blocking_op(Op::Write(address, &word)))?;
        }
        Ok(())
    }
}

impl<'d> Eeprom<'d, Async> {
    /// Async read.
    ///
    /// Note: reading from the EEPROM can't actually block, so this is the same as
    /// `blocking_read`.
    pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        self.blocking_read(offset, bytes)
    }

    /// Async write.
    ///
    /// NOTE: `offset` is an offset from the start of the EEPROM, [`EEPROM_BASE`].
    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        check_range(offset, bytes.len(), EEPROM_SIZE as u32)?;
        let _guard = REGION_ACCESS.lock().await;
        for (address, word) in words(offset, bytes) {
            op(Op::Write(address, &word)).await?;
        }
        Ok(())
    }
}

/// The words to program to write `bytes` at `offset`, with the bytes they hold outside of
/// `bytes` unchanged. The words that already have their new value are skipped.
fn words(offset: u32, bytes: &[u8]) -> impl Iterator<Item = (u32, [u8; WRITE_SIZE])> + '_ {
    let start = EEPROM_BASE as u32 + offset;
    let end = start + bytes.len() as u32;
    (start & !3..end).step_by(4).filter_map(move |address| {
        let current = unsafe { (address as *const u32).read_volatile() }.to_le_bytes();
        let mut word = current;
        for (i, b) in word.iter_mut().enumerate() {
            let a = address + i as u32;
            if (start..end).contains(&a) {
                *b = bytes[(a - start) as usize];
            }
        }
        (word != current).then_some((address, word))
    })
}

impl<M: Mode> embedded_storage::ReadStorage for Eeprom<'_, M> {
    type Error = Error;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        Eeprom::capacity(self)
    }
}

impl embedded_storage::Storage for Eeprom<'_, Blocking> {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}
