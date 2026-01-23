#[cfg(stm32u5)]
use pac::adc::vals::{Adc4Dmacfg as Dmacfg, Adc4Exten as Exten, Adc4OversamplingRatio as OversamplingRatio};
#[allow(unused)]
#[cfg(stm32wba)]
use pac::adc::vals::{Chselrmod, Cont, Dmacfg, Exten, OversamplingRatio, Ovss, Smpsel};

use super::blocking_delay_us;
use crate::adc::{AdcRegs, ConversionMode, Instance};
#[cfg(stm32u5)]
pub use crate::pac::adc::regs::Adc4Chselrmod0 as Chselr;
#[cfg(stm32wba)]
pub use crate::pac::adc::regs::Chselr;
#[cfg(stm32u5)]
pub use crate::pac::adc::vals::{Adc4Presc as Presc, Adc4Res as Resolution, Adc4SampleTime as SampleTime};
#[cfg(stm32wba)]
pub use crate::pac::adc::vals::{Presc, Res as Resolution, SampleTime};
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL and TSCAL registers (3.0V).
pub const VREF_CALIB_MV: u32 = 3000;

/// Temperature at which TS_CAL1 was measured (30°C).
pub const TS_CAL1_TEMP_C: i32 = 30;
/// Temperature at which TS_CAL2 was measured (130°C).
pub const TS_CAL2_TEMP_C: i32 = 130;

/// Factory calibration values read from the DESIG peripheral.
///
/// These values are programmed during manufacturing and can be used
/// for accurate temperature and voltage measurements.
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

impl Calibration {
    /// Read factory calibration values from the DESIG and VREFINTCAL peripherals.
    ///
    /// These values are unique to each chip and were measured during manufacturing
    /// at VDDA = 3.0V.
    #[cfg(stm32wba)]
    pub fn read() -> Self {
        Self {
            ts_cal1: pac::DESIG.tscal1r().read().ts_cal1(),
            ts_cal2: pac::DESIG.tscal2r().read().ts_cal2(),
            vrefint_cal: pac::VREFINTCAL.data().read().vrefint_cal(),
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
    ///
    /// # Example
    /// ```ignore
    /// let cal = Calibration::read();
    /// let temp_mc = cal.convert_to_millicelsius(temp_adc_reading, vrefint_adc_reading);
    /// let temp_c = temp_mc / 1000;
    /// let temp_frac = (temp_mc % 1000).unsigned_abs();
    /// info!("Temperature: {}.{:03} C", temp_c, temp_frac);
    /// ```
    pub fn convert_to_millicelsius(&self, ts_data: u32, vrefint_data: u32) -> i32 {
        // Compensate TS_DATA for actual VDDA vs calibration VDDA (3.0V)
        // TS_DATA_compensated = TS_DATA * VREFINT_CAL / VREFINT_DATA
        let ts_data_comp = if vrefint_data > 0 {
            (ts_data * self.vrefint_cal as u32) / vrefint_data
        } else {
            ts_data
        };

        // Use i32 for signed arithmetic (temperature can be negative)
        let ts_data_comp = ts_data_comp as i32;
        let ts_cal1 = self.ts_cal1 as i32;
        let ts_cal2 = self.ts_cal2 as i32;

        // Calculate temperature in millidegrees
        // Temp_mC = TS_CAL1_TEMP * 1000 + (TS_CAL2_TEMP - TS_CAL1_TEMP) * 1000 * (TS_DATA - TS_CAL1) / (TS_CAL2 - TS_CAL1)
        let delta_temp = (TS_CAL2_TEMP_C - TS_CAL1_TEMP_C) * 1000; // 100000 millidegrees
        let delta_cal = ts_cal2 - ts_cal1;

        if delta_cal == 0 {
            // Avoid division by zero - return raw estimate
            return ts_data_comp * 10;
        }

        TS_CAL1_TEMP_C * 1000 + (delta_temp * (ts_data_comp - ts_cal1)) / delta_cal
    }

    /// Calculate the actual VDDA voltage in millivolts using VREFINT.
    ///
    /// The formula is: VDDA = 3000mV × VREFINT_CAL / VREFINT_DATA
    ///
    /// # Arguments
    /// * `vrefint_data` - Raw ADC reading from the VREFINT channel
    ///
    /// # Returns
    /// Actual VDDA voltage in millivolts
    pub fn calculate_vdda_mv(&self, vrefint_data: u32) -> u32 {
        if vrefint_data > 0 {
            (VREF_CALIB_MV * self.vrefint_cal as u32) / vrefint_data
        } else {
            VREF_DEFAULT_MV
        }
    }
}

impl super::SealedSpecialConverter<super::VrefInt> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 0;
}

impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 13;
}

impl super::SealedSpecialConverter<super::Vcore> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 12;
}

impl super::SealedSpecialConverter<super::Vbat> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 14;
}

impl super::SealedSpecialConverter<super::Dac> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 21;
}

#[derive(Copy, Clone)]
pub enum DacChannel {
    OUT1,
    OUT2,
}

