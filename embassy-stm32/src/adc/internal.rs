//! Internal ADC channels (VREFINT, temperature sensor, VBAT, ...).
//!
//! Each internal source is a zero-sized token implementing [`AdcChannel`] for the ADC instances
//! that have it connected, with the channel number of the source on that instance. The tokens are
//! obtained from the `enable_*` methods of [`Adc`], which switch the source on.

use super::{Adc, AdcChannel, AdcRegs, Instance, InternalChannel, SealedAdcChannel};
use crate::mode::Mode;

/// Trait implemented by the internal channel tokens.
pub(crate) trait SpecialChannel {}

/// `T` can convert internal channel `C`, on hardware channel `CHANNEL`.
pub(crate) trait ConverterFor<C: SpecialChannel> {
    const CHANNEL: u8;
}

/// ADC instances that have the internal channel `C`.
#[allow(private_bounds)]
pub trait HasInternalChannel<C: SpecialChannel>: ConverterFor<C> {}
impl<C: SpecialChannel, T: ConverterFor<C>> HasInternalChannel<C> for T {}

impl<'d, C: SpecialChannel, T: Instance + ConverterFor<C>> AdcChannel<'d, T> for C {}
impl<C: SpecialChannel, T: Instance + ConverterFor<C>> SealedAdcChannel<T> for C {
    fn channel(&self) -> u8 {
        T::CHANNEL
    }
}

macro_rules! token {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        pub struct $name;
        impl SpecialChannel for $name {}
    };
}

token!(
    /// Internal voltage reference channel.
    VrefInt
);
token!(
    /// Internal temperature sensor channel.
    Temperature
);
token!(
    /// Backup battery voltage channel (VBAT divided by 2, 3 or 4 depending on the chip).
    Vbat
);
token!(
    /// Core supply voltage channel.
    VddCore
);
token!(
    /// DAC output channel, internally connected to the ADC.
    Dac
);

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    /// Enable the internal voltage reference channel.
    pub fn enable_vrefint(&mut self) -> VrefInt
    where
        T: HasInternalChannel<VrefInt>,
    {
        self.enable_internal(InternalChannel::VrefInt);
        VrefInt
    }

    /// Enable the internal temperature sensor channel.
    ///
    /// The sensor needs a long sample time (see the datasheet for `ts_temp`), and on some chips
    /// shares its channel with VBAT: enabling both returns VBAT.
    pub fn enable_temperature(&mut self) -> Temperature
    where
        T: HasInternalChannel<Temperature>,
    {
        self.enable_internal(InternalChannel::Temperature);
        Temperature
    }

    /// Enable the VBAT channel. This connects VBAT to an internal divider, which draws current
    /// from the battery for as long as it is enabled; use [`disable_vbat`](Self::disable_vbat)
    /// when done.
    pub fn enable_vbat(&mut self) -> Vbat
    where
        T: HasInternalChannel<Vbat>,
    {
        self.enable_internal(InternalChannel::Vbat);
        Vbat
    }

    /// Disconnect the VBAT divider.
    pub fn disable_vbat(&mut self, _vbat: Vbat)
    where
        T: HasInternalChannel<Vbat>,
    {
        T::regs().enable_internal(T::common(), InternalChannel::Vbat, false);
    }

    /// Enable the core supply voltage channel.
    pub fn enable_vddcore(&mut self) -> VddCore
    where
        T: HasInternalChannel<VddCore>,
    {
        self.enable_internal(InternalChannel::VddCore);
        VddCore
    }

    /// Enable the internal connection to the DAC output.
    ///
    /// `dac_channel` selects which DAC output is connected on chips that mux several onto one ADC
    /// channel (0 for `OUT1`, 1 for `OUT2`); it is ignored elsewhere.
    pub fn enable_dac(&mut self, dac_channel: u8) -> Dac
    where
        T: HasInternalChannel<Dac>,
    {
        self.enable_internal(InternalChannel::Dac(dac_channel));
        Dac
    }
}

macro_rules! internal {
    ($inst:ident: $($tok:ident = $ch:expr),* $(,)?) => {
        $(
            impl ConverterFor<$tok> for crate::peripherals::$inst {
                const CHANNEL: u8 = $ch;
            }
        )*
    };
    ($inst:ident, $($insts:ident),+: $($tok:ident = $ch:expr),* $(,)?) => {
        internal!($inst: $($tok = $ch),*);
        internal!($($insts),+: $($tok = $ch),*);
    };
}

