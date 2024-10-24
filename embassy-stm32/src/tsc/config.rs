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

/// Error type for SSDeviation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SSDeviationError {
    /// The provided value is too low (0)
    ValueTooLow,
    /// The provided value is too high (greater than 128)
    ValueTooHigh,
}

/// Spread Spectrum Deviation
#[derive(Copy, Clone)]
pub struct SSDeviation(u8);
impl SSDeviation {
    /// Create new deviation value, acceptable inputs are 1-128
    pub fn new(val: u8) -> Result<Self, SSDeviationError> {
        if val == 0 {
            return Err(SSDeviationError::ValueTooLow);
        } else if val > 128 {
            return Err(SSDeviationError::ValueTooHigh);
        }
        Ok(Self(val - 1))
    }
}

impl Into<u8> for SSDeviation {
    fn into(self) -> u8 {
        self.0
    }
}

/// Peripheral configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// Duration of high state of the charge transfer pulse
    pub ct_pulse_high_length: ChargeTransferPulseCycle,
    /// Duration of the low state of the charge transfer pulse
    pub ct_pulse_low_length: ChargeTransferPulseCycle,
    /// Enable/disable of spread spectrum feature
    pub spread_spectrum: bool,
    /// Adds variable number of periods of the SS clk to pulse high state
    pub spread_spectrum_deviation: SSDeviation,
    /// Selects AHB clock divider used to generate SS clk
    pub spread_spectrum_prescaler: bool,
    /// Selects AHB clock divider used to generate pulse generator clk
    pub pulse_generator_prescaler: PGPrescalerDivider,
    /// Maximum number of charge transfer pulses that can be generated before error
    pub max_count_value: MaxCount,
    /// Defines config of all IOs when no ongoing acquisition
    pub io_default_mode: bool,
    /// Polarity of sync input pin
    pub synchro_pin_polarity: bool,
    /// Acquisition starts when start bit is set or with sync pin input
    pub acquisition_mode: bool,
    /// Enable max count interrupt
    pub max_count_interrupt: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ct_pulse_high_length: ChargeTransferPulseCycle::_1,
            ct_pulse_low_length: ChargeTransferPulseCycle::_1,
            spread_spectrum: false,
            spread_spectrum_deviation: SSDeviation::new(1).unwrap(),
            spread_spectrum_prescaler: false,
            pulse_generator_prescaler: PGPrescalerDivider::_1,
            max_count_value: MaxCount::_255,
            io_default_mode: false,
            synchro_pin_polarity: false,
            acquisition_mode: false,
            max_count_interrupt: false,
        }
    }
}
