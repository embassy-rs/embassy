use core::ops::BitOr;

/// Pin defines
#[allow(missing_docs)]
pub enum TscIOPin {
    Group1Io1,
    Group1Io2,
    Group1Io3,
    Group1Io4,
    Group2Io1,
    Group2Io2,
    Group2Io3,
    Group2Io4,
    Group3Io1,
    Group3Io2,
    Group3Io3,
    Group3Io4,
    Group4Io1,
    Group4Io2,
    Group4Io3,
    Group4Io4,
    Group5Io1,
    Group5Io2,
    Group5Io3,
    Group5Io4,
    Group6Io1,
    Group6Io2,
    Group6Io3,
    Group6Io4,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io1,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io2,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io3,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io4,
    #[cfg(tsc_v3)]
    Group8Io1,
    #[cfg(tsc_v3)]
    Group8Io2,
    #[cfg(tsc_v3)]
    Group8Io3,
    #[cfg(tsc_v3)]
    Group8Io4,
}

impl BitOr<TscIOPin> for u32 {
    type Output = u32;
    fn bitor(self, rhs: TscIOPin) -> Self::Output {
        let rhs: u32 = rhs.into();
        self | rhs
    }
}

impl BitOr<u32> for TscIOPin {
    type Output = u32;
    fn bitor(self, rhs: u32) -> Self::Output {
        let val: u32 = self.into();
        val | rhs
    }
}

impl BitOr for TscIOPin {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        let val: u32 = self.into();
        let rhs: u32 = rhs.into();
        val | rhs
    }
}

impl Into<u32> for TscIOPin {
    fn into(self) -> u32 {
        match self {
            TscIOPin::Group1Io1 => 0x00000001,
            TscIOPin::Group1Io2 => 0x00000002,
            TscIOPin::Group1Io3 => 0x00000004,
            TscIOPin::Group1Io4 => 0x00000008,
            TscIOPin::Group2Io1 => 0x00000010,
            TscIOPin::Group2Io2 => 0x00000020,
            TscIOPin::Group2Io3 => 0x00000040,
            TscIOPin::Group2Io4 => 0x00000080,
            TscIOPin::Group3Io1 => 0x00000100,
            TscIOPin::Group3Io2 => 0x00000200,
            TscIOPin::Group3Io3 => 0x00000400,
            TscIOPin::Group3Io4 => 0x00000800,
            TscIOPin::Group4Io1 => 0x00001000,
            TscIOPin::Group4Io2 => 0x00002000,
            TscIOPin::Group4Io3 => 0x00004000,
            TscIOPin::Group4Io4 => 0x00008000,
            TscIOPin::Group5Io1 => 0x00010000,
            TscIOPin::Group5Io2 => 0x00020000,
            TscIOPin::Group5Io3 => 0x00040000,
            TscIOPin::Group5Io4 => 0x00080000,
            TscIOPin::Group6Io1 => 0x00100000,
            TscIOPin::Group6Io2 => 0x00200000,
            TscIOPin::Group6Io3 => 0x00400000,
            TscIOPin::Group6Io4 => 0x00800000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io1 => 0x01000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io2 => 0x02000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io3 => 0x04000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io4 => 0x08000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io1 => 0x10000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io2 => 0x20000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io3 => 0x40000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io4 => 0x80000000,
        }
    }
}

/// Spread Spectrum Deviation
#[derive(Copy, Clone)]
pub struct SSDeviation(u8);
impl SSDeviation {
    /// Create new deviation value, acceptable inputs are 1-128
    pub fn new(val: u8) -> Result<Self, ()> {
        if val == 0 || val > 128 {
            return Err(());
        }
        Ok(Self(val - 1))
    }
}

impl Into<u8> for SSDeviation {
    fn into(self) -> u8 {
        self.0
    }
}

/// Charge transfer pulse cycles
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq)]
pub enum ChargeTransferPulseCycle {
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
}

impl Into<u8> for ChargeTransferPulseCycle {
    fn into(self) -> u8 {
        match self {
            ChargeTransferPulseCycle::_1 => 0,
            ChargeTransferPulseCycle::_2 => 1,
            ChargeTransferPulseCycle::_3 => 2,
            ChargeTransferPulseCycle::_4 => 3,
            ChargeTransferPulseCycle::_5 => 4,
            ChargeTransferPulseCycle::_6 => 5,
            ChargeTransferPulseCycle::_7 => 6,
            ChargeTransferPulseCycle::_8 => 7,
            ChargeTransferPulseCycle::_9 => 8,
            ChargeTransferPulseCycle::_10 => 9,
            ChargeTransferPulseCycle::_11 => 10,
            ChargeTransferPulseCycle::_12 => 11,
            ChargeTransferPulseCycle::_13 => 12,
            ChargeTransferPulseCycle::_14 => 13,
            ChargeTransferPulseCycle::_15 => 14,
            ChargeTransferPulseCycle::_16 => 15,
        }
    }
}

/// Prescaler divider
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq)]
pub enum PGPrescalerDivider {
    _1,
    _2,
    _4,
    _8,
    _16,
    _32,
    _64,
    _128,
}

impl Into<u8> for PGPrescalerDivider {
    fn into(self) -> u8 {
        match self {
            PGPrescalerDivider::_1 => 0,
            PGPrescalerDivider::_2 => 1,
            PGPrescalerDivider::_4 => 2,
            PGPrescalerDivider::_8 => 3,
            PGPrescalerDivider::_16 => 4,
            PGPrescalerDivider::_32 => 5,
            PGPrescalerDivider::_64 => 6,
            PGPrescalerDivider::_128 => 7,
        }
    }
}

/// Max count
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum MaxCount {
    _255,
    _511,
    _1023,
    _2047,
    _4095,
    _8191,
    _16383,
}

impl Into<u8> for MaxCount {
    fn into(self) -> u8 {
        match self {
            MaxCount::_255 => 0,
            MaxCount::_511 => 1,
            MaxCount::_1023 => 2,
            MaxCount::_2047 => 3,
            MaxCount::_4095 => 4,
            MaxCount::_8191 => 5,
            MaxCount::_16383 => 6,
        }
    }
}
