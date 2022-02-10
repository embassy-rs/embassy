use core::marker::PhantomData;

use crate::pac::CRC as PAC_CRC;
use crate::peripherals::CRC;
use crate::rcc::sealed::RccPeripheral;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

pub struct Crc<'d> {
    _peripheral: CRC,
    _phantom: PhantomData<&'d mut CRC>,
}

impl<'d> Crc<'d> {
    /// Instantiates the CRC32 peripheral and initializes it to default values.
    pub fn new(peripheral: impl Unborrow<Target = CRC> + 'd) -> Self {
        // Note: enable and reset come from RccPeripheral.
        // enable CRC clock in RCC.
        CRC::enable();
        // Reset CRC to default values.
        CRC::reset();
        // Unborrow the peripheral
        unborrow!(peripheral);
        let mut instance = Self {
            _peripheral: peripheral,
            _phantom: PhantomData,
        };
        instance.reset();
        instance
    }

    /// Resets the CRC unit to default value (0xFFFF_FFFF)
    pub fn reset(&mut self) {
        unsafe { PAC_CRC.cr().write(|w| w.set_reset(true)) };
    }

    /// Feeds a word to the peripheral and returns the current CRC value
    pub fn feed_word(&mut self, word: u32) -> u32 {
        // write a single byte to the device, and return the result
        unsafe {
            PAC_CRC.dr().write_value(word);
        }
        self.read()
    }
    /// Feed a slice of words to the peripheral and return the result.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            unsafe { PAC_CRC.dr().write_value(*word) }
        }

        self.read()
    }
    pub fn read(&self) -> u32 {
        unsafe { PAC_CRC.dr().read() }
    }
}
