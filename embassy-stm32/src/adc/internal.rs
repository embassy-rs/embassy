//! Internal ADC channels (VREFINT, temperature sensor, VBAT, ...).
//!
//! Each internal source is a zero-sized token implementing [`AdcChannel`] for the ADC instances
//! that have it connected, with the channel number of the source on that instance. The tokens are
//! obtained from the `enable_*` methods of [`Adc`], which switch the source on.

use super::{
    Adc, AdcChannel, AdcRegs, BorrowedChannel, Instance, InternalChannel, Resolution, SampleTimeOf, SealedAdcChannel,
};
use crate::mode::{Async, Mode};

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

#[allow(unused_macros)]
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
internal!(ADC1: Temperature = 16, VrefInt = 17);
// RM0360 (F030/F070) has no VBAT channel.
#[cfg(all(adc_v2_f0, not(stm32f0x0)))]
internal!(ADC1: Vbat = 18);
// L0
#[cfg(adc_v2_l0)]
internal!(ADC1: VrefInt = 17);
// RM0451 (L0x0) has no temperature sensor.
#[cfg(all(adc_v2_l0, not(stm32l0x0)))]
internal!(ADC1: Temperature = 18);
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
internal!(ADC1, ADC2: VrefInt = 0, Vbat = 16, Temperature = 17, VddCore = 18);
// N6: the temperature sensor is a separate peripheral (DTS).
#[cfg(adc_v3_n6)]
internal!(ADC1, ADC2: VrefInt = 17);
#[cfg(adc_v3_n6)]
internal!(ADC2: Vbat = 16, VddCore = 17);
// C5
#[cfg(adc_v3_c5)]
internal!(ADC1: Temperature = 12, VrefInt = 13);

impl VrefInt {
    /// The reading the internal reference gives when VDDA is at the factory calibration voltage
    /// [`VREF_CALIB_MV`], measured at production and stored in flash.
    #[cfg(adc_vrefint_cal)]
    pub fn calibrated_value(&self) -> u16 {
        read_vrefint_cal()
    }
}

impl Temperature {
    /// The temperature sensor factory calibration readings at [`TS_CAL1_TEMP_C`] and
    /// [`TS_CAL2_TEMP_C`].
    #[cfg(adc_temp_two_point)]
    pub fn calibrated_value(&self) -> (u16, u16) {
        (read_ts_cal1(), read_ts_cal2())
    }

    /// The single-point temperature sensor factory calibration reading at [`TS_CAL1_TEMP_C`].
    #[cfg(adc_temp_cal)]
    pub fn cal1(&self) -> u16 {
        read_ts_cal1()
    }
}

#[cfg(adc_vrefint_cal)]
#[inline]
fn read_vrefint_cal() -> u16 {
    crate::pac::VREFINTCAL.data().read() & cal_mask(crate::_generated::ADC_VREF_CAL_BITS)
}

#[cfg(adc_temp_cal)]
#[inline]
fn read_ts_cal1() -> u16 {
    crate::pac::TS_CAL1.data().read() & cal_mask(crate::_generated::ADC_TEMP_CAL_BITS)
}

#[cfg(adc_temp_two_point)]
#[inline]
fn read_ts_cal2() -> u16 {
    crate::pac::TS_CAL2.data().read() & cal_mask(crate::_generated::ADC_TEMP_CAL_BITS)
}

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;

/// VREF voltage used for factory calibration of the `VREFINTCAL` register.
#[cfg(adc_vrefint_cal)]
pub const VREF_CALIB_MV: u32 = crate::_generated::ADC_VREF_CAL_MV;
/// VREF voltage used for factory calibration of the `VREFINTCAL` register.
#[cfg(not(adc_vrefint_cal))]
pub const VREF_CALIB_MV: u32 = 3300;

/// Bit width of the factory-programmed `VREFINTCAL` value.
#[cfg(adc_vrefint_cal)]
pub const VREF_CALIB_BITS: u8 = crate::_generated::ADC_VREF_CAL_BITS;

