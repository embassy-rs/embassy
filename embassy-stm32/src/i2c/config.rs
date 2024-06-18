use stm32_metapac::i2c::vals::Oamsk;

use crate::gpio::Pull;

#[repr(u8)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddrMask {
    NOMASK,
    MASK1,
    MASK2,
    MASK3,
    MASK4,
    MASK5,
    MASK6,
    MASK7,
}
impl From<AddrMask> for Oamsk {
    fn from(value: AddrMask) -> Self {
        match value {
            AddrMask::NOMASK => Oamsk::NOMASK,
            AddrMask::MASK1 => Oamsk::MASK1,
            AddrMask::MASK2 => Oamsk::MASK2,
            AddrMask::MASK3 => Oamsk::MASK3,
            AddrMask::MASK4 => Oamsk::MASK4,
            AddrMask::MASK5 => Oamsk::MASK5,
            AddrMask::MASK6 => Oamsk::MASK6,
            AddrMask::MASK7 => Oamsk::MASK7,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Address {
    SevenBit(u8),
    TenBit(u16),
}
impl From<u8> for Address {
    fn from(value: u8) -> Self {
        Address::SevenBit(value)
    }
}
impl From<u16> for Address {
    fn from(value: u16) -> Self {
        assert!(value < 0x400, "Ten bit address must be less than 0x400");
        Address::TenBit(value)
    }
}
impl Address {
    pub(super) fn add_mode(&self) -> stm32_metapac::i2c::vals::Addmode {
        match self {
            Address::SevenBit(_) => stm32_metapac::i2c::vals::Addmode::BIT7,
            Address::TenBit(_) => stm32_metapac::i2c::vals::Addmode::BIT10,
        }
    }
    pub fn addr(&self) -> u16 {
        match self {
            Address::SevenBit(addr) => *addr as u16,
            Address::TenBit(addr) => *addr,
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OA2 {
    pub addr: u8,
    pub mask: AddrMask,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OwnAddresses {
    OA1(Address),
    OA2(OA2),
    Both { oa1: Address, oa2: OA2 },
}

/// Slave Configuration
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SlaveAddrConfig {
    /// Target Address(es)
    pub addr: OwnAddresses,
    /// Control if the peripheral should respond to the general call address
    pub general_call: bool,
}
impl SlaveAddrConfig {
    pub fn new_oa1(addr: Address, general_call: bool) -> Self {
        Self {
            addr: OwnAddresses::OA1(addr),
            general_call,
        }
    }

    pub fn basic(addr: Address) -> Self {
        Self {
            addr: OwnAddresses::OA1(addr),
            general_call: false,
        }
    }
}

/// I2C config
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Enable internal pullup on SDA.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub sda_pullup: bool,
    /// Enable internal pullup on SCL.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub scl_pullup: bool,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sda_pullup: false,
            scl_pullup: false,
            #[cfg(feature = "time")]
            timeout: embassy_time::Duration::from_millis(1000),
        }
    }
}

impl Config {
    pub(super) fn scl_pull_mode(&self) -> Pull {
        match self.scl_pullup {
            true => Pull::Up,
            false => Pull::Down,
        }
    }

    pub(super) fn sda_pull_mode(&self) -> Pull {
        match self.sda_pullup {
            true => Pull::Up,
            false => Pull::Down,
        }
    }
}
