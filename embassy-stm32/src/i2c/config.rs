#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, OutputType, Speed};
use crate::time::Hertz;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
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

// These methods are only used by the v1 software address sequencing (v2 handles
// 10-bit addressing in hardware). Gated so v2-only builds don't trigger dead_code.
#[cfg(any(i2c_v1, test))]
impl Address {
    /// Wire byte for the write address phase (first byte after START).
    ///
    /// - 7-bit: `addr << 1` (R/W = 0)
    /// - 10-bit: header byte `11110_XX_0` where XX = addr\[9:8\]
    pub(super) fn write_header(&self) -> u8 {
        match self {
            Address::SevenBit(addr) => addr << 1,
            Address::TenBit(addr) => 0xF0 | ((*addr >> 7) as u8 & 0x06),
        }
    }

    /// Wire byte for the read address phase (first byte after START).
    ///
    /// - 7-bit: `(addr << 1) | 1` (R/W = 1)
    /// - 10-bit: header byte `11110_XX_1` where XX = addr\[9:8\]
    pub(super) fn read_header(&self) -> u8 {
        match self {
            Address::SevenBit(addr) => (addr << 1) | 1,
            Address::TenBit(addr) => 0xF0 | ((*addr >> 7) as u8 & 0x06) | 1,
        }
    }

    /// Whether this is a 7-bit address in the reserved 10-bit header range (0x78–0x7B).
    ///
    /// Write bytes for these addresses match the 10-bit header pattern `11110_XX_0`,
    /// causing the v1 I2C peripheral to set ADD10 instead of ADDR.
    pub(super) fn is_reserved_range(&self) -> bool {
        matches!(self, Address::SevenBit(0x78..=0x7B))
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The second Own Address register.
pub struct OA2 {
    /// The address.
    pub addr: u8,
    /// The bit mask that will affect how the own address 2 register is compared.
    pub mask: AddrMask,
}

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
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
    /// Frequency
    pub frequency: Hertz,
    /// GPIO Speed
    pub gpio_speed: Speed,
    /// Enable internal pullup on SDA.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    #[cfg(gpio_v2)]
    pub sda_pullup: bool,
    /// Enable internal pullup on SCL.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    #[cfg(gpio_v2)]
    pub scl_pullup: bool,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Hertz::khz(100),
            gpio_speed: Speed::Medium,
            #[cfg(gpio_v2)]
            sda_pullup: false,
            #[cfg(gpio_v2)]
            scl_pullup: false,
            #[cfg(feature = "time")]
            timeout: embassy_time::Duration::from_millis(1000),
        }
    }
}

impl Config {
    pub(super) fn scl_af(&self) -> AfType {
        #[cfg(gpio_v1)]
        return AfType::output(OutputType::OpenDrain, self.gpio_speed);
        #[cfg(gpio_v2)]
        return AfType::output_pull(
            OutputType::OpenDrain,
            self.gpio_speed,
            match self.scl_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );
    }