/// Temperature at which `TS_CAL1` was measured (typically 30 °C).
#[cfg(adc_temp_cal)]
pub const TS_CAL1_TEMP_C: i32 = crate::_generated::ADC_TEMP_CAL1_C;
/// Temperature at which `TS_CAL1` was measured (30 °C).
#[cfg(not(adc_temp_cal))]
pub const TS_CAL1_TEMP_C: i32 = 30;

/// Temperature at which `TS_CAL2` was measured (110 °C, 130 °C, or 140 °C depending on chip).
#[cfg(adc_temp_two_point)]
pub const TS_CAL2_TEMP_C: i32 = crate::_generated::ADC_TEMP_CAL2_C;
/// Temperature at which `TS_CAL2` was measured (130 °C).
#[cfg(not(adc_temp_two_point))]
pub const TS_CAL2_TEMP_C: i32 = 130;

#[inline]
#[allow(dead_code)]
const fn cal_mask(bits: u8) -> u16 {
    ((1u32 << bits) - 1) as u16
}

#[inline]
#[allow(dead_code)]
const fn bit_max_count(bits: u8) -> u32 {
    (1u32 << bits) - 1
}

/// Convert a raw ADC sample acquired at `resolution` to millivolts using `vdda_mv`.
#[inline]
pub const fn raw_to_mv(sample: u32, resolution: Resolution, vdda_mv: u32) -> u32 {
    raw_count_to_mv(sample, resolution.max_count(), vdda_mv)
}

#[inline]
const fn raw_count_to_mv(sample: u32, max_count: u32, vdda_mv: u32) -> u32 {
    if max_count == 0 {
        return 0;
    }
    ((sample as u64 * vdda_mv as u64) / max_count as u64) as u32
}

#[inline]
#[allow(dead_code)]
const fn calc_vdda_uv(
    vref_cal: u16,
    vref_cal_mv: u32,
    vref_cal_bits: u8,
    vrefint_data: u32,
    adc_max_count: u32,
    default_vdda_mv: u32,
) -> u64 {
    if vrefint_data == 0 {
        return (default_vdda_mv as u64) * 1000;
    }
    let cal_max = bit_max_count(vref_cal_bits) as u64;
    (vref_cal_mv as u64 * 1000 * vref_cal as u64 * adc_max_count as u64) / (vrefint_data as u64 * cal_max)
}

#[inline]
#[allow(dead_code)]
const fn sample_uv_to_mv(sample: u32, sample_max_count: u32, vdda_uv: u64) -> u32 {
    if sample_max_count == 0 {
        return 0;
    }
    ((sample as u64 * vdda_uv) / (sample_max_count as u64 * 1000)) as u32
}

#[inline]
#[allow(dead_code)]
const fn raw_to_uv(sample: u32, max_count: u32, vdda_uv: u64) -> i64 {
    if max_count == 0 {
        return 0;
    }
    ((sample as u64 * vdda_uv) / max_count as u64) as i64
}

#[inline]
#[allow(dead_code, clippy::too_many_arguments)]
const fn two_point_temp_mc(
    ts_data: u32,
    ts_max_count: u32,
    vdda_uv: u64,
    ts_cal1: u16,
    ts_cal2: u16,
    cal_mv: u32,
    cal_bits: u8,
    cal1_temp_c: i32,
    cal2_temp_c: i32,
) -> i32 {
    let cal_max = bit_max_count(cal_bits);
    let cal_uv = cal_mv as u64 * 1000;
    let v_sense_uv = raw_to_uv(ts_data, ts_max_count, vdda_uv);
    let v_cal1_uv = raw_to_uv(ts_cal1 as u32, cal_max, cal_uv);
    let v_cal2_uv = raw_to_uv(ts_cal2 as u32, cal_max, cal_uv);
    let delta_v = v_cal2_uv - v_cal1_uv;
    if delta_v == 0 {
        return cal1_temp_c * 1000;
    }
    let delta_temp_mc = (cal2_temp_c - cal1_temp_c) as i64 * 1000;
    (cal1_temp_c as i64 * 1000 + (delta_temp_mc * (v_sense_uv - v_cal1_uv)) / delta_v) as i32
}

