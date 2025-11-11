use cfg_if::cfg_if;
#[cfg(adc_g0)]
use heapless::Vec;
use pac::adc::vals::Dmacfg;
#[cfg(adc_g0)]
use pac::adc::vals::{Ckmode, Smpsel};
#[cfg(adc_v3)]
use pac::adc::vals::{OversamplingRatio, OversamplingShift, Rovsm, Trovs};
#[cfg(adc_g0)]
pub use pac::adc::vals::{Ovsr, Ovss, Presc};

#[allow(unused_imports)]
use super::SealedAdcChannel;
use super::{
    Adc, AdcChannel, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime, Temperature, Vbat, VrefInt,
    blocking_delay_us,
};

#[cfg(any(adc_v3, adc_g0, adc_u0))]
mod ringbuffered;

#[cfg(any(adc_v3, adc_g0, adc_u0))]
use ringbuffered::RingBufferedAdc;

use crate::dma::Transfer;
use crate::{Peri, pac, rcc};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

#[cfg(adc_g0)]
/// The number of variants in Smpsel
// TODO: Use [#![feature(variant_count)]](https://github.com/rust-lang/rust/issues/73662) when stable
const SAMPLE_TIMES_CAPACITY: usize = 2;

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 17;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 0;
}

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 11;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 17;
}

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 14;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 2;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 18;
}

cfg_if! {
    if #[cfg(any(adc_h5, adc_h7rs))] {
        pub struct VddCore;
        impl<T: Instance> AdcChannel<T> for VddCore {}
        impl<T: Instance> super::SealedAdcChannel<T> for VddCore {
            fn channel(&self) -> u8 {
                6
            }
        }
    }
}

cfg_if! {
    if #[cfg(adc_u0)] {
        pub struct DacOut;
        impl<T: Instance> AdcChannel<T> for DacOut {}
        impl<T: Instance> super::SealedAdcChannel<T> for DacOut {
            fn channel(&self) -> u8 {
                19
            }
        }
    }
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

cfg_if! { if #[cfg(adc_g0)] {

/// Synchronous PCLK prescaler
pub enum CkModePclk {
    DIV1,
    DIV2,
    DIV4,
}

/// The analog clock is either the synchronous prescaled PCLK or
/// the asynchronous prescaled ADCCLK configured by the RCC mux.
/// The data sheet states the maximum analog clock frequency -
/// for STM32WL55CC it is 36 MHz.
pub enum Clock {
    Sync { div: CkModePclk },
    Async { div: Presc },
}

}}

impl<'d, T: Instance> Adc<'d, T> {
    /// Enable the voltage regulator
    fn init_regulator() {
        rcc::enable_and_reset::<T>();
        T::regs().cr().modify(|reg| {
            #[cfg(not(any(adc_g0, adc_u0)))]
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        // If this is false then each ADC_CHSELR bit enables an input channel.
        // This is the reset value, so has no effect.
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
        });

        blocking_delay_us(20);
    }

    /// Calibrate to remove conversion offset
    fn init_calibrate() {
        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        blocking_delay_us(1);
    }

    #[cfg(any(adc_v3, adc_g0, adc_u0))]
    pub(super) fn start() {
        // Start adc conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    #[cfg(any(adc_v3, adc_g0, adc_u0))]
    pub(super) fn stop() {
        // Stop adc conversion
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while T::regs().cr().read().adstart() {}
        }
    }