// F0
#[cfg(adc_v2_f0)]
internal!(ADC1: Temperature = 16, VrefInt = 17, Vbat = 18);
// L0
#[cfg(adc_v2_l0)]
internal!(ADC1: VrefInt = 17, Temperature = 18);
// WB10/WB15
#[cfg(adc_v2_wb1)]
internal!(ADC1: Temperature = 12, VrefInt = 13, Vbat = 14);
// G0, WL
#[cfg(all(adc_v2_g0, any(stm32g0, stm32wl)))]
internal!(ADC1: Temperature = 12, VrefInt = 13, Vbat = 14);
#[cfg(all(adc_v2_g0, stm32wl))]
internal!(ADC1: Dac = 17);
// U0
#[cfg(all(adc_v2_g0, stm32u0))]
internal!(ADC1: Temperature = 11, VrefInt = 12, Vbat = 13, Dac = 19);
// C0
#[cfg(all(adc_v2_g0, stm32c0))]
internal!(ADC1: Temperature = 9, VrefInt = 10);
// U5 ADC4 and WBA ADC4
#[cfg(any(adc_v2_u5, adc_v2_wba))]
internal!(ADC4: VrefInt = 0, VddCore = 12, Temperature = 13);
#[cfg(adc_v2_u5)]
internal!(ADC4: Vbat = 14, Dac = 21);
// U5 ADC1/ADC2
#[cfg(adc_v3_u5)]
internal!(ADC1: VrefInt = 0, Vbat = 18, Temperature = 19);
#[cfg(all(adc_v3_u5, peri_adc2))]
internal!(ADC2: VrefInt = 0, Vbat = 18, Temperature = 19);

// F1
#[cfg(adc_v1_f1)]
internal!(ADC1: Temperature = 16, VrefInt = 17);
// F2, F4, F7: the temperature sensor moved to channel 18 (shared with VBAT) on F42x and later.
#[cfg(all(adc_v1_f4, any(stm32f2, stm32f40x, stm32f41x)))]
internal!(ADC1: Temperature = 16, VrefInt = 17, Vbat = 18);
#[cfg(all(adc_v1_f4, not(any(stm32f2, stm32f40x, stm32f41x))))]
internal!(ADC1: VrefInt = 17, Temperature = 18, Vbat = 18);
// L1
#[cfg(adc_v1_l1)]
internal!(ADC1: Temperature = 16, VrefInt = 17);

// F30x
#[cfg(adc_v3_f3)]
internal!(ADC1: Temperature = 16, Vbat = 17, VrefInt = 18);
#[cfg(all(adc_v3_f3, peri_adc2))]
internal!(ADC2: VrefInt = 18);
#[cfg(all(adc_v3_f3, peri_adc3))]
internal!(ADC3: VrefInt = 18);
#[cfg(all(adc_v3_f3, peri_adc4))]
internal!(ADC4: VrefInt = 18);
// L4, L5, WB55
#[cfg(adc_v3_l4)]
internal!(ADC1: VrefInt = 0, Temperature = 17, Vbat = 18);
// G4
#[cfg(all(adc_v3_g4, stm32g4))]
internal!(ADC1: Temperature = 16, Vbat = 17, VrefInt = 18);
#[cfg(all(adc_v3_g4, stm32g4, peri_adc3))]
internal!(ADC3: Vbat = 17, VrefInt = 18);
#[cfg(all(adc_v3_g4, stm32g4, peri_adc4))]
internal!(ADC4: VrefInt = 18);
#[cfg(all(adc_v3_g4, stm32g4, peri_adc5))]
internal!(ADC5: Temperature = 4, Vbat = 17, VrefInt = 18);
// H5, H7RS
#[cfg(all(adc_v3_g4, any(stm32h5, stm32h7rs)))]
internal!(ADC1: Temperature = 16, VrefInt = 17);
#[cfg(all(adc_v3_g4, stm32h50x))]
internal!(ADC1: Vbat = 2);
#[cfg(all(adc_v3_g4, any(stm32h5, stm32h7rs), peri_adc2))]
internal!(ADC2: Vbat = 16, VddCore = 17);
// H7
#[cfg(all(adc_v3_h7, any(stm32h72x, stm32h73x)))]
internal!(ADC3: Vbat = 16, Temperature = 17, VrefInt = 18);
#[cfg(all(adc_v3_h7, any(stm32h74x, stm32h75x)))]
internal!(ADC3: Vbat = 17, Temperature = 18, VrefInt = 19);
#[cfg(all(adc_v3_h7, any(stm32h7ax, stm32h7bx)))]
internal!(ADC2: Vbat = 14, Temperature = 18, VrefInt = 19);
// U3
#[cfg(adc_v3_u3)]
internal!(ADC1, ADC2: VrefInt = 0, Vbat = 16, Temperature = 17);
// N6: the temperature sensor is a separate peripheral (DTS).
#[cfg(adc_v3_n6)]
internal!(ADC1, ADC2: VrefInt = 17);
#[cfg(adc_v3_n6)]
internal!(ADC2: Vbat = 16);
// C5
#[cfg(adc_v3_c5)]
internal!(ADC1: Temperature = 12, VrefInt = 13);