#[inline]
#[allow(dead_code)]
const fn linear_slope_temp_mc(
    ts_data: u32,
    ts_max_count: u32,
    vdda_uv: u64,
    ref_uv: i64,
    ref_temp_c: i32,
    avg_slope_uv_per_c: i32,
) -> i32 {
    if avg_slope_uv_per_c == 0 {
        return ref_temp_c * 1000;
    }
    let v_sense_uv = raw_to_uv(ts_data, ts_max_count, vdda_uv);
    (ref_temp_c as i64 * 1000 + ((v_sense_uv - ref_uv) * 1000) / avg_slope_uv_per_c as i64) as i32
}

/// Factory and datasheet ADC calibration parameters for the current chip.
///
/// Read once via [`Calibration::read`] and reuse to convert raw ADC counts into calibrated VDDA
/// voltages, channel millivolts, and millidegrees Celsius.
#[cfg(any(adc_vrefint_cal, adc_temp_cal, adc_temp_nominal))]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Calibration {
    /// Temperature sensor calibration value at [`TS_CAL1_TEMP_C`].
    #[cfg(adc_temp_cal)]
    pub ts_cal1: u16,
    /// Temperature sensor calibration value at [`TS_CAL2_TEMP_C`].
    #[cfg(adc_temp_two_point)]
    pub ts_cal2: u16,
    /// Internal voltage reference calibration value measured at [`VREF_CALIB_MV`].
    #[cfg(adc_vrefint_cal)]
    pub vrefint_cal: u16,
}

#[cfg(any(adc_vrefint_cal, adc_temp_cal, adc_temp_nominal))]
impl Calibration {
    /// Read factory calibration values from flash/signature registers.
    pub fn read() -> Self {
        Self {
            #[cfg(adc_temp_cal)]
            ts_cal1: read_ts_cal1(),
            #[cfg(adc_temp_two_point)]
            ts_cal2: read_ts_cal2(),
            #[cfg(adc_vrefint_cal)]
            vrefint_cal: read_vrefint_cal(),
        }
    }

    #[cfg(adc_vrefint_cal)]
    #[inline]
    fn vdda_uv_for_count(&self, vrefint_data: u32, adc_max_count: u32) -> u64 {
        calc_vdda_uv(
            self.vrefint_cal,
            crate::_generated::ADC_VREF_CAL_MV,
            crate::_generated::ADC_VREF_CAL_BITS,
            vrefint_data,
            adc_max_count,
            VREF_DEFAULT_MV,
        )
    }

    /// Calculate actual VDDA voltage in millivolts from a `VrefInt` sample taken at `resolution`.
    ///
    /// Returns [`VREF_DEFAULT_MV`] if `vrefint_data` is 0.
    #[cfg(adc_vrefint_cal)]
    pub fn vdda_mv(&self, vrefint_data: u32, resolution: Resolution) -> u32 {
        (self.vdda_uv_for_count(vrefint_data, resolution.max_count()) / 1000) as u32
    }

    /// Calculate actual VDDA voltage in millivolts assuming `vrefint_data` matches
    /// [`VREF_CALIB_BITS`] resolution.
    ///
    /// Prefer [`Calibration::vdda_mv`] when sampling at an explicit [`Resolution`] (such as
    /// 12-bit `ADC4` on STM32U5 where factory calibration is 14-bit).
    #[cfg(adc_vrefint_cal)]
    pub fn calculate_vdda_mv(&self, vrefint_data: u32) -> u32 {
        (self.vdda_uv_for_count(vrefint_data, bit_max_count(crate::_generated::ADC_VREF_CAL_BITS)) / 1000) as u32
    }