/// Number of samples used for averaging.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Averaging {
    Disabled,
    Samples2,
    Samples4,
    Samples8,
    Samples16,
    Samples32,
    Samples64,
    Samples128,
    Samples256,
}

pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    match res {
        Resolution::BITS12 => (1 << 12) - 1,
        Resolution::BITS10 => (1 << 10) - 1,
        Resolution::BITS8 => (1 << 8) - 1,
        Resolution::BITS6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}

fn from_ker_ck(frequency: Hertz) -> Presc {
    let raw_prescaler = rcc::raw_prescaler(frequency.0, MAX_ADC_CLK_FREQ.0);
    match raw_prescaler {
        0 => Presc::DIV1,
        1 => Presc::DIV2,
        2..=3 => Presc::DIV4,
        4..=5 => Presc::DIV6,
        6..=7 => Presc::DIV8,
        8..=9 => Presc::DIV10,
        10..=11 => Presc::DIV12,
        _ => unimplemented!(),
    }
}

impl AdcRegs for crate::pac::adc::Adc4 {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc4::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        if !self.cr().read().aden() || !self.isr().read().adrdy() {
            self.isr().write(|w| w.set_adrdy(true));
            self.cr().modify(|w| w.set_aden(true));
            while !self.isr().read().adrdy() {}
        }
    }

    fn start(&self) {
        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self) {
        let cr = self.cr().read();
        if cr.adstart() {
            self.cr().modify(|w| w.set_adstp(true));
            while self.cr().read().adstart() {}
        }

        if cr.aden() || cr.adstart() {
            self.cr().modify(|w| w.set_addis(true));
            while self.cr().read().aden() {}
        }

        // Reset configuration.
        self.cfgr1().modify(|reg| {
            reg.set_dmaen(false);
        });
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Clear overrun and conversion flags
        self.isr().modify(|reg| {
            reg.set_ovr(true);
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        match conversion_mode {
            ConversionMode::Singular => {
                self.cfgr1().modify(|reg| {
                    reg.set_dmaen(true);
                    reg.set_dmacfg(Dmacfg::ONE_SHOT);
                    reg.set_discen(false);
                    #[cfg(stm32u5)]
                    {
                        reg.set_cont(false);
                        reg.set_chselrmod(false);
                    }
                    #[cfg(stm32wba)]
                    {
                        reg.set_cont(Cont::SINGLE);
                        reg.set_chselrmod(Chselrmod::ENABLE_INPUT);
                    }
                });
            }
            #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0))]
            ConversionMode::Repeated(_) => unreachable!(),
            #[cfg(stm32wba)]
            ConversionMode::Repeated(_mode) => {
                // Configure for circular DMA with continuous conversion
                self.cfgr1().modify(|reg| {
                    reg.set_dmaen(true);
                    reg.set_dmacfg(Dmacfg::CIRCULAR); // Enable circular DMA mode
                    reg.set_cont(Cont::CONTINUOUS); // Enable continuous conversion
                    reg.set_discen(false); // Disable discontinuous mode
                    reg.set_chselrmod(Chselrmod::ENABLE_INPUT);
                });
            }
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut prev_channel: i16 = -1;
        #[cfg(stm32wba)]
        self.chselr().write_value(Chselr(0_u32));
        #[cfg(stm32u5)]
        self.chselrmod0().write_value(Chselr(0_u32));

        #[cfg(stm32wba)]
        let mut first_sample_time: Option<SampleTime> = None;

        for (_i, ((channel, _), sample_time)) in sequence.enumerate() {
            // For STM32WBA: SMPR only has 2 sample time slots (SMP1, SMP2).
            // We use SMP1 for all channels with the first channel's sample time.
            // For STM32U5: Each channel can have its own sample time.
            #[cfg(stm32u5)]
            self.smpr().modify(|w| {
                w.set_smp(_i, sample_time);
            });

            #[cfg(stm32wba)]
            {
                // Set SMP1 (index 0) with the first channel's sample time, use it for all channels
                if first_sample_time.is_none() {
                    first_sample_time = Some(sample_time);
                    self.smpr().modify(|w| {
                        w.set_smp(0, sample_time); // Index 0 = SMP1
                    });
                }
                // Set SMPSEL for this channel to use SMP1
                self.smpr().modify(|w| {
                    w.set_smpsel(channel as usize, Smpsel::SMP1);
                });
            }

            let channel_num = channel;
            if channel_num as i16 <= prev_channel {
                return;
            };
            prev_channel = channel_num as i16;

            #[cfg(stm32wba)]
            self.chselr().modify(|w| {
                w.set_chsel0(channel as usize, true);
            });
            #[cfg(stm32u5)]
            self.chselrmod0().modify(|w| {
                w.set_chsel(channel as usize, true);
            });
        }
    }

    fn convert(&self) {
        // Reset interrupts
        self.isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });

        while !self.isr().read().eos() {
            // spin
        }
    }
}

