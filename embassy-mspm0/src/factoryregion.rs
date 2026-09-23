//! Factory region.
//!
//! The factory region is a read-only region of non-main flash containing device-specific
//! values programmed by TI during manufacturing, such as calibration constants.

use crate::pac;

/// Read the factory-region temperature sensor calibration constant.
///
/// This reads `FACTORYREGION.TEMP_SENSE0`, the reading of the internal temperature sensor
/// at the factory trim temperature, used to convert raw ADC readings to a temperature.
///
/// The value is a right-aligned 12-bit ADC code sampled with the 1.4V internal reference.
///
/// Only available on devices with an internal temperature sensor, see
/// [`adc::TempSenseChannel`](crate::adc::TempSenseChannel).
#[cfg(temp_sensor)]
#[must_use = "The calibration constant should be used to convert ADC readings to temperature"]
pub fn read_temp_calibration_constant() -> u32 {
    pac::FACTORYREGION.temp_sense0().read()
}
