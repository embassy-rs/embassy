//! Enums used in Hspi configuration.

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum HspiMode {
    IndirectWrite,
    IndirectRead,
    AutoPolling,
    MemoryMapped,
}

impl Into<u8> for HspiMode {
    fn into(self) -> u8 {
        match self {
            HspiMode::IndirectWrite => 0b00,
            HspiMode::IndirectRead => 0b01,
            HspiMode::AutoPolling => 0b10,
            HspiMode::MemoryMapped => 0b11,
        }
    }
}

/// Hspi lane width
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HspiWidth {
    /// None
    NONE,
    /// Single lane
    SING,
    /// Dual lanes
    DUAL,
    /// Quad lanes
    QUAD,
    /// Eight lanes
    OCTO,
    /// Sixteen lanes
    HEXADECA,
}

impl Into<u8> for HspiWidth {
    fn into(self) -> u8 {
        match self {
            HspiWidth::NONE => 0b00,
            HspiWidth::SING => 0b01,
            HspiWidth::DUAL => 0b10,
            HspiWidth::QUAD => 0b11,
            HspiWidth::OCTO => 0b100,
            HspiWidth::HEXADECA => 0b101,
        }
    }
}

/// Flash bank selection
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashSelection {
    /// Bank 1
    Flash1,
    /// Bank 2
    Flash2,
}

impl Into<bool> for FlashSelection {
    fn into(self) -> bool {
        match self {
            FlashSelection::Flash1 => false,
            FlashSelection::Flash2 => true,
        }
    }
}

/// Wrap Size
#[allow(dead_code)]
#[allow(missing_docs)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WrapSize {
    None,
    _16Bytes,
    _32Bytes,
    _64Bytes,
    _128Bytes,
}

impl Into<u8> for WrapSize {
    fn into(self) -> u8 {
        match self {
            WrapSize::None => 0x00,
            WrapSize::_16Bytes => 0x02,
            WrapSize::_32Bytes => 0x03,
            WrapSize::_64Bytes => 0x04,
            WrapSize::_128Bytes => 0x05,
        }
    }
}

/// Memory Type
#[allow(missing_docs)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemoryType {
    Micron,
    Macronix,
    Standard,
    MacronixRam,
    HyperBusMemory,
    HyperBusRegister,
}

impl Into<u8> for MemoryType {
    fn into(self) -> u8 {
        match self {
            MemoryType::Micron => 0x00,
            MemoryType::Macronix => 0x01,
            MemoryType::Standard => 0x02,
            MemoryType::MacronixRam => 0x03,
            MemoryType::HyperBusMemory => 0x04,
            MemoryType::HyperBusRegister => 0x04,
        }
    }
}

/// Hspi memory size.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemorySize {
    _1KiB,
    _2KiB,
    _4KiB,
    _8KiB,
    _16KiB,
    _32KiB,
    _64KiB,
    _128KiB,
    _256KiB,
    _512KiB,
    _1MiB,
    _2MiB,
    _4MiB,
    _8MiB,
    _16MiB,
    _32MiB,
    _64MiB,
    _128MiB,
    _256MiB,
    _512MiB,
    _1GiB,
    _2GiB,
    _4GiB,
    Other(u8),
}

impl Into<u8> for MemorySize {
    fn into(self) -> u8 {
        match self {
            MemorySize::_1KiB => 6,
            MemorySize::_2KiB => 7,
            MemorySize::_4KiB => 8,
            MemorySize::_8KiB => 9,
            MemorySize::_16KiB => 10,
            MemorySize::_32KiB => 11,
            MemorySize::_64KiB => 12,
            MemorySize::_128KiB => 13,
            MemorySize::_256KiB => 14,
            MemorySize::_512KiB => 15,
            MemorySize::_1MiB => 16,
            MemorySize::_2MiB => 17,
            MemorySize::_4MiB => 18,
            MemorySize::_8MiB => 19,
            MemorySize::_16MiB => 20,
            MemorySize::_32MiB => 21,
            MemorySize::_64MiB => 22,
            MemorySize::_128MiB => 23,
            MemorySize::_256MiB => 24,
            MemorySize::_512MiB => 25,
            MemorySize::_1GiB => 26,
            MemorySize::_2GiB => 27,
            MemorySize::_4GiB => 28,
            MemorySize::Other(val) => val,
        }
    }
}

/// Hspi Address size
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressSize {
    /// 8-bit address
    _8Bit,
    /// 16-bit address
    _16Bit,
    /// 24-bit address
    _24Bit,
    /// 32-bit address
    _32Bit,
}

impl Into<u8> for AddressSize {
    fn into(self) -> u8 {
        match self {
            AddressSize::_8Bit => 0b00,
            AddressSize::_16Bit => 0b01,
            AddressSize::_24Bit => 0b10,
            AddressSize::_32Bit => 0b11,
        }
    }
}

/// Time the Chip Select line stays high.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChipSelectHighTime {
    _1Cycle,
    _2Cycle,
    _3Cycle,
    _4Cycle,
    _5Cycle,
    _6Cycle,
    _7Cycle,
    _8Cycle,
}

