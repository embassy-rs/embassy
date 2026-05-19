use core::marker::PhantomData;

#[cfg(stm32u5)]
use pac::adc::vals::{Adc4Dmacfg as Dmacfg, Adc4Exten as Exten, Adc4OversamplingRatio as OversamplingRatio};
#[cfg(stm32wba)]
use pac::adc::vals::{Dmacfg, Exten, OversamplingRatio, Ovss, Smpsel};

use crate::adc::{AdcRegs, ConversionMode, Instance};
#[cfg(stm32u5)]
pub use crate::pac::adc::regs::Adc4Chselrmod0 as Chselr;
#[cfg(stm32wba)]
pub use crate::pac::adc::regs::Chselr;
#[cfg(stm32u5)]
pub use crate::pac::adc::vals::{Adc4Presc as Presc, Adc4Res as Resolution, Adc4SampleTime as SampleTime};
#[cfg(stm32wba)]
pub use crate::pac::adc::vals::{Extsel, Presc, Res as Resolution, SampleTime};
use crate::time::Hertz;
use crate::wait::block_for_us;
use crate::{Peri, interrupt, pac, rcc};

mod watchdog_adc4;
pub use watchdog_adc4::{AnalogWatchdog, WatchdogChannels, WatchdogIndex};

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);

/// Interrupt handler.
pub struct InterruptHandler<T: Instance<Regs = crate::pac::adc::Adc4>> {
    _phantom: PhantomData<T>,
}

impl<T: Instance<Regs = crate::pac::adc::Adc4>> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let isr = T::regs().isr().read();
        let ier = T::regs().ier().read();

        if ier.eocie() && isr.eoc() {
            T::regs().ier().modify(|w| w.set_eocie(false));
        } else if ier.eosie() && isr.eos() {
            T::regs().ier().modify(|w| w.set_eosie(false));
        } else if (0..3).any(|i| ier.awdie(i) && isr.awd(i)) {
            // Disable AWDIE + clear ISR flag to deassert the interrupt line.
            T::regs().ier().modify(|w| {
                for i in 0..3 {
                    if ier.awdie(i) && isr.awd(i) {
                        w.set_awdie(i, false);
                    }
                }
            });
            T::regs().isr().write(|w| {
                for i in 0..3 {
                    if isr.awd(i) {
                        w.set_awd(i, true);
                    }
                }
            });
            // Read-back flushes the write buffer (Cortex-M pattern).
            let _ = T::regs().isr().read();
            // Signal the driver via atomic flags (ISR flag is now cleared).
            for i in 0..3 {
                if ier.awdie(i) && isr.awd(i) {
                    T::state().awd_triggered[i].store(true, core::sync::atomic::Ordering::Release);
                }
            }
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

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

impl super::ConverterFor<super::VrefInt> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 0;
}

impl super::ConverterFor<super::Temperature> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 13;
}

impl super::ConverterFor<super::Vcore> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 12;
}

impl super::ConverterFor<super::Vbat> for crate::peripherals::ADC4 {
    const CHANNEL: u8 = 14;
}

impl super::ConverterFor<super::Dac> for crate::peripherals::ADC4 {
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
        Resolution::Bits12 => (1 << 12) - 1,
        Resolution::Bits10 => (1 << 10) - 1,
        Resolution::Bits8 => (1 << 8) - 1,
        Resolution::Bits6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}

/// Number of bits to left-shift a right-aligned N-bit threshold into the 12-bit AWD register.
///
/// The AWD comparison is always performed on left-aligned 12-bit raw data (RM Table 158).
/// DR returns right-aligned N-bit values; this shift converts them to 12-bit threshold space.
pub const fn resolution_to_awd_left_shift(res: Resolution) -> u32 {
    match res {
        Resolution::Bits12 => 0,
        Resolution::Bits10 => 2,
        Resolution::Bits8 => 4,
        Resolution::Bits6 => 6,
        #[allow(unreachable_patterns)]
        _ => 0,
    }
}

