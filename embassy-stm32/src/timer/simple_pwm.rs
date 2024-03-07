//! Simple PWM driver.

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::{AnyPin, OutputType};
use crate::time::Hertz;
use crate::Peripheral;

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}
/// Channel 3 marker type.
pub enum Ch3 {}
/// Channel 4 marker type.
pub enum Ch4 {}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct PwmPin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: CaptureCompare16bitInstance> PwmPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd, output_type: OutputType) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), output_type.into());
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                PwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ch3, Ch3, Channel3Pin);
channel_impl!(new_ch4, Ch4, Channel4Pin);

/// Simple PWM driver.
pub struct SimplePwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare16bitInstance> SimplePwm<'d, T> {
    /// Create a new simple PWM driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, counting_mode: CountingMode) -> Self {
        into_ref!(tim);

        T::enable_and_reset();

        let mut this = Self { inner: tim };

        this.inner.set_counting_mode(counting_mode);
        this.set_frequency(freq);
        this.inner.start();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);

                this.inner.set_output_compare_preload(channel, true);
            });

        this
    }

    /// Enable the given channel.
    pub fn enable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self, channel: Channel) -> bool {
        self.inner.get_channel_enable_state(channel)
    }

    /// Set PWM frequency.
    ///
    /// Note: when you call this, the max duty value changes, so you will have to
    /// call `set_duty` on all channels with the duty calculated based on the new max duty.
    pub fn set_frequency(&mut self, freq: Hertz) {
        let multiplier = if self.inner.get_counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.inner.set_frequency(freq * multiplier);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    /// Get the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn get_duty(&self, channel: Channel) -> u16 {
        self.inner.get_compare_value(channel)
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_output_polarity(channel, polarity);
    }

    /// Set the output compare mode for a given channel.
    pub fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
        self.inner.set_output_compare_mode(channel, mode);
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:  
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub async fn waveform_up(
        &mut self,
        dma: impl Peripheral<P = impl super::UpDma<T>>,
        channel: Channel,
        duty: &[u16],
    ) {
        assert!(duty.iter().all(|v| *v <= self.get_max_duty()));

        into_ref!(dma);

        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        let original_duty_state = self.get_duty(channel);
        let original_enable_state = self.is_enabled(channel);
        let original_update_dma_state = self.inner.get_update_dma_state();

        if !original_update_dma_state {
            self.inner.enable_update_dma(true);
        }

        if !original_enable_state {
            self.enable(channel);
        }

        unsafe {
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};
            use crate::dma::{Transfer, TransferOptions};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr8,
                ..Default::default()
            };

            Transfer::new_write(
                &mut dma,
                req,
                duty,
                T::regs_1ch().ccr(channel.index()).as_ptr() as *mut _,
                dma_transfer_option,
            )
            .await
        };

        // restore output compare state
        if !original_enable_state {
            self.disable(channel);
        }

        self.set_duty(channel, original_duty_state);

        // Since DMA is closed before timer update event trigger DMA is turn off,
        // this can almost always trigger a DMA FIFO error.
        //
        // optional TODO:
        // clean FEIF after disable UDE
        if !original_update_dma_state {
            self.inner.enable_update_dma(false);
        }
    }
}

macro_rules! impl_waveform_chx {
    ($fn_name:ident, $dma_ch:ident, $cc_ch:ident) => {
        impl<'d, T: CaptureCompare16bitInstance> SimplePwm<'d, T> {
            /// Generate a sequence of PWM waveform
            ///
            /// Note:
            /// you will need to provide corresponding TIMx_CHy DMA channel to use this method.
            pub async fn $fn_name(&mut self, dma: impl Peripheral<P = impl super::$dma_ch<T>>, duty: &[u16]) {
                use super::vals::Ccds;

                assert!(duty.iter().all(|v| *v <= self.get_max_duty()));

                into_ref!(dma);

                #[allow(clippy::let_unit_value)] // eg. stm32f334
                let req = dma.request();

                let cc_channel = super::Channel::$cc_ch;

                let original_duty_state = self.get_duty(cc_channel);
                let original_enable_state = self.is_enabled(cc_channel);
                let original_cc_dma_on_update = self.inner.get_cc_dma_selection() == Ccds::ONUPDATE;
                let original_cc_dma_enabled = self.inner.get_cc_dma_enable_state(cc_channel);

                // redirect CC DMA request onto Update Event
                if !original_cc_dma_on_update {
                    self.inner.set_cc_dma_selection(Ccds::ONUPDATE)
                }

                if !original_cc_dma_enabled {
                    self.inner.set_cc_dma_enable_state(cc_channel, true);
                }

                if !original_enable_state {
                    self.enable(cc_channel);
                }

                unsafe {
                    #[cfg(not(any(bdma, gpdma)))]
                    use crate::dma::{Burst, FifoThreshold};
                    use crate::dma::{Transfer, TransferOptions};

                    let dma_transfer_option = TransferOptions {
                        #[cfg(not(any(bdma, gpdma)))]
                        fifo_threshold: Some(FifoThreshold::Full),
                        #[cfg(not(any(bdma, gpdma)))]
                        mburst: Burst::Incr8,
                        ..Default::default()
                    };

                    Transfer::new_write(
                        &mut dma,
                        req,
                        duty,
                        T::regs_gp16().ccr(cc_channel.index()).as_ptr() as *mut _,
                        dma_transfer_option,
                    )
                    .await
                };

                // restore output compare state
                if !original_enable_state {
                    self.disable(cc_channel);
                }

                self.set_duty(cc_channel, original_duty_state);

                // Since DMA is closed before timer Capture Compare Event trigger DMA is turn off,
                // this can almost always trigger a DMA FIFO error.
                //
                // optional TODO:
                // clean FEIF after disable UDE
                if !original_cc_dma_enabled {
                    self.inner.set_cc_dma_enable_state(cc_channel, false);
                }

                if !original_cc_dma_on_update {
                    self.inner.set_cc_dma_selection(Ccds::ONCOMPARE)
                }
            }
        }
    };
}

impl_waveform_chx!(waveform_ch1, Ch1Dma, Ch1);
impl_waveform_chx!(waveform_ch2, Ch2Dma, Ch2);
impl_waveform_chx!(waveform_ch3, Ch3Dma, Ch3);
impl_waveform_chx!(waveform_ch4, Ch4Dma, Ch4);

impl<'d, T: CaptureCompare16bitInstance> embedded_hal_02::Pwm for SimplePwm<'d, T> {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u16;

    fn disable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, false);
    }

    fn enable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, true);
    }

    fn get_period(&self) -> Self::Time {
        self.inner.get_frequency()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.inner.get_compare_value(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.inner.get_max_compare_value() + 1
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into());
    }
}
