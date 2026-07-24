//! Cyclic Redundancy Check (CRC)

use core::marker::PhantomData;

use crate::clocks::{SysconPeripheral, enable_and_reset};
use crate::{Peri, PeripheralType, peripherals};

/// CRC polynomial selection.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Polynomial {
    /// CRC-CCITT polynomial
    CrcCcitt = 0,
    /// CRC-16 polynomial
    Crc16 = 1,
    /// CRC-32 polynomial
    Crc32 = 2,
}

/// CRC driver.
pub struct Crc<'d> {
    info: Info,
    _config: Config,
    _lifetime: PhantomData<&'d ()>,
}

/// CRC configuration
pub struct Config {
    /// Polynomial to be used
    pub polynomial: Polynomial,

    /// Reverse bit order of input?
    pub reverse_in: bool,

    /// 1's complement input?
    pub complement_in: bool,

    /// Reverse CRC bit order?
    pub reverse_out: bool,

    /// 1's complement CRC?
    pub complement_out: bool,

    /// CRC Seed
    pub seed: u32,
}

impl Config {
    /// Create a new CRC config.
    #[must_use]
    pub fn new(
        polynomial: Polynomial,
        reverse_in: bool,
        complement_in: bool,
        reverse_out: bool,
        complement_out: bool,
        seed: u32,
    ) -> Self {
        Config {
            polynomial,
            reverse_in,
            complement_in,
            reverse_out,
            complement_out,
            seed,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polynomial: Polynomial::CrcCcitt,
            reverse_in: false,
            complement_in: false,
            reverse_out: false,
            complement_out: false,
            seed: 0xffff,
        }
    }
}

impl<'d> Crc<'d> {
    /// Instantiates new CRC peripheral and initializes to default values.
    pub fn new<T: Instance>(_peripheral: Peri<'d, T>, config: Config) -> Self {
        // enable CRC clock
        enable_and_reset::<T>();

        let mut instance = Self {
            info: T::info(),
            _config: config,
            _lifetime: PhantomData,
        };

        instance.reconfigure();
        instance
    }

    /// Reconfigured the CRC peripheral.
    fn reconfigure(&mut self) {
        self.info.regs.mode().write(|w| {
            w.set_crc_poly(self._config.polynomial as u8);
            w.set_bit_rvs_wr(self._config.reverse_in);
            w.set_cmpl_wr(self._config.complement_in);
            w.set_bit_rvs_sum(self._config.reverse_out);
            w.set_cmpl_sum(self._config.complement_out);
        });

        // Init CRC value
        self.info.regs.seed().write(|w| w.set_crc_seed(self._config.seed));
    }

    /// Feeds a byte into the CRC peripheral. Returns the computed checksum.
    pub fn feed_byte(&mut self, byte: u8) -> u32 {
        self.info.regs.wr_data8().write(|w| w.set_data(byte));

        self.info.regs.sum().read().crc_sum()
    }

    /// Feeds an slice of bytes into the CRC peripheral. Returns the computed checksum.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> u32 {
        let (prefix, data, suffix) = unsafe { bytes.align_to::<u32>() };

        for b in prefix {
            self.info.regs.wr_data8().write(|w| w.set_data(*b));
        }

        for d in data {
            self.info.regs.wr_data32().write(|w| w.set_data(*d));
        }

        for b in suffix {
            self.info.regs.wr_data8().write(|w| w.set_data(*b));
        }

        self.info.regs.sum().read().crc_sum()
    }

    /// Feeds a halfword into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfword(&mut self, halfword: u16) -> u32 {
        self.info.regs.wr_data16().write(|w| w.set_data(halfword));

        self.info.regs.sum().read().crc_sum()
    }

    /// Feeds an slice of halfwords into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfwords(&mut self, halfwords: &[u16]) -> u32 {
        for halfword in halfwords {
            self.info.regs.wr_data16().write(|w| w.set_data(*halfword));
        }

        self.info.regs.sum().read().crc_sum()
    }

    /// Feeds a words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_word(&mut self, word: u32) -> u32 {
        self.info.regs.wr_data32().write(|w| w.set_data(word));

        self.info.regs.sum().read().crc_sum()
    }

    /// Feeds an slice of words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            self.info.regs.wr_data32().write(|w| w.set_data(*word));
        }

        self.info.regs.sum().read().crc_sum()
    }
}

struct Info {
    regs: crate::pac::crc::Crc,
}

trait SealedInstance {
    fn info() -> Info;
}

/// CRC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + SysconPeripheral + 'static + Send {}

impl Instance for peripherals::CRC {}

impl SealedInstance for peripherals::CRC {
    fn info() -> Info {
        Info {
            regs: crate::pac::CRC_ENGINE,
        }
    }
}
