use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::pac::CRC as PAC_CRC;
use crate::peripherals::CRC;
use crate::rcc::sealed::RccPeripheral;
use crate::Peripheral;

pub struct Crc<'d> {
    _peri: PeripheralRef<'d, CRC>,
}

impl<'d> Crc<'d> {
    /// Instantiates the CRC32 peripheral and initializes it to default values.
    pub fn new(peripheral: impl Peripheral<P = CRC> + 'd) -> Self {
        into_ref!(peripheral);

        // Note: enable and reset come from RccPeripheral.
        // enable CRC clock in RCC.
        CRC::enable();
        // Reset CRC to default values.
        CRC::reset();
        // Peripheral the peripheral
        let mut instance = Self { _peri: peripheral };
        instance.reset();
        instance
    }

    /// Resets the CRC unit to default value (0xFFFF_FFFF)
    pub fn reset(&mut self) {
        PAC_CRC.cr().write(|w| w.set_reset(true));
    }

    /// Feeds a word to the peripheral and returns the current CRC value
    pub fn feed_word(&mut self, word: u32) -> u32 {
        // write a single byte to the device, and return the result
        PAC_CRC.dr().write_value(word);
        self.read()
    }
    /// Feed a slice of words to the peripheral and return the result.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            PAC_CRC.dr().write_value(*word);
        }

        self.read()
    }
    pub fn read(&self) -> u32 {
        PAC_CRC.dr().read()
    }
}
