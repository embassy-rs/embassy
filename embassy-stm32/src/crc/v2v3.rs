use crate::pac::crc::vals;
use crate::pac::CRC as PAC_CRC;
use crate::peripherals::CRC;
use crate::{rcc, Peri};

/// CRC driver.
pub struct Crc<'d> {
    _peripheral: Peri<'d, CRC>,
    _config: Config,
}

/// CRC configuration errlr
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigError {
    /// The selected polynomial is invalid.
    InvalidPolynomial,
}

/// CRC configuration
pub struct Config {
    reverse_in: InputReverseConfig,
    reverse_out: bool,
    #[cfg(crc_v3)]
    poly_size: PolySize,
    crc_init_value: u32,
    #[cfg(crc_v3)]
    crc_poly: u32,
}

/// Input reverse configuration.
pub enum InputReverseConfig {
    /// Don't reverse anything
    None,
    /// Reverse bytes
    Byte,
    /// Reverse 16-bit halfwords.
    Halfword,
    /// Reverse 32-bit words.
    Word,
}

impl Config {
    /// Create a new CRC config.
    pub fn new(
        reverse_in: InputReverseConfig,
        reverse_out: bool,
        #[cfg(crc_v3)] poly_size: PolySize,
        crc_init_value: u32,
        #[cfg(crc_v3)] crc_poly: u32,
    ) -> Result<Self, ConfigError> {
        // As Per RM0091 (DocID018940 Rev 9), Even polynomials are not supported.
        #[cfg(crc_v3)]
        if crc_poly % 2 == 0 {
            return Err(ConfigError::InvalidPolynomial);
        }
        Ok(Config {
            reverse_in,
            reverse_out,
            #[cfg(crc_v3)]
            poly_size,
            crc_init_value,
            #[cfg(crc_v3)]
            crc_poly,
        })
    }
}

/// Polynomial size
#[cfg(crc_v3)]
#[allow(missing_docs)]
pub enum PolySize {
    Width7,
    Width8,
    Width16,
    Width32,
}

impl<'d> Crc<'d> {
    /// Instantiates the CRC32 peripheral and initializes it to default values.
    pub fn new(peripheral: Peri<'d, CRC>, config: Config) -> Self {
        // Note: enable and reset come from RccPeripheral.
        // reset to default values and enable CRC clock in RCC.
        rcc::enable_and_reset::<CRC>();
        let mut instance = Self {
            _peripheral: peripheral,
            _config: config,
        };
        instance.reconfigure();
        instance.reset();
        instance
    }

    /// Reset the CRC engine.
    pub fn reset(&mut self) {
        PAC_CRC.cr().modify(|w| w.set_reset(true));
    }

    /// Reconfigures the CRC peripheral. Doesn't reset.
    fn reconfigure(&mut self) {
        // Init CRC value
        PAC_CRC.init().write_value(self._config.crc_init_value);
        #[cfg(crc_v3)]
        PAC_CRC.pol().write_value(self._config.crc_poly);

        // configure CR components
        // (reverse I/O, polysize, poly)
        PAC_CRC.cr().write(|w| {
            // configure reverse output
            w.set_rev_out(match self._config.reverse_out {
                true => vals::RevOut::REVERSED,
                false => vals::RevOut::NORMAL,
            });
            // configure reverse input
            w.set_rev_in(match self._config.reverse_in {
                InputReverseConfig::None => vals::RevIn::NORMAL,
                InputReverseConfig::Byte => vals::RevIn::BYTE,
                InputReverseConfig::Halfword => vals::RevIn::HALF_WORD,
                InputReverseConfig::Word => vals::RevIn::WORD,
            });
            // configure the polynomial.
            #[cfg(crc_v3)]
            w.set_polysize(match self._config.poly_size {
                PolySize::Width7 => vals::Polysize::POLYSIZE7,
                PolySize::Width8 => vals::Polysize::POLYSIZE8,
                PolySize::Width16 => vals::Polysize::POLYSIZE16,
                PolySize::Width32 => vals::Polysize::POLYSIZE32,
            });
        });

        self.reset();
    }

    /// Feeds a byte into the CRC peripheral. Returns the computed checksum.
    pub fn feed_byte(&mut self, byte: u8) -> u32 {
        PAC_CRC.dr8().write_value(byte);
        PAC_CRC.dr32().read()
    }

    /// Feeds an slice of bytes into the CRC peripheral. Returns the computed checksum.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> u32 {
        for byte in bytes {
            PAC_CRC.dr8().write_value(*byte);
        }
        PAC_CRC.dr32().read()
    }
    /// Feeds a halfword into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfword(&mut self, halfword: u16) -> u32 {
        PAC_CRC.dr16().write_value(halfword);
        PAC_CRC.dr32().read()
    }
    /// Feeds an slice of halfwords into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfwords(&mut self, halfwords: &[u16]) -> u32 {
        for halfword in halfwords {
            PAC_CRC.dr16().write_value(*halfword);
        }
        PAC_CRC.dr32().read()
    }
    /// Feeds a words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_word(&mut self, word: u32) -> u32 {
        PAC_CRC.dr32().write_value(word as u32);
        PAC_CRC.dr32().read()
    }
    /// Feeds an slice of words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            PAC_CRC.dr32().write_value(*word as u32);
        }
        PAC_CRC.dr32().read()
    }
}