    #[cfg(any(adc_v3, adc_g0, adc_u0))]
    pub(super) fn teardown_dma() {
        //disable dma control
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_dmaen(false);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_dmaen(false);
        });
    }

    /// Initialize the ADC leaving any analog clock at reset value.
    /// For G0 and WL, this is the async clock without prescaler.
    pub fn new(adc: Peri<'d, T>) -> Self {
        Self::init_regulator();
        Self::init_calibrate();
        Self { adc }
    }

    #[cfg(adc_g0)]
    /// Initialize ADC with explicit clock for the analog ADC
    pub fn new_with_clock(adc: Peri<'d, T>, clock: Clock) -> Self {
        Self::init_regulator();

        #[cfg(any(stm32wl5x))]
        {
            // Reset value 0 is actually _No clock selected_ in the STM32WL5x reference manual
            let async_clock_available = pac::RCC.ccipr().read().adcsel() != pac::rcc::vals::Adcsel::_RESERVED_0;
            match clock {
                Clock::Async { div: _ } => {
                    assert!(async_clock_available);
                }
                Clock::Sync { div: _ } => {
                    if async_clock_available {
                        warn!("Not using configured ADC clock");
                    }
                }
            }
        }
        match clock {
            Clock::Async { div } => T::regs().ccr().modify(|reg| reg.set_presc(div)),
            Clock::Sync { div } => T::regs().cfgr2().modify(|reg| {
                reg.set_ckmode(match div {
                    CkModePclk::DIV1 => Ckmode::PCLK,
                    CkModePclk::DIV2 => Ckmode::PCLK_DIV2,
                    CkModePclk::DIV4 => Ckmode::PCLK_DIV4,
                })
            }),
        }

        Self::init_calibrate();

        Self { adc }
    }

    // Enable ADC only when it is not already running.
    fn enable(&mut self) {
        // Make sure bits are off
        while T::regs().cr().read().addis() {
            // spin
        }

        if !T::regs().cr().read().aden() {
            // Enable ADC
            T::regs().isr().modify(|reg| {
                reg.set_adrdy(true);
            });
            T::regs().cr().modify(|reg| {
                reg.set_aden(true);
            });

            while !T::regs().isr().read().adrdy() {
                // spin
            }
        }
    }

    pub fn enable_vrefint(&self) -> VrefInt {
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        // "Table 24. Embedded internal voltage reference" states that it takes a maximum of 12 us
        // to stabilize the internal voltage reference.
        blocking_delay_us(15);

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
                T::regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch17sel(true);
                });
            }
        }

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
                T::regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch18sel(true);
                });
            }
        }

        Vbat {}
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
    }

    pub fn set_averaging(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, 0, 0),
            Averaging::Samples2 => (true, 0, 1),
            Averaging::Samples4 => (true, 1, 2),
            Averaging::Samples8 => (true, 2, 3),
            Averaging::Samples16 => (true, 3, 4),
            Averaging::Samples32 => (true, 4, 5),
            Averaging::Samples64 => (true, 5, 6),
            Averaging::Samples128 => (true, 6, 7),
            Averaging::Samples256 => (true, 7, 8),
        };
        T::regs().cfgr2().modify(|reg| {
            #[cfg(not(any(adc_g0, adc_u0)))]
            reg.set_rovse(enable);
            #[cfg(any(adc_g0, adc_u0))]
            reg.set_ovse(enable);
            #[cfg(any(adc_h5, adc_h7rs))]
            reg.set_ovsr(samples.into());
            #[cfg(not(any(adc_h5, adc_h7rs)))]
            reg.set_ovsr(samples.into());
            reg.set_ovss(right_shift.into());
        })
    }
    /*
    /// Convert a raw sample from the `Temperature` to deg C
    pub fn to_degrees_centigrade(sample: u16) -> f32 {
        (130.0 - 30.0) / (VtempCal130::get().read() as f32 - VtempCal30::get().read() as f32)
            * (sample as f32 - VtempCal30::get().read() as f32)
            + 30.0
    }
     */

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        T::regs().isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        // Start conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        while !T::regs().isr().read().eos() {
            // spin
        }

        T::regs().dr().read().0 as u16
    }

    /// Read an ADC channel.
    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        self.read_channel(channel, sample_time)
    }

    /// Read one or multiple ADC channels using DMA.
    ///
    /// `readings` must have a length that is a multiple of the length of the
    /// `sequence` iterator.
    ///
    /// Note: The order of values in `readings` is defined by the pin ADC
    /// channel number and not the pin order in `sequence`.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new(p.ADC1);
    /// let mut adc_pin0 = p.PA0.degrade_adc();
    /// let mut adc_pin1 = p.PA1.degrade_adc();
    /// let mut measurements = [0u16; 2];
    ///
    /// adc.read(
    ///     p.DMA1_CH2.reborrow(),
    ///     [
    ///         (&mut *adc_pin0, SampleTime::CYCLES160_5),
    ///         (&mut *adc_pin1, SampleTime::CYCLES160_5),
    ///     ]
    ///     .into_iter(),
    ///     &mut measurements,
    /// )
    /// .await;
    /// defmt::info!("measurements: {}", measurements);
    /// ```
    pub async fn read(
        &mut self,
        rx_dma: Peri<'_, impl RxDma<T>>,
        sequence: impl ExactSizeIterator<Item = (&mut AnyAdcChannel<T>, SampleTime)>,
        readings: &mut [u16],
    ) {
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            readings.len() % sequence.len() == 0,
            "Readings length must be a multiple of sequence length"
        );
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );

        #[cfg(all(feature = "low-power", stm32wlex))]
        let _device_busy = crate::low_power::DeviceBusy::new_stop1();

        // Ensure no conversions are ongoing and ADC is enabled.
        Self::cancel_conversions();
        self.enable();

        // Set sequence length
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        #[cfg(adc_g0)]
        {
            let mut sample_times = Vec::<SampleTime, SAMPLE_TIMES_CAPACITY>::new();

            T::regs().chselr().write(|chselr| {
                T::regs().smpr().write(|smpr| {
                    for (channel, sample_time) in sequence {
                        chselr.set_chsel(channel.channel.into(), true);
                        if let Some(i) = sample_times.iter().position(|&t| t == sample_time) {
                            smpr.set_smpsel(channel.channel.into(), (i as u8).into());
                        } else {
                            smpr.set_sample_time(sample_times.len(), sample_time);
                            if let Err(_) = sample_times.push(sample_time) {
                                panic!(
                                    "Implementation is limited to {} unique sample times among all channels.",
                                    SAMPLE_TIMES_CAPACITY
                                );
                            }
                        }
                    }
                })
            });
        }
        #[cfg(not(adc_g0))]
        {
            #[cfg(adc_u0)]
            let mut channel_mask = 0;

            // Configure channels and ranks
            for (_i, (channel, sample_time)) in sequence.enumerate() {
                Self::configure_channel(channel, sample_time);

                // Each channel is sampled according to sequence
                #[cfg(not(any(adc_g0, adc_u0)))]
                match _i {
                    0..=3 => {
                        T::regs().sqr1().modify(|w| {
                            w.set_sq(_i, channel.channel());
                        });
                    }
                    4..=8 => {
                        T::regs().sqr2().modify(|w| {
                            w.set_sq(_i - 4, channel.channel());
                        });
                    }
                    9..=13 => {
                        T::regs().sqr3().modify(|w| {
                            w.set_sq(_i - 9, channel.channel());
                        });
                    }
                    14..=15 => {
                        T::regs().sqr4().modify(|w| {
                            w.set_sq(_i - 14, channel.channel());
                        });
                    }
                    _ => unreachable!(),
                }

                #[cfg(adc_u0)]
                {
                    channel_mask |= 1 << channel.channel();
                }
            }

            // On G0 and U0 enabled channels are sampled from 0 to last channel.
            // It is possible to add up to 8 sequences if CHSELRMOD = 1.
            // However for supporting more than 8 channels alternative CHSELRMOD = 0 approach is used.
            #[cfg(adc_u0)]
            T::regs().chselr().modify(|reg| {
                reg.set_chsel(channel_mask);
            });
        }
        // Set continuous mode with oneshot dma.
        // Clear overrun flag before starting transfer.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::ONE_SHOT);
            reg.set_dmaen(true);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::ONE_SHOT);
            reg.set_dmaen(true);
        });

        let request = rx_dma.request();
        let transfer = unsafe {
            Transfer::new_read(
                rx_dma,
                request,
                T::regs().dr().as_ptr() as *mut u16,
                readings,
                Default::default(),
            )
        };

        // Start conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        // Wait for conversion sequence to finish.
        transfer.await;

        // Ensure conversions are finished.
        Self::cancel_conversions();

        // Reset configuration.
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_cont(false);
        });
    }

    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// The `dma_buf` should be large enough to prevent DMA buffer overrun.
    /// The length of the `dma_buf` should be a multiple of the ADC channel count.
    /// For example, if 3 channels are measured, its length can be 3 * 40 = 120 measurements.
    ///
    /// `read` method is used to read out measurements from the DMA ring buffer, and its buffer should be exactly half of the `dma_buf` length.
    /// It is critical to call `read` frequently to prevent DMA buffer overrun.
    ///
    /// [`read`]: #method.read
    #[cfg(any(adc_v3, adc_g0, adc_u0))]
    pub fn into_ring_buffered<'a>(
        &mut self,
        dma: Peri<'a, impl RxDma<T>>,
        dma_buf: &'a mut [u16],
        sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<T>, SampleTime)>,
    ) -> RingBufferedAdc<'a, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );
        // reset conversions and enable the adc
        Self::cancel_conversions();
        self.enable();

        //adc side setup

        // Set sequence length
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        #[cfg(adc_g0)]
        {
            let mut sample_times = Vec::<SampleTime, SAMPLE_TIMES_CAPACITY>::new();

            T::regs().chselr().write(|chselr| {
                T::regs().smpr().write(|smpr| {
                    for (channel, sample_time) in sequence {
                        chselr.set_chsel(channel.channel.into(), true);
                        if let Some(i) = sample_times.iter().position(|&t| t == sample_time) {
                            smpr.set_smpsel(channel.channel.into(), (i as u8).into());
                        } else {
                            smpr.set_sample_time(sample_times.len(), sample_time);
                            if let Err(_) = sample_times.push(sample_time) {
                                panic!(
                                    "Implementation is limited to {} unique sample times among all channels.",
                                    SAMPLE_TIMES_CAPACITY
                                );
                            }
                        }
                    }
                })
            });
        }
        #[cfg(not(adc_g0))]
        {
            #[cfg(adc_u0)]
            let mut channel_mask = 0;

            // Configure channels and ranks
            for (_i, (mut channel, sample_time)) in sequence.enumerate() {
                Self::configure_channel(&mut channel, sample_time);

                // Each channel is sampled according to sequence
                #[cfg(not(any(adc_g0, adc_u0)))]
                match _i {
                    0..=3 => {
                        T::regs().sqr1().modify(|w| {
                            w.set_sq(_i, channel.channel());
                        });
                    }
                    4..=8 => {
                        T::regs().sqr2().modify(|w| {
                            w.set_sq(_i - 4, channel.channel());
                        });
                    }
                    9..=13 => {
                        T::regs().sqr3().modify(|w| {
                            w.set_sq(_i - 9, channel.channel());
                        });
                    }
                    14..=15 => {
                        T::regs().sqr4().modify(|w| {
                            w.set_sq(_i - 14, channel.channel());
                        });
                    }
                    _ => unreachable!(),
                }

                #[cfg(adc_u0)]
                {
                    channel_mask |= 1 << channel.channel();
                }
            }

            // On G0 and U0 enabled channels are sampled from 0 to last channel.
            // It is possible to add up to 8 sequences if CHSELRMOD = 1.
            // However for supporting more than 8 channels alternative CHSELRMOD = 0 approach is used.
            #[cfg(adc_u0)]
            T::regs().chselr().modify(|reg| {
                reg.set_chsel(channel_mask);
            });
        }
        // Set continuous mode with Circular dma.
        // Clear overrun flag before starting transfer.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::CIRCULAR);
            reg.set_dmaen(true);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::CIRCULAR);
            reg.set_dmaen(true);
        });

        RingBufferedAdc::new(dma, dma_buf)
    }

    #[cfg(not(adc_g0))]
    fn configure_channel(channel: &mut impl AdcChannel<T>, sample_time: SampleTime) {
        // RM0492, RM0481, etc.
        // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
        #[cfg(any(adc_h5, adc_h7rs))]
        if channel.channel() == 0 {
            T::regs().or().modify(|reg| reg.set_op0(true));
        }

        // Configure channel
        Self::set_channel_sample_time(channel.channel(), sample_time);
    }

    fn read_channel(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        self.enable();
        #[cfg(not(adc_g0))]
        Self::configure_channel(channel, sample_time);
        #[cfg(adc_g0)]
        T::regs().smpr().write(|reg| {
            reg.set_sample_time(0, sample_time);
            reg.set_smpsel(channel.channel().into(), Smpsel::SMP1);
        });
        // Select channel
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().write(|reg| reg.set_sq(0, channel.channel()));
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().chselr().write(|reg| {
            #[cfg(adc_g0)]
            reg.set_chsel(channel.channel().into(), true);
            #[cfg(adc_u0)]
            reg.set_chsel(1 << channel.channel());
        });

        // Some models are affected by an erratum:
        // If we perform conversions slower than 1 kHz, the first read ADC value can be
        // corrupted, so we discard it and measure again.
        //
        // STM32L471xx: Section 2.7.3
        // STM32G4: Section 2.7.3
        #[cfg(any(rcc_l4, rcc_g4))]
        let _ = self.convert();
        let val = self.convert();

        T::regs().cr().modify(|reg| reg.set_addis(true));

        // RM0492, RM0481, etc.
        // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
        #[cfg(any(adc_h5, adc_h7rs))]
        if channel.channel() == 0 {
            T::regs().or().modify(|reg| reg.set_op0(false));
        }

        val
    }

    #[cfg(adc_g0)]
    pub fn set_oversampling_shift(&mut self, shift: Ovss) {
        T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
    }
    #[cfg(adc_u0)]
    pub fn set_oversampling_shift(&mut self, shift: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
    }

    #[cfg(adc_g0)]
    pub fn set_oversampling_ratio(&mut self, ratio: Ovsr) {
        T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
    }
    #[cfg(adc_u0)]
    pub fn set_oversampling_ratio(&mut self, ratio: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
    }

    #[cfg(any(adc_g0, adc_u0))]
    pub fn oversampling_enable(&mut self, enable: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_ovse(enable));
    }

    #[cfg(adc_v3)]
    pub fn enable_regular_oversampling_mode(&mut self, mode: Rovsm, trig_mode: Trovs, enable: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_trovs(trig_mode));
        T::regs().cfgr2().modify(|reg| reg.set_rovsm(mode));
        T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
    }

    #[cfg(adc_v3)]
    pub fn set_oversampling_ratio(&mut self, ratio: OversamplingRatio) {
        T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
    }

    #[cfg(adc_v3)]
    pub fn set_oversampling_shift(&mut self, shift: OversamplingShift) {
        T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
    }

    #[cfg(not(adc_g0))]
    fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        cfg_if! {
            if #[cfg(adc_u0)] {
                // On G0 and U6 all channels use the same sampling time.
                T::regs().smpr().modify(|reg| reg.set_smp1(sample_time.into()));
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                match _ch {
                    0..=9 => T::regs().smpr1().modify(|w| w.set_smp(_ch as usize % 10, sample_time.into())),
                    _ => T::regs().smpr2().modify(|w| w.set_smp(_ch as usize % 10, sample_time.into())),
                }
            } else {
                let sample_time = sample_time.into();
                T::regs()
                    .smpr(_ch as usize / 10)
                    .modify(|reg| reg.set_smp(_ch as usize % 10, sample_time));
            }
        }
    }

    fn cancel_conversions() {
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while T::regs().cr().read().adstart() {}
        }
    }
}