    /// Convert a raw channel sample to millivolts using a `VrefInt` reading for VDDA compensation.
    #[cfg(adc_vrefint_cal)]
    pub fn sample_to_mv(
        &self,
        sample: u32,
        sample_resolution: Resolution,
        vrefint_data: u32,
        vrefint_resolution: Resolution,
    ) -> u32 {
        let vdda_uv = self.vdda_uv_for_count(vrefint_data, vrefint_resolution.max_count());
        sample_uv_to_mv(sample, sample_resolution.max_count(), vdda_uv)
    }

    #[cfg(any(adc_temp_cal, adc_temp_nominal))]
    fn temp_mc_from_uv(&self, ts_data: u32, ts_max_count: u32, vdda_uv: u64) -> i32 {
        #[cfg(adc_temp_two_point)]
        {
            two_point_temp_mc(
                ts_data,
                ts_max_count,
                vdda_uv,
                self.ts_cal1,
                self.ts_cal2,
                crate::_generated::ADC_TEMP_CAL_MV,
                crate::_generated::ADC_TEMP_CAL_BITS,
                crate::_generated::ADC_TEMP_CAL1_C,
                crate::_generated::ADC_TEMP_CAL2_C,
            )
        }
        #[cfg(adc_temp_one_point)]
        {
            let cal_uv = raw_to_uv(
                self.ts_cal1 as u32,
                bit_max_count(crate::_generated::ADC_TEMP_CAL_BITS),
                crate::_generated::ADC_TEMP_CAL_MV as u64 * 1000,
            );
            linear_slope_temp_mc(
                ts_data,
                ts_max_count,
                vdda_uv,
                cal_uv,
                crate::_generated::ADC_TEMP_CAL1_C,
                crate::_generated::ADC_TEMP_AVG_SLOPE_UV_PER_C,
            )
        }
        #[cfg(adc_temp_nominal)]
        {
            linear_slope_temp_mc(
                ts_data,
                ts_max_count,
                vdda_uv,
                crate::_generated::ADC_TEMP_VREF_MV as i64 * 1000,
                crate::_generated::ADC_TEMP_REF_C,
                crate::_generated::ADC_TEMP_AVG_SLOPE_UV_PER_C,
            )
        }
    }

    /// Convert a temperature sensor reading at `ts_resolution` to millidegrees Celsius, using a
    /// `VrefInt` sample at `vrefint_resolution` for VDDA compensation.
    #[cfg(all(adc_vrefint_cal, adc_temp_cal))]
    pub fn temperature_millicelsius(
        &self,
        ts_data: u32,
        ts_resolution: Resolution,
        vrefint_data: u32,
        vrefint_resolution: Resolution,
    ) -> i32 {
        let vdda_uv = self.vdda_uv_for_count(vrefint_data, vrefint_resolution.max_count());
        self.temp_mc_from_uv(ts_data, ts_resolution.max_count(), vdda_uv)
    }

    /// Convert a temperature sensor reading at `ts_resolution` to millidegrees Celsius using a
    /// known `vdda_mv`.
    #[cfg(any(adc_temp_cal, adc_temp_nominal))]
    pub fn temperature_millicelsius_with_vdda(&self, ts_data: u32, ts_resolution: Resolution, vdda_mv: u32) -> i32 {
        self.temp_mc_from_uv(ts_data, ts_resolution.max_count(), (vdda_mv as u64) * 1000)
    }