impl<'d, T: Instance<Regs = crate::pac::adc::Adc4>> super::Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new_adc4(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        let prescaler = from_ker_ck(T::frequency());

        T::regs().ccr().modify(|w| w.set_presc(prescaler));

        let frequency = T::frequency() / prescaler;
        info!("ADC4 frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for ADC4 is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        T::regs().isr().modify(|w| {
            w.set_ldordy(true);
        });
        T::regs().cr().modify(|w| {
            w.set_advregen(true);
        });
        while !T::regs().isr().read().ldordy() {}

        T::regs().isr().modify(|w| {
            w.set_ldordy(true);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));
        while T::regs().cr().read().adcal() {}
        T::regs().isr().modify(|w| w.set_eocal(true));

        blocking_delay_us(1);

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            #[cfg(stm32u5)]
            w.set_cont(false);
            #[cfg(stm32wba)]
            w.set_cont(Cont::SINGLE);
            w.set_discen(false);
            w.set_exten(Exten::DISABLED);
            #[cfg(stm32u5)]
            w.set_chselrmod(false);
            #[cfg(stm32wba)]
            w.set_chselrmod(Chselrmod::ENABLE_INPUT);
        });

        // only use one channel at the moment
        T::regs().smpr().modify(|w| {
            #[cfg(stm32u5)]
            for i in 0..24 {
                w.set_smpsel(i, false);
            }
            #[cfg(stm32wba)]
            for i in 0..14 {
                w.set_smpsel(i, Smpsel::SMP1);
            }
        });

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint_adc4(&self) -> super::VrefInt {
        T::regs().ccr().modify(|w| {
            w.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature_adc4(&self) -> super::Temperature {
        T::regs().ccr().modify(|w| {
            w.set_vsensesel(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    #[cfg(stm32u5)]
    pub fn enable_vbat_adc4(&self) -> super::Vbat {
        T::regs().ccr().modify(|w| {
            w.set_vbaten(true);
        });

        super::Vbat {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vcore_adc4(&self) -> super::Vcore {
        super::Vcore {}
    }

    /// Enable reading the vbat internal channel.
    #[cfg(stm32u5)]
    pub fn enable_dac_channel_adc4(&self, dac: DacChannel) -> super::Dac {
        let mux;
        match dac {
            DacChannel::OUT1 => mux = false,
            DacChannel::OUT2 => mux = true,
        }
        T::regs().or().modify(|w| w.set_chn21sel(mux));
        super::Dac {}
    }

    /// Set the ADC resolution.
    pub fn set_resolution_adc4(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|w| w.set_res(resolution.into()));
    }

    /// Set hardware averaging.
    #[cfg(stm32u5)]
    pub fn set_averaging_adc4(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, OversamplingRatio::OVERSAMPLE2X, 0),
            Averaging::Samples2 => (true, OversamplingRatio::OVERSAMPLE2X, 1),
            Averaging::Samples4 => (true, OversamplingRatio::OVERSAMPLE4X, 2),
            Averaging::Samples8 => (true, OversamplingRatio::OVERSAMPLE8X, 3),
            Averaging::Samples16 => (true, OversamplingRatio::OVERSAMPLE16X, 4),
            Averaging::Samples32 => (true, OversamplingRatio::OVERSAMPLE32X, 5),
            Averaging::Samples64 => (true, OversamplingRatio::OVERSAMPLE64X, 6),
            Averaging::Samples128 => (true, OversamplingRatio::OVERSAMPLE128X, 7),
            Averaging::Samples256 => (true, OversamplingRatio::OVERSAMPLE256X, 8),
        };

        T::regs().cfgr2().modify(|w| {
            w.set_ovsr(samples);
            w.set_ovss(right_shift);
            w.set_ovse(enable)
        })
    }
    #[cfg(stm32wba)]
    pub fn set_averaging_adc4(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, OversamplingRatio::OVERSAMPLE2X, Ovss::SHIFT0),
            Averaging::Samples2 => (true, OversamplingRatio::OVERSAMPLE2X, Ovss::SHIFT1),
            Averaging::Samples4 => (true, OversamplingRatio::OVERSAMPLE4X, Ovss::SHIFT2),
            Averaging::Samples8 => (true, OversamplingRatio::OVERSAMPLE8X, Ovss::SHIFT3),
            Averaging::Samples16 => (true, OversamplingRatio::OVERSAMPLE16X, Ovss::SHIFT4),
            Averaging::Samples32 => (true, OversamplingRatio::OVERSAMPLE32X, Ovss::SHIFT5),
            Averaging::Samples64 => (true, OversamplingRatio::OVERSAMPLE64X, Ovss::SHIFT6),
            Averaging::Samples128 => (true, OversamplingRatio::OVERSAMPLE128X, Ovss::SHIFT7),
            Averaging::Samples256 => (true, OversamplingRatio::OVERSAMPLE256X, Ovss::SHIFT8),
        };

        T::regs().cfgr2().modify(|w| {
            w.set_ovsr(samples);
            w.set_ovss(right_shift);
            w.set_ovse(enable)
        })
    }
}
