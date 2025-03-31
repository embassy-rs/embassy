use crate::pac::CRC as PAC_CRC;
use crate::peripherals::CRC;
use crate::{rcc, Peri};

/// CRC driver.
pub struct Crc<'d> {
    _peri: Peri<'d, CRC>,
}

impl<'d> Crc<'d> {
    /// Instantiates the CRC32 peripheral and initializes it to default values.
    pub fn new(peripheral: Peri<'d, CRC>) -> Self {
        // Note: enable and reset come from RccPeripheral.
        // enable CRC clock in RCC.
        rcc::enable_and_reset::<CRC>();
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
        #[cfg(not(crc_v1))]
        PAC_CRC.dr32().write_value(word);
        #[cfg(crc_v1)]
        PAC_CRC.dr().write_value(word);
        self.read()
    }

    /// Feed a slice of words to the peripheral and return the result.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            #[cfg(not(crc_v1))]
            PAC_CRC.dr32().write_value(*word);
            #[cfg(crc_v1)]
            PAC_CRC.dr().write_value(*word);
        }

        self.read()
    }

    /// Read the CRC result value.
    #[cfg(not(crc_v1))]
    pub fn read(&self) -> u32 {
        PAC_CRC.dr32().read()
    }
    /// Read the CRC result value.
    #[cfg(crc_v1)]
    pub fn read(&self) -> u32 {
        PAC_CRC.dr().read()
    }
}