    /// Convert a temperature sensor reading to millidegrees Celsius assuming both `ts_data` and
    /// `vrefint_data` match factory calibration bit widths.
    ///
    /// Prefer [`Calibration::temperature_millicelsius`] when sampling at an explicit [`Resolution`].
    #[cfg(all(adc_vrefint_cal, adc_temp_cal))]
    pub fn convert_to_millicelsius(&self, ts_data: u32, vrefint_data: u32) -> i32 {
        let vdda_uv = self.vdda_uv_for_count(vrefint_data, bit_max_count(crate::_generated::ADC_VREF_CAL_BITS));
        self.temp_mc_from_uv(ts_data, bit_max_count(crate::_generated::ADC_TEMP_CAL_BITS), vdda_uv)
    }
}

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    /// Convert a raw ADC sample taken at `self.resolution()` to millivolts using a known `vdda_mv`.
    pub fn raw_to_mv(&self, raw: u32, vdda_mv: u32) -> u32 {
        raw_to_mv(raw, self.resolution(), vdda_mv)
    }

    /// Sample `VrefInt` once (blocking) and return actual VDDA in millivolts.
    #[cfg(adc_vrefint_cal)]
    pub fn blocking_read_vdda_mv(&mut self, vrefint: &mut VrefInt, sample_time: SampleTimeOf<T>) -> u32
    where
        T: HasInternalChannel<VrefInt>,
    {
        let raw = self.blocking_read(vrefint, sample_time);
        Calibration::read().vdda_mv(raw as u32, self.resolution())
    }

    /// Sample `channel` and `VrefInt` (blocking) and return the calibrated voltage in millivolts.
    #[cfg(adc_vrefint_cal)]
    pub fn blocking_read_calibrated_mv<'a>(
        &mut self,
        channel: impl BorrowedChannel<'a, T>,
        vrefint: &mut VrefInt,
        sample_time: SampleTimeOf<T>,
    ) -> u32
    where
        T: HasInternalChannel<VrefInt>,
    {
        let res = self.resolution();
        let sample = self.blocking_read(channel, sample_time);
        let vref = self.blocking_read(vrefint, sample_time);
        Calibration::read().sample_to_mv(sample as u32, res, vref as u32, res)
    }

    /// Sample `channel` (blocking) and convert to millivolts using a known `vdda_mv`.
    pub fn blocking_read_mv_with_vdda<'a>(
        &mut self,
        channel: impl BorrowedChannel<'a, T>,
        sample_time: SampleTimeOf<T>,
        vdda_mv: u32,
    ) -> u32 {
        let sample = self.blocking_read(channel, sample_time);
        self.raw_to_mv(sample as u32, vdda_mv)
    }

    /// Sample the internal temperature sensor and `VrefInt` (blocking) and return temperature in
    /// millidegrees Celsius.
    ///
    /// Use a `sample_time` meeting the datasheet minimums for both internal channels (for example,
    /// via [`Adc::sample_time_for_us`]).
    #[cfg(all(adc_vrefint_cal, adc_temp_cal))]
    pub fn blocking_read_temperature_mc(
        &mut self,
        temp: &mut Temperature,
        vrefint: &mut VrefInt,
        sample_time: SampleTimeOf<T>,
    ) -> i32
    where
        T: HasInternalChannel<Temperature> + HasInternalChannel<VrefInt>,
    {
        let res = self.resolution();
        let ts = self.blocking_read(temp, sample_time);
        let vref = self.blocking_read(vrefint, sample_time);
        Calibration::read().temperature_millicelsius(ts as u32, res, vref as u32, res)
    }

    /// Sample the internal temperature sensor (blocking) and convert to millidegrees Celsius
    /// using a known `vdda_mv`.
    #[cfg(any(adc_temp_cal, adc_temp_nominal))]
    pub fn blocking_read_temperature_mc_with_vdda(
        &mut self,
        temp: &mut Temperature,
        sample_time: SampleTimeOf<T>,
        vdda_mv: u32,
    ) -> i32
    where
        T: HasInternalChannel<Temperature>,
    {
        let res = self.resolution();
        let ts = self.blocking_read(temp, sample_time);
        Calibration::read().temperature_millicelsius_with_vdda(ts as u32, res, vdda_mv)
    }
}

impl<'d, T: Instance> Adc<'d, T, Async> {
    /// Sample `VrefInt` once and return actual VDDA in millivolts.
    #[cfg(adc_vrefint_cal)]
    pub async fn read_vdda_mv(&mut self, vrefint: &mut VrefInt, sample_time: SampleTimeOf<T>) -> u32
    where
        T: HasInternalChannel<VrefInt>,
    {
        let raw = self.read(vrefint, sample_time).await;
        Calibration::read().vdda_mv(raw as u32, self.resolution())
    }

