use cfg_if::cfg_if;
use pac::adc::vals::Dmacfg;

use super::{
    blocking_delay_us, Adc, AdcChannel, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime, SealedAdcChannel,
};
use crate::dma::Transfer;
use crate::{pac, rcc, Peri};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

pub struct VrefInt;
impl<T: Instance> AdcChannel<T> for VrefInt {}
impl<T: Instance> SealedAdcChannel<T> for VrefInt {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 13;
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                let val = 17;
            } else if #[cfg(adc_u0)] {
                let val = 12;
            } else {
                let val = 0;
            }
        }
        val
    }
}

pub struct Temperature;
impl<T: Instance> AdcChannel<T> for Temperature {}
impl<T: Instance> SealedAdcChannel<T> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 12;
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                let val = 16;
            } else if #[cfg(adc_u0)] {
                let val = 11;
            } else {
                let val = 17;
            }
        }
        val
    }
}

pub struct Vbat;
impl<T: Instance> AdcChannel<T> for Vbat {}
impl<T: Instance> SealedAdcChannel<T> for Vbat {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 14;
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                let val = 2;
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                let val = 13;
            } else {
                let val = 18;
            }
        }
        val
    }
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

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        T::regs().cr().modify(|reg| {
            #[cfg(not(any(adc_g0, adc_u0)))]
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        // If this is false then each ADC_CHSELR bit enables an input channel.
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
        });

        blocking_delay_us(20);

        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        blocking_delay_us(1);

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
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

    /// Set the ADC sample time.
    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Get the ADC sample time.
    pub fn sample_time(&self) -> SampleTime {
        self.sample_time
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
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
    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        self.read_channel(channel)
    }

    /// Read one or multiple ADC channels using DMA.
    ///
    /// `sequence` iterator and `readings` must have the same length.
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
    /// adc.read_async(
    ///     p.DMA1_CH2,
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
            sequence.len() == readings.len(),
            "Sequence length must be equal to readings length"
        );
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );

        // Ensure no conversions are ongoing and ADC is enabled.
        Self::cancel_conversions();
        self.enable();

        // Set sequence length
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        #[cfg(any(adc_g0, adc_u0))]
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

            #[cfg(any(adc_g0, adc_u0))]
            {
                channel_mask |= 1 << channel.channel();
            }
        }

        // On G0 and U0 enabled channels are sampled from 0 to last channel.
        // It is possible to add up to 8 sequences if CHSELRMOD = 1.
        // However for supporting more than 8 channels alternative CHSELRMOD = 0 approach is used.
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().chselr().modify(|reg| {
            reg.set_chsel(channel_mask);
        });

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

    fn read_channel(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        self.enable();
        Self::configure_channel(channel, self.sample_time);

        // Select channel
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().write(|reg| reg.set_sq(0, channel.channel()));
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().chselr().write(|reg| reg.set_chsel(1 << channel.channel()));

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

    #[cfg(any(adc_g0, adc_u0))]
    pub fn set_oversampling_shift(&mut self, shift: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
    }

    #[cfg(any(adc_g0, adc_u0))]
    pub fn set_oversampling_ratio(&mut self, ratio: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
    }

    #[cfg(any(adc_g0, adc_u0))]
    pub fn oversampling_enable(&mut self, enable: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_ovse(enable));
    }

    fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
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