    pub(super) fn sda_af(&self) -> AfType {
        #[cfg(gpio_v1)]
        return AfType::output(OutputType::OpenDrain, self.gpio_speed);
        #[cfg(gpio_v2)]
        return AfType::output_pull(
            OutputType::OpenDrain,
            self.gpio_speed,
            match self.sda_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- From conversions ----

    #[test]
    fn from_u8_gives_seven_bit() {
        let addr: Address = 0x50u8.into();
        assert_eq!(addr, Address::SevenBit(0x50));
    }

    #[test]
    fn from_u16_gives_ten_bit() {
        let addr: Address = 0x123u16.into();
        assert_eq!(addr, Address::TenBit(0x123));
    }

    #[test]
    #[should_panic(expected = "Ten bit address must be less than 0x400")]
    fn from_u16_rejects_out_of_range() {
        let _addr: Address = 0x400u16.into();
    }

    #[test]
    fn addr_method_returns_raw_value() {
        assert_eq!(Address::SevenBit(0x50).addr(), 0x50);
        assert_eq!(Address::TenBit(0x3FF).addr(), 0x3FF);
    }

    // ---- write_header ----

    #[test]
    fn write_header_seven_bit_standard() {
        // 0x50 << 1 = 0xA0
        assert_eq!(Address::SevenBit(0x50).write_header(), 0xA0);
    }

    #[test]
    fn write_header_seven_bit_zero() {
        assert_eq!(Address::SevenBit(0x00).write_header(), 0x00);
    }

    #[test]
    fn write_header_seven_bit_max() {
        // 0x7F << 1 = 0xFE
        assert_eq!(Address::SevenBit(0x7F).write_header(), 0xFE);
    }

    #[test]
    fn write_header_seven_bit_reserved_range() {
        // 0x78 << 1 = 0xF0 (matches 10-bit header pattern)
        assert_eq!(Address::SevenBit(0x78).write_header(), 0xF0);
        assert_eq!(Address::SevenBit(0x79).write_header(), 0xF2);
        assert_eq!(Address::SevenBit(0x7A).write_header(), 0xF4);
        assert_eq!(Address::SevenBit(0x7B).write_header(), 0xF6);
    }

    #[test]
    fn write_header_ten_bit_zero() {
        // addr=0x000: header = 11110_00_0 = 0xF0
        assert_eq!(Address::TenBit(0x000).write_header(), 0xF0);
    }

    #[test]
    fn write_header_ten_bit_max() {
        // addr=0x3FF: bits[9:8]=11 → header = 11110_11_0 = 0xF6
        assert_eq!(Address::TenBit(0x3FF).write_header(), 0xF6);
    }

    #[test]
    fn write_header_ten_bit_all_upper_bit_combos() {
        // XX=00 → 0xF0
        assert_eq!(Address::TenBit(0x000).write_header(), 0xF0);
        // XX=01 → 0xF2
        assert_eq!(Address::TenBit(0x100).write_header(), 0xF2);
        // XX=10 → 0xF4
        assert_eq!(Address::TenBit(0x200).write_header(), 0xF4);
        // XX=11 → 0xF6
        assert_eq!(Address::TenBit(0x300).write_header(), 0xF6);
    }

    // ---- read_header ----

    #[test]
    fn read_header_seven_bit_standard() {
        // (0x50 << 1) | 1 = 0xA1
        assert_eq!(Address::SevenBit(0x50).read_header(), 0xA1);
    }

    #[test]
    fn read_header_seven_bit_zero() {
        assert_eq!(Address::SevenBit(0x00).read_header(), 0x01);
    }

    #[test]
    fn read_header_seven_bit_max() {
        // (0x7F << 1) | 1 = 0xFF
        assert_eq!(Address::SevenBit(0x7F).read_header(), 0xFF);
    }

    #[test]
    fn read_header_seven_bit_reserved_range() {
        // 0x78: (0x78 << 1) | 1 = 0xF1 (R/W=1, does NOT trigger ADD10)
        assert_eq!(Address::SevenBit(0x78).read_header(), 0xF1);
        assert_eq!(Address::SevenBit(0x7B).read_header(), 0xF7);
    }

    #[test]
    fn read_header_ten_bit_zero() {
        // addr=0x000: header = 11110_00_1 = 0xF1
        assert_eq!(Address::TenBit(0x000).read_header(), 0xF1);
    }

    #[test]
    fn read_header_ten_bit_max() {
        // addr=0x3FF: header = 11110_11_1 = 0xF7
        assert_eq!(Address::TenBit(0x3FF).read_header(), 0xF7);
    }

    #[test]
    fn read_header_ten_bit_all_upper_bit_combos() {
        assert_eq!(Address::TenBit(0x000).read_header(), 0xF1);
        assert_eq!(Address::TenBit(0x100).read_header(), 0xF3);
        assert_eq!(Address::TenBit(0x200).read_header(), 0xF5);
        assert_eq!(Address::TenBit(0x300).read_header(), 0xF7);
    }

    // ---- read_header is write_header | 1 ----

    #[test]
    fn read_header_equals_write_header_or_one() {
        for addr in [0x000u16, 0x055, 0x100, 0x1FF, 0x200, 0x2AB, 0x300, 0x3FF] {
            let a = Address::TenBit(addr);
            assert_eq!(
                a.read_header(),
                a.write_header() | 1,
                "mismatch at 10-bit addr {:#05x}",
                addr
            );
        }
        for addr in [0x00u8, 0x27, 0x50, 0x77, 0x78, 0x7B, 0x7F] {
            let a = Address::SevenBit(addr);
            assert_eq!(
                a.read_header(),
                a.write_header() | 1,
                "mismatch at 7-bit addr {:#04x}",
                addr
            );
        }
    }

    // ---- is_reserved_range ----

    #[test]
    fn reserved_range_boundaries() {
        assert!(!Address::SevenBit(0x77).is_reserved_range());
        assert!(Address::SevenBit(0x78).is_reserved_range());
        assert!(Address::SevenBit(0x79).is_reserved_range());
        assert!(Address::SevenBit(0x7A).is_reserved_range());
        assert!(Address::SevenBit(0x7B).is_reserved_range());
        assert!(!Address::SevenBit(0x7C).is_reserved_range());
    }

    #[test]
    fn reserved_range_common_addresses() {
        assert!(!Address::SevenBit(0x00).is_reserved_range());
        assert!(!Address::SevenBit(0x50).is_reserved_range());
        assert!(!Address::SevenBit(0x68).is_reserved_range());
    }

    #[test]
    fn reserved_range_ten_bit_is_never_reserved() {
        // TenBit addresses are true 10-bit, not "reserved-range 7-bit"
        assert!(!Address::TenBit(0x078).is_reserved_range());
        assert!(!Address::TenBit(0x0F0).is_reserved_range());
        assert!(!Address::TenBit(0x3FF).is_reserved_range());
    }

    // ---- 10-bit second address byte ----

    #[test]
    fn ten_bit_second_byte_is_low_byte() {
        assert_eq!(Address::TenBit(0x3FF).addr() as u8, 0xFF);
        assert_eq!(Address::TenBit(0x100).addr() as u8, 0x00);
        assert_eq!(Address::TenBit(0x0AB).addr() as u8, 0xAB);
    }

    // ---- impl Into<Address> accepts both u8 and Address ----

    fn takes_address(_addr: impl Into<Address>) {}

    #[test]
    fn into_address_accepts_u8() {
        takes_address(0x50u8);
    }

    #[test]
    fn into_address_accepts_address() {
        takes_address(Address::SevenBit(0x50));
        takes_address(Address::TenBit(0x123));
    }
}