    /// Sample `channel` and `VrefInt` and return the calibrated voltage in millivolts.
    #[cfg(adc_vrefint_cal)]
    pub async fn read_calibrated_mv<'a>(
        &mut self,
        channel: impl BorrowedChannel<'a, T>,
        vrefint: &mut VrefInt,
        sample_time: SampleTimeOf<T>,
    ) -> u32
    where
        T: HasInternalChannel<VrefInt>,
    {
        let res = self.resolution();
        let sample = self.read(channel, sample_time).await;
        let vref = self.read(vrefint, sample_time).await;
        Calibration::read().sample_to_mv(sample as u32, res, vref as u32, res)
    }

    /// Sample `channel` and convert to millivolts using a known `vdda_mv`.
    pub async fn read_mv_with_vdda<'a>(
        &mut self,
        channel: impl BorrowedChannel<'a, T>,
        sample_time: SampleTimeOf<T>,
        vdda_mv: u32,
    ) -> u32 {
        let sample = self.read(channel, sample_time).await;
        self.raw_to_mv(sample as u32, vdda_mv)
    }

    /// Sample the internal temperature sensor and `VrefInt` and return temperature in millidegrees
    /// Celsius.
    ///
    /// Use a `sample_time` meeting the datasheet minimums for both internal channels (for example,
    /// via [`Adc::sample_time_for_us`]).
    #[cfg(all(adc_vrefint_cal, adc_temp_cal))]
    pub async fn read_temperature_mc(
        &mut self,
        temp: &mut Temperature,
        vrefint: &mut VrefInt,
        sample_time: SampleTimeOf<T>,
    ) -> i32
    where
        T: HasInternalChannel<Temperature> + HasInternalChannel<VrefInt>,
    {
        let res = self.resolution();
        let ts = self.read(temp, sample_time).await;
        let vref = self.read(vrefint, sample_time).await;
        Calibration::read().temperature_millicelsius(ts as u32, res, vref as u32, res)
    }

    /// Sample the internal temperature sensor and convert to millidegrees Celsius using a known
    /// `vdda_mv`.
    #[cfg(any(adc_temp_cal, adc_temp_nominal))]
    pub async fn read_temperature_mc_with_vdda(
        &mut self,
        temp: &mut Temperature,
        sample_time: SampleTimeOf<T>,
        vdda_mv: u32,
    ) -> i32
    where
        T: HasInternalChannel<Temperature>,
    {
        let res = self.resolution();
        let ts = self.read(temp, sample_time).await;
        Calibration::read().temperature_millicelsius_with_vdda(ts as u32, res, vdda_mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdda_scaling() {
        // 12-bit factory cal at 3.0 V, reading matches cal -> 3000 mV
        assert_eq!(calc_vdda_uv(1656, 3000, 12, 1656, 4095, 3300) / 1000, 3000);
        // Zero reading falls back cleanly to default VDDA
        assert_eq!(calc_vdda_uv(1656, 3000, 12, 0, 4095, 3300) / 1000, 3300);
        // U5: 14-bit factory cal read with 12-bit ADC4 (~1/4 of cal code) -> ~3000 mV
        let vdda_uv = calc_vdda_uv(6624, 3000, 14, 1656, 4095, 3300);
        let mv = (vdda_uv / 1000) as u32;
        assert!((2998..=3002).contains(&mv), "got {mv}");
        // H7: 16-bit factory cal read with 12-bit ADC3 -> ~3300 mV
        let vdda_uv = calc_vdda_uv(1656 * 16, 3300, 16, 1656, 4095, 3300);
        let mv = (vdda_uv / 1000) as u32;
        assert!((3298..=3302).contains(&mv), "got {mv}");
    }

    #[test]
    fn test_sample_to_mv() {
        let vdda_uv = calc_vdda_uv(1656, 3000, 12, 1656, 4095, 3300);
        assert_eq!(sample_uv_to_mv(4095, 4095, vdda_uv), 3000);
        assert_eq!(sample_uv_to_mv(2047, 4095, vdda_uv), 1499);
        assert_eq!(raw_count_to_mv(2048, 4095, 3300), 1650);
    }

    #[test]
    fn test_two_point_temp_variants() {
        // 30 C..130 C (e.g. G0/G4/L4/U0/WBA), at 3.0 V
        let vdda_3v = 3_000_000;
        assert_eq!(
            two_point_temp_mc(1000, 4095, vdda_3v, 1000, 1300, 3000, 12, 30, 130),
            30_000
        );
        assert_eq!(
            two_point_temp_mc(1300, 4095, vdda_3v, 1000, 1300, 3000, 12, 30, 130),
            130_000
        );
        assert_eq!(
            two_point_temp_mc(1150, 4095, vdda_3v, 1000, 1300, 3000, 12, 30, 130),
            80_000
        );

        // Sub-zero (-10 C): ts_data below cal1 by 0.4 * (cal2 - cal1)
        assert_eq!(
            two_point_temp_mc(880, 4095, vdda_3v, 1000, 1300, 3000, 12, 30, 130),
            -10_000
        );

        // 30 C..140 C (G5), at 3.3 V
        let vdda_3v3 = 3_300_000;
        assert_eq!(
            two_point_temp_mc(1400, 4095, vdda_3v3, 1000, 1400, 3300, 12, 30, 140),
            140_000
        );

        // Negative slope (ts_cal2 < ts_cal1, e.g. F0/F3/L1), 30 C..110 C
        assert_eq!(
            two_point_temp_mc(900, 4095, vdda_3v3, 1000, 800, 3300, 12, 30, 110),
            70_000
        );

        // U5: 14-bit cal points sampled at 12-bit ADC4 resolution with 3.0 V VDDA.
        // 1150 / 4095 * 16383 ≈ 4601.1 -> ~80.070 C.
        let temp_u5 = two_point_temp_mc(1150, 4095, vdda_3v, 4000, 5200, 3000, 14, 30, 130);
        assert!((80_000..=80_100).contains(&temp_u5), "got {temp_u5}");

        // Degenerate cal1 == cal2 guard
        assert_eq!(
            two_point_temp_mc(1200, 4095, vdda_3v, 1000, 1000, 3000, 12, 30, 130),
            30_000
        );
    }

    #[test]
    fn test_one_point_and_nominal_temp() {
        // C0: one-point, 3.0 V, 30 C, +2530 uV/C
        let v_cal1_c0 = raw_to_uv(1000, 4095, 3_000_000);
        // +25.3 mV above V_CAL1 -> +10 C -> 40 C
        let t_c0 = linear_slope_temp_mc(1000, 4095, 3_000_000, v_cal1_c0 - 25_300, 30, 2530);
        assert_eq!(t_c0, 40_000);

        // F1: nominal, V25 = 1430 mV, -4300 uV/C
        // V_SENSE at 45 C = 1430 mV - 20 * 4.3 mV = 1344 mV
        let sample_f1 = (1344 * 4095) / 3300;
        let t_f1 = linear_slope_temp_mc(sample_f1, 4095, 3_300_000, 1_430_000, 25, -4300);
        assert!((44_800..=45_200).contains(&t_f1), "got {t_f1}");

        // F2: nominal, V25 = 760 mV, +2500 uV/C
        // V_SENSE at 45 C = 760 mV + 20 * 2.5 mV = 810 mV
        let sample_f2 = (810 * 4095) / 3300;
        let t_f2 = linear_slope_temp_mc(sample_f2, 4095, 3_300_000, 760_000, 25, 2500);
        assert!((44_800..=45_200).contains(&t_f2), "got {t_f2}");
    }
}
