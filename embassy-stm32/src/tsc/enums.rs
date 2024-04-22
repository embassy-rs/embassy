/// Charge transfer pulse cycles
#[allow(missing_docs)]
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
