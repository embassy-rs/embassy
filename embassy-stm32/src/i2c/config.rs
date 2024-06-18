use crate::gpio::Pull;

#[repr(u8)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Bits of the I2C OA2 register to mask out.
pub enum AddrMask {
    /// No mask
    NOMASK,
    /// OA2\[1\] is masked and don’t care. Only OA2\[7:2\] are compared.
    MASK1,
    /// OA2\[2:1\] are masked and don’t care. Only OA2\[7:3\] are compared.
    MASK2,
    /// OA2\[3:1\] are masked and don’t care. Only OA2\[7:4\] are compared.
    MASK3,
    /// OA2\[4:1\] are masked and don’t care. Only OA2\[7:5\] are compared.
    MASK4,
    /// OA2\[5:1\] are masked and don’t care. Only OA2\[7:6\] are compared.
    MASK5,
    /// OA2\[6:1\] are masked and don’t care. Only OA2\[7:6\] are compared.
    MASK6,
    /// OA2\[7:1\] are masked and don’t care. No comparison is done, and all (except reserved) 7-bit received addresses are acknowledged
    MASK7,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// An I2C address. Either 7 or 10 bit.
pub enum Address {
    /// A 7 bit address
    SevenBit(u8),
    /// A 10 bit address.
    ///
    /// When using an address to configure the Own Address, only the OA1 register can be set to a 10-bit address.
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
    /// Get the inner address as a u16.
    ///
    /// For 7 bit addresses, the u8 that was used to store the address is returned as a u16.
    pub fn addr(&self) -> u16 {
        match self {
            Address::SevenBit(addr) => *addr as u16,
            Address::TenBit(addr) => *addr,
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The second Own Address register.
pub struct OA2 {
    /// The address.
    pub addr: u8,
    /// The bit mask that will affect how the own address 2 register is compared.
    pub mask: AddrMask,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The Own Address(es) of the I2C peripheral.
pub enum OwnAddresses {
    /// Configuration for only the OA1 register.
    OA1(Address),
    /// Configuration for only the OA2 register.
    OA2(OA2),
    /// Configuration for both the OA1 and OA2 registers.
    Both {
        /// The [Address] for the OA1 register.
        oa1: Address,
        /// The [OA2] configuration.
        oa2: OA2,
    },
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
    /// Create a new slave address configuration with only the OA1 register set in 7 bit mode and the general call disabled.
    pub fn basic(addr: u8) -> Self {
        Self {
            addr: OwnAddresses::OA1(Address::SevenBit(addr)),
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
