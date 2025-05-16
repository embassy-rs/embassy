//! Enums used in Xspi configuration.
#[derive(Copy, Clone)]
pub(crate) enum XspiMode {
    IndirectWrite,
    IndirectRead,
    #[expect(dead_code)]
    AutoPolling,
    #[expect(dead_code)]
    MemoryMapped,
}

impl Into<u8> for XspiMode {
    fn into(self) -> u8 {
        match self {
            XspiMode::IndirectWrite => 0b00,
            XspiMode::IndirectRead => 0b01,
            XspiMode::AutoPolling => 0b10,
            XspiMode::MemoryMapped => 0b11,
        }
    }
}

/// Xspi lane width
#[derive(Copy, Clone)]
pub enum XspiWidth {
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
}

impl Into<u8> for XspiWidth {
    fn into(self) -> u8 {
        match self {
            XspiWidth::NONE => 0b00,
            XspiWidth::SING => 0b01,
            XspiWidth::DUAL => 0b10,
            XspiWidth::QUAD => 0b11,
            XspiWidth::OCTO => 0b100,
        }
    }
}

/// Wrap Size
#[allow(missing_docs)]
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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

/// Xspi memory size.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
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
            MemorySize::_1KiB => 9,
            MemorySize::_2KiB => 10,
            MemorySize::_4KiB => 11,
            MemorySize::_8KiB => 12,
            MemorySize::_16KiB => 13,
            MemorySize::_32KiB => 14,
            MemorySize::_64KiB => 15,
            MemorySize::_128KiB => 16,
            MemorySize::_256KiB => 17,
            MemorySize::_512KiB => 18,
            MemorySize::_1MiB => 19,
            MemorySize::_2MiB => 20,
            MemorySize::_4MiB => 21,
            MemorySize::_8MiB => 22,
            MemorySize::_16MiB => 23,
            MemorySize::_32MiB => 24,
            MemorySize::_64MiB => 25,
            MemorySize::_128MiB => 26,
            MemorySize::_256MiB => 27,
            MemorySize::_512MiB => 28,
            MemorySize::_1GiB => 29,
            MemorySize::_2GiB => 30,
            MemorySize::_4GiB => 31,
            MemorySize::Other(val) => val,
        }
    }
}

/// Xspi Address size
#[derive(Copy, Clone)]
pub enum AddressSize {
    /// 8-bit address
    _8bit,
    /// 16-bit address
    _16bit,
    /// 24-bit address
    _24bit,
    /// 32-bit address
    _32bit,
}

impl Into<u8> for AddressSize {
    fn into(self) -> u8 {
        match self {
            AddressSize::_8bit => 0b00,
            AddressSize::_16bit => 0b01,
            AddressSize::_24bit => 0b10,
            AddressSize::_32bit => 0b11,
        }
    }
}

/// Time the Chip Select line stays high.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
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