impl Into<u8> for ChipSelectHighTime {
    fn into(self) -> u8 {
        match self {
            ChipSelectHighTime::_1Cycle => 0,
            ChipSelectHighTime::_2Cycle => 1,
            ChipSelectHighTime::_3Cycle => 2,
            ChipSelectHighTime::_4Cycle => 3,
            ChipSelectHighTime::_5Cycle => 4,
            ChipSelectHighTime::_6Cycle => 5,
            ChipSelectHighTime::_7Cycle => 6,
            ChipSelectHighTime::_8Cycle => 7,
        }
    }
}

/// FIFO threshold.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FIFOThresholdLevel {
    _1Bytes,
    _2Bytes,
    _3Bytes,
    _4Bytes,
    _5Bytes,
    _6Bytes,
    _7Bytes,
    _8Bytes,
    _9Bytes,
    _10Bytes,
    _11Bytes,
    _12Bytes,
    _13Bytes,
    _14Bytes,
    _15Bytes,
    _16Bytes,
    _17Bytes,
    _18Bytes,
    _19Bytes,
    _20Bytes,
    _21Bytes,
    _22Bytes,
    _23Bytes,
    _24Bytes,
    _25Bytes,
    _26Bytes,
    _27Bytes,
    _28Bytes,
    _29Bytes,
    _30Bytes,
    _31Bytes,
    _32Bytes,
}

impl Into<u8> for FIFOThresholdLevel {
    fn into(self) -> u8 {
        match self {
            FIFOThresholdLevel::_1Bytes => 0,
            FIFOThresholdLevel::_2Bytes => 1,
            FIFOThresholdLevel::_3Bytes => 2,
            FIFOThresholdLevel::_4Bytes => 3,
            FIFOThresholdLevel::_5Bytes => 4,
            FIFOThresholdLevel::_6Bytes => 5,
            FIFOThresholdLevel::_7Bytes => 6,
            FIFOThresholdLevel::_8Bytes => 7,
            FIFOThresholdLevel::_9Bytes => 8,
            FIFOThresholdLevel::_10Bytes => 9,
            FIFOThresholdLevel::_11Bytes => 10,
            FIFOThresholdLevel::_12Bytes => 11,
            FIFOThresholdLevel::_13Bytes => 12,
            FIFOThresholdLevel::_14Bytes => 13,
            FIFOThresholdLevel::_15Bytes => 14,
            FIFOThresholdLevel::_16Bytes => 15,
            FIFOThresholdLevel::_17Bytes => 16,
            FIFOThresholdLevel::_18Bytes => 17,
            FIFOThresholdLevel::_19Bytes => 18,
            FIFOThresholdLevel::_20Bytes => 19,
            FIFOThresholdLevel::_21Bytes => 20,
            FIFOThresholdLevel::_22Bytes => 21,
            FIFOThresholdLevel::_23Bytes => 22,
            FIFOThresholdLevel::_24Bytes => 23,
            FIFOThresholdLevel::_25Bytes => 24,
            FIFOThresholdLevel::_26Bytes => 25,
            FIFOThresholdLevel::_27Bytes => 26,
            FIFOThresholdLevel::_28Bytes => 27,
            FIFOThresholdLevel::_29Bytes => 28,
            FIFOThresholdLevel::_30Bytes => 29,
            FIFOThresholdLevel::_31Bytes => 30,
            FIFOThresholdLevel::_32Bytes => 31,
        }
    }
}

/// Dummy cycle count
#[allow(missing_docs)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DummyCycles {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18,
    _19,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
    _26,
    _27,
    _28,
    _29,
    _30,
    _31,
}

impl Into<u8> for DummyCycles {
    fn into(self) -> u8 {
        match self {
            DummyCycles::_0 => 0,
            DummyCycles::_1 => 1,
            DummyCycles::_2 => 2,
            DummyCycles::_3 => 3,
            DummyCycles::_4 => 4,
            DummyCycles::_5 => 5,
            DummyCycles::_6 => 6,
            DummyCycles::_7 => 7,
            DummyCycles::_8 => 8,
            DummyCycles::_9 => 9,
            DummyCycles::_10 => 10,
            DummyCycles::_11 => 11,
            DummyCycles::_12 => 12,
            DummyCycles::_13 => 13,
            DummyCycles::_14 => 14,
            DummyCycles::_15 => 15,
            DummyCycles::_16 => 16,
            DummyCycles::_17 => 17,
            DummyCycles::_18 => 18,
            DummyCycles::_19 => 19,
            DummyCycles::_20 => 20,
            DummyCycles::_21 => 21,
            DummyCycles::_22 => 22,
            DummyCycles::_23 => 23,
            DummyCycles::_24 => 24,
            DummyCycles::_25 => 25,
            DummyCycles::_26 => 26,
            DummyCycles::_27 => 27,
            DummyCycles::_28 => 28,
            DummyCycles::_29 => 29,
            DummyCycles::_30 => 30,
            DummyCycles::_31 => 31,
        }
    }
}

/// Functional mode
#[allow(missing_docs)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FunctionalMode {
    IndirectWrite,
    IndirectRead,
    AutoStatusPolling,
    MemoryMapped,
}

impl Into<u8> for FunctionalMode {
    fn into(self) -> u8 {
        match self {
            FunctionalMode::IndirectWrite => 0x00,
            FunctionalMode::IndirectRead => 0x01,
            FunctionalMode::AutoStatusPolling => 0x02,
            FunctionalMode::MemoryMapped => 0x03,
        }
    }
}