impl VrefInt {
    /// The reading the internal reference gives when VDDA is at the factory calibration voltage
    /// `VREF_CALIB_MV`, measured at production and stored in flash.
    #[cfg(any(
        stm32f0,
        stm32f3,
        stm32f7,
        stm32g0,
        stm32g4,
        stm32l0,
        stm32l1,
        stm32l4,
        stm32l4_plus,
        stm32l5,
        stm32wb,
        stm32wl,
        stm32u0,
    ))]
    pub fn calibrated_value(&self) -> u16 {
        crate::pac::VREFINTCAL.data().read()
    }
}

impl Temperature {
    /// The temperature sensor factory calibration readings at 30 °C and 130 °C.
    #[cfg(stm32u0)]
    pub fn calibrated_value(&self) -> (u16, u16) {
        let lower = crate::pac::TSCAL.tscal1().read();
        let upper = crate::pac::TSCAL.tscal2().read();
        (lower, upper)
    }
}

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;

/// VREF voltage used for factory calibration of VREFINTCAL register.
#[cfg(any(
    stm32l4,
    stm32l4_plus,
    stm32l5,
    stm32wb,
    stm32wl,
    stm32g0,
    stm32g4,
    stm32u0,
    stm32wba,
    stm32u5,
    stm32u3
))]
pub const VREF_CALIB_MV: u32 = 3000;
/// VREF voltage used for factory calibration of VREFINTCAL register.
#[cfg(not(any(
    stm32l4,
    stm32l4_plus,
    stm32l5,
    stm32wb,
    stm32wl,
    stm32g0,
    stm32g4,
    stm32u0,
    stm32wba,
    stm32u5,
    stm32u3
)))]
pub const VREF_CALIB_MV: u32 = 3300;

/// Temperature at which TS_CAL1 was measured (30°C).
pub const TS_CAL1_TEMP_C: i32 = 30;
/// Temperature at which TS_CAL2 was measured (130°C).
pub const TS_CAL2_TEMP_C: i32 = 130;

/// Factory calibration values read from the DESIG peripheral.
///
/// These values are programmed during manufacturing and can be used
/// for accurate temperature and voltage measurements.
#[cfg(stm32wba)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Calibration {
    /// Temperature sensor calibration value at 30°C (12-bit).
    pub ts_cal1: u16,
    /// Temperature sensor calibration value at 130°C (12-bit).
    pub ts_cal2: u16,
    /// Internal voltage reference calibration value (12-bit).
    /// Measured at VDDA = 3.0V.
    pub vrefint_cal: u16,
}

#[cfg(stm32wba)]
impl Calibration {
    /// Read factory calibration values from the DESIG and VREFINTCAL peripherals.
    ///
    /// These values are unique to each chip and were measured during manufacturing
    /// at VDDA = 3.0V.
    pub fn read() -> Self {
        Self {
            ts_cal1: crate::pac::DESIG.tscal1r().read().ts_cal1(),
            ts_cal2: crate::pac::DESIG.tscal2r().read().ts_cal2(),
            vrefint_cal: crate::pac::VREFINTCAL.data().read().vrefint_cal(),
        }
    }

    /// Convert a temperature sensor ADC reading to temperature in millidegrees Celsius.
    ///
    /// This function applies VDDA compensation using the VREFINT reading to account
    /// for differences between the actual supply voltage and the 3.0V calibration voltage.
    ///
    /// # Arguments
    /// * `ts_data` - Raw ADC reading from the temperature sensor channel
    /// * `vrefint_data` - Raw ADC reading from the VREFINT channel (for VDDA compensation)
    ///
    /// # Returns
    /// Temperature in millidegrees Celsius (e.g., 25000 = 25.000°C)
    pub fn convert_to_millicelsius(&self, ts_data: u32, vrefint_data: u32) -> i32 {
        // Compensate TS_DATA for actual VDDA vs calibration VDDA (3.0V)
        let ts_data_comp = if vrefint_data > 0 {
            (ts_data * self.vrefint_cal as u32) / vrefint_data
        } else {
            ts_data
        };

        let ts_data_comp = ts_data_comp as i32;
        let ts_cal1 = self.ts_cal1 as i32;
        let ts_cal2 = self.ts_cal2 as i32;

        let delta_temp = (TS_CAL2_TEMP_C - TS_CAL1_TEMP_C) * 1000;
        let delta_cal = ts_cal2 - ts_cal1;

        if delta_cal == 0 {
            return ts_data_comp * 10;
        }

        TS_CAL1_TEMP_C * 1000 + (delta_temp * (ts_data_comp - ts_cal1)) / delta_cal
    }

    /// Calculate the actual VDDA voltage in millivolts using VREFINT.
    ///
    /// The formula is: VDDA = 3000mV × VREFINT_CAL / VREFINT_DATA
    pub fn calculate_vdda_mv(&self, vrefint_data: u32) -> u32 {
        if vrefint_data > 0 {
            (VREF_CALIB_MV * self.vrefint_cal as u32) / vrefint_data
        } else {
            VREF_DEFAULT_MV
        }
    }
}