fn from_ker_ck(frequency: Hertz) -> Presc {
    let raw_prescaler = rcc::raw_prescaler(frequency.0, MAX_ADC_CLK_FREQ.0);
    match raw_prescaler {
        0 => Presc::Div1,
        1 => Presc::Div2,
        2..=3 => Presc::Div4,
        4..=5 => Presc::Div6,
        6..=7 => Presc::Div8,
        8..=9 => Presc::Div10,
        10..=11 => Presc::Div12,
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

    fn stop(&self, _disable: bool) {
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
        // Clear overrun and conversion flags.
        // ISR is W1C (write-1-to-clear): use write(), not modify(), to avoid
        // accidentally clearing other set flags (e.g. ADRDY) via read-modify-write.
        self.isr().write(|reg| {
            reg.set_ovr(true);
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        self.cfgr1().modify(|reg| {
            reg.set_dmaen(!matches!(conversion_mode, ConversionMode::NoDma));
            reg.set_dmacfg(Dmacfg::Circular);
            reg.set_discen(false);
            reg.set_cont(matches!(conversion_mode, ConversionMode::Repeated(None)));
            reg.set_chselrmod(false);

            #[cfg(stm32wba)]
            if let ConversionMode::Repeated(Some((trigger, _edge))) = conversion_mode {
                reg.set_extsel(Extsel::from(trigger));
            }
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut prev_channel: i16 = -1;
        let mut chselr = Chselr::default();
        let mut smpr = self.smpr().read();

        #[cfg(stm32wba)]
        let mut first_sample_time: Option<SampleTime> = None;

        for (_i, ((channel, _), sample_time)) in sequence.enumerate() {
            // For STM32WBA: SMPR only has 2 sample time slots (SMP1, SMP2).
            // We use SMP1 for all channels with the first channel's sample time.
            // For STM32U5: Each channel can have its own sample time.
            #[cfg(stm32u5)]
            smpr.set_smp(_i, sample_time);

            #[cfg(stm32wba)]
            {
                // Set SMP1 (index 0) with the first channel's sample time, use it for all channels
                if first_sample_time.is_none() {
                    first_sample_time = Some(sample_time);
                    smpr.set_smp(0, sample_time); // Index 0 = SMP1
                }
                // Set SMPSEL for this channel to use SMP1
                smpr.set_smpsel(channel as usize, Smpsel::Smp1);
            }

            let channel_num = channel;
            if channel_num as i16 <= prev_channel {
                break;
            };
            prev_channel = channel_num as i16;

            #[cfg(stm32wba)]
            chselr.set_chsel0(channel as usize, true);

            #[cfg(stm32u5)]
            chselr.set_chsel(channel as usize, true);
        }

        self.smpr().write_value(smpr);
        #[cfg(stm32wba)]
        self.chselr().write_value(chselr);
        #[cfg(stm32u5)]
        self.chselrmod0().write_value(chselr);
    }

    fn wait_done(&self) -> bool {
        self.isr().read().eos()
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

        block_for_us(1);

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            w.set_cont(false);
            w.set_discen(false);
            w.set_exten(Exten::Disabled);
            w.set_chselrmod(false);
        });

        // only use one channel at the moment
        T::regs().smpr().modify(|w| {
            #[cfg(stm32u5)]
            for i in 0..24 {
                w.set_smpsel(i, false);
            }
            #[cfg(stm32wba)]
            for i in 0..14 {
                w.set_smpsel(i, Smpsel::Smp1);
            }
        });

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint_adc4(&mut self) -> super::VrefInt {
        T::regs().ccr().modify(|w| {
            w.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature_adc4(&mut self) -> super::Temperature {
        T::regs().ccr().modify(|w| {
            w.set_vsensesel(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    #[cfg(stm32u5)]
    pub fn enable_vbat_adc4(&mut self) -> super::Vbat {
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
    pub fn enable_dac_channel_adc4(&mut self, dac: DacChannel) -> super::Dac {
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
            Averaging::Disabled => (false, OversamplingRatio::Oversample2x, 0),
            Averaging::Samples2 => (true, OversamplingRatio::Oversample2x, 1),
            Averaging::Samples4 => (true, OversamplingRatio::Oversample4x, 2),
            Averaging::Samples8 => (true, OversamplingRatio::Oversample8x, 3),
            Averaging::Samples16 => (true, OversamplingRatio::Oversample16x, 4),
            Averaging::Samples32 => (true, OversamplingRatio::Oversample32x, 5),
            Averaging::Samples64 => (true, OversamplingRatio::Oversample64x, 6),
            Averaging::Samples128 => (true, OversamplingRatio::Oversample128x, 7),
            Averaging::Samples256 => (true, OversamplingRatio::Oversample256x, 8),
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
            Averaging::Disabled => (false, OversamplingRatio::Oversample2x, Ovss::Shift0),
            Averaging::Samples2 => (true, OversamplingRatio::Oversample2x, Ovss::Shift1),
            Averaging::Samples4 => (true, OversamplingRatio::Oversample4x, Ovss::Shift2),
            Averaging::Samples8 => (true, OversamplingRatio::Oversample8x, Ovss::Shift3),
            Averaging::Samples16 => (true, OversamplingRatio::Oversample16x, Ovss::Shift4),
            Averaging::Samples32 => (true, OversamplingRatio::Oversample32x, Ovss::Shift5),
            Averaging::Samples64 => (true, OversamplingRatio::Oversample64x, Ovss::Shift6),
            Averaging::Samples128 => (true, OversamplingRatio::Oversample128x, Ovss::Shift7),
            Averaging::Samples256 => (true, OversamplingRatio::Oversample256x, Ovss::Shift8),
        };

        T::regs().cfgr2().modify(|w| {
            w.set_ovsr(samples);
            w.set_ovss(right_shift);
            w.set_ovse(enable)
        })
    }

    /// Enable an analog watchdog and return a guard.
    ///
    /// `watchdog` selects which of the three hardware watchdogs to use. `channels` controls which
    /// ADC channels are monitored; see [`WatchdogChannels`] for which variants are valid for each
    /// watchdog. `low_threshold` and `high_threshold` are raw ADC counts in the **same space as
    /// `ADC_DR`** for the currently configured resolution (i.e. `[0, 2^N − 1]` for N-bit
    /// resolution). The watchdog fires when a sample falls **outside** `[low_threshold,
    /// high_threshold]`.
    ///
    /// ## Threshold scaling
    ///
    /// The hardware AWD comparison is always against a 12-bit register value, but the data
    /// format and threshold encoding depend on whether oversampling is active and which watchdog
    /// is selected.
    ///
    /// **Without oversampling (RM Table 158):** comparison is on left-aligned 12-bit raw data.
    /// For N-bit resolution, `DR` returns a right-aligned N-bit value while the hardware
    /// left-aligns it to 12 bits for comparison.  This method left-shifts caller thresholds by
    /// `(12 - N)` automatically so you may pass values in the same range as `DR`.
    ///
    /// **With oversampling ([`Adc::set_averaging_adc4`]):** `RES` bits are ignored; all three
    /// AWDs compare `ADC_DR[15:4]` against their threshold register.  With the matched
    /// right-shift used by [`Adc::set_averaging_adc4`], `DR` holds a 12-bit result in
    /// `DR[11:0]` and the effective comparison window is the upper 8 bits.  **Pass thresholds
    /// in the same 12-bit space as `DR`** — this method handles the register encoding
    /// difference transparently:
    ///
    /// - **AWD1** stores its comparison value in `HT1[7:0]`; this method right-shifts by 4.
    /// - **AWD2/AWD3** store the comparison value in `HT[11:4]` (lower 4 bits are hardware-
    ///   ignored); writing the raw threshold places `T>>4` in `HT[11:4]` automatically — no
    ///   explicit shift is applied.
    ///
    /// The returned [`AnalogWatchdog`] does **not** borrow the ADC, so you may use the ADC for
    /// DMA or other operations while the watchdog is active.  Call [`AnalogWatchdog::wait`] to
    /// detect threshold crossings concurrently, or [`AnalogWatchdog::monitor`] for self-contained
    /// single-pin monitoring (which temporarily borrows the ADC).
    ///
    /// Dropping the guard disables the watchdog and its interrupt.
    ///
    /// # Panics
    ///
    /// Panics if `low_threshold > high_threshold`, or if a channel selection variant is used that
    /// is not supported by the chosen watchdog (e.g., [`WatchdogChannels::All`] with AWD2/AWD3,
    /// or [`WatchdogChannels::Channels`] with AWD1).
    #[must_use]
    pub fn enable_watchdog(
        &mut self,
        watchdog: WatchdogIndex,
        channels: WatchdogChannels,
        low_threshold: u16,
        high_threshold: u16,
    ) -> AnalogWatchdog<T> {
        assert!(
            low_threshold <= high_threshold,
            "low_threshold must be <= high_threshold"
        );

        let (lt, ht) = if T::regs().cfgr2().read().ovse() {
            // Under OVS all three AWDs compare ADC_DR[15:4] against the threshold register.
            // With matched OVSS (log2 of ratio), DR holds a 12-bit result in DR[11:0], so
            // the effective comparison window is DR[11:4] — 8 bits.
            //
            // However the threshold register bit layout differs between AWD1 and AWD2/AWD3:
            //
            //   AWD1  — comparison value in HT1[7:0]; HT1[11:8] must be zero.
            //           Formula: write (T >> 4) so that HT1[7:0] holds the right value.
            //
            //   AWD2/AWD3 — 8-bit effective resolution; lower 4 threshold bits are hardware-
            //           ignored; comparison is HT[11:4] vs DR[15:4].
            //           Formula: write T as-is — HT[11:4] = T[11:4] = T>>4 naturally,
            //           which is the right comparison value without an explicit shift.
            //           Applying >>4 here would write (T>>4) into the register, making
            //           HT[11:4] = T>>8 — 16× too low, causing instant false trips.
            match watchdog {
                WatchdogIndex::Awd1 => (low_threshold >> 4, high_threshold >> 4),
                _ => (low_threshold, high_threshold),
            }
        } else {
            // Without oversampling, comparison is on left-aligned 12-bit raw data (RM Table 158).
            // DR returns N-bit right-aligned values; left-shift to 12-bit space so the lower
            // (12-N) threshold bits are kept zero as the RM requires.
            let shift = resolution_to_awd_left_shift(T::regs().cfgr1().read().res());
            (
                ((low_threshold as u32) << shift) as u16,
                ((high_threshold as u32) << shift) as u16,
            )
        };

        let index = watchdog.index();
        AnalogWatchdog::<T>::setup_awd(watchdog, channels, lt, ht);
        AnalogWatchdog::new(index)
    }
}
