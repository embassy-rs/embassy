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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    /// Sixteen lanes (Hexadeca-SPI)
    HEXA,
}

impl Into<u8> for XspiWidth {
    fn into(self) -> u8 {
        match self {
            XspiWidth::NONE => 0b00,
            XspiWidth::SING => 0b01,
            XspiWidth::DUAL => 0b10,
            XspiWidth::QUAD => 0b11,
            XspiWidth::OCTO => 0b100,
            XspiWidth::HEXA => 0b101,
        }
    }
}

/// Wrap Size
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
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemoryType {
    Micron,
    Macronix,
    Standard,
    MacronixRam,
    HyperBusMemory,
    HyperBusRegister,
    APMemory16Bits, // AP Memory 16-bit (for  PSRAM in X8/X16 mode)
    APMemory,       //The same as Standard
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
            MemoryType::APMemory16Bits => 0x06,
            MemoryType::APMemory => 0x02,
        }
    }
}

/// Xspi memory size.
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    _9Cycle,
    _10Cycle,
    _11Cycle,
    _12Cycle,
    _13Cycle,
    _14Cycle,
    _15Cycle,
    _16Cycle,
    _17Cycle,
    _18Cycle,
    _19Cycle,
    _20Cycle,
    _21Cycle,
    _22Cycle,
    _23Cycle,
    _24Cycle,
    _25Cycle,
    _26Cycle,
    _27Cycle,
    _28Cycle,
    _29Cycle,
    _30Cycle,
    _31Cycle,
    _32Cycle,
    _33Cycle,
    _34Cycle,
    _35Cycle,
    _36Cycle,
    _37Cycle,
    _38Cycle,
    _39Cycle,
    _40Cycle,
    _41Cycle,
    _42Cycle,
    _43Cycle,
    _44Cycle,
    _45Cycle,
    _46Cycle,
    _47Cycle,
    _48Cycle,
    _49Cycle,
    _50Cycle,
    _51Cycle,
    _52Cycle,
    _53Cycle,
    _54Cycle,
    _55Cycle,
    _56Cycle,
    _57Cycle,
    _58Cycle,
    _59Cycle,
    _60Cycle,
    _61Cycle,
    _62Cycle,
    _63Cycle,
    _64Cycle,
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
            ChipSelectHighTime::_9Cycle => 8,
            ChipSelectHighTime::_10Cycle => 9,
            ChipSelectHighTime::_11Cycle => 10,
            ChipSelectHighTime::_12Cycle => 11,
            ChipSelectHighTime::_13Cycle => 12,
            ChipSelectHighTime::_14Cycle => 13,
            ChipSelectHighTime::_15Cycle => 14,
            ChipSelectHighTime::_16Cycle => 15,
            ChipSelectHighTime::_17Cycle => 16,
            ChipSelectHighTime::_18Cycle => 17,
            ChipSelectHighTime::_19Cycle => 18,
            ChipSelectHighTime::_20Cycle => 19,
            ChipSelectHighTime::_21Cycle => 20,
            ChipSelectHighTime::_22Cycle => 21,
            ChipSelectHighTime::_23Cycle => 22,
            ChipSelectHighTime::_24Cycle => 23,
            ChipSelectHighTime::_25Cycle => 24,
            ChipSelectHighTime::_26Cycle => 25,
            ChipSelectHighTime::_27Cycle => 26,
            ChipSelectHighTime::_28Cycle => 27,
            ChipSelectHighTime::_29Cycle => 28,
            ChipSelectHighTime::_30Cycle => 29,
            ChipSelectHighTime::_31Cycle => 30,
            ChipSelectHighTime::_32Cycle => 31,
            ChipSelectHighTime::_33Cycle => 32,
            ChipSelectHighTime::_34Cycle => 33,
            ChipSelectHighTime::_35Cycle => 34,
            ChipSelectHighTime::_36Cycle => 35,
            ChipSelectHighTime::_37Cycle => 36,
            ChipSelectHighTime::_38Cycle => 37,
            ChipSelectHighTime::_39Cycle => 38,
            ChipSelectHighTime::_40Cycle => 39,
            ChipSelectHighTime::_41Cycle => 40,
            ChipSelectHighTime::_42Cycle => 41,
            ChipSelectHighTime::_43Cycle => 42,
            ChipSelectHighTime::_44Cycle => 43,
            ChipSelectHighTime::_45Cycle => 44,
            ChipSelectHighTime::_46Cycle => 45,
            ChipSelectHighTime::_47Cycle => 46,
            ChipSelectHighTime::_48Cycle => 47,
            ChipSelectHighTime::_49Cycle => 48,
            ChipSelectHighTime::_50Cycle => 49,
            ChipSelectHighTime::_51Cycle => 50,
            ChipSelectHighTime::_52Cycle => 51,
            ChipSelectHighTime::_53Cycle => 52,
            ChipSelectHighTime::_54Cycle => 53,
            ChipSelectHighTime::_55Cycle => 54,
            ChipSelectHighTime::_56Cycle => 55,
            ChipSelectHighTime::_57Cycle => 56,
            ChipSelectHighTime::_58Cycle => 57,
            ChipSelectHighTime::_59Cycle => 58,
            ChipSelectHighTime::_60Cycle => 59,
            ChipSelectHighTime::_61Cycle => 60,
            ChipSelectHighTime::_62Cycle => 61,
            ChipSelectHighTime::_63Cycle => 62,
            ChipSelectHighTime::_64Cycle => 63,
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
