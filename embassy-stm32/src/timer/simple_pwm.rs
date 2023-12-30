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
pub struct SimplePwm<'d, T, Dma> {
    inner: PeripheralRef<'d, T>,
    dma: PeripheralRef<'d, Dma>,
}

impl<'d, T: CaptureCompare16bitInstance, Dma> SimplePwm<'d, T, Dma> {
    /// Create a new simple PWM driver.
    ///
    /// Note:  
    /// If you want to use [`Self::gen_waveform()`], you need to provide corresponding TIMx_UP DMA channel.
    /// Otherwise you can just put a [`dma::NoDma`](crate::dma::NoDma)  
    /// Currently, you can only use one channel at a time to generate waveform with [`Self::gen_waveform()`].  
    /// But you can always use multiple TIM to generate multiple waveform simultaneously.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        freq: Hertz,
        counting_mode: CountingMode,
        dma: impl Peripheral<P = Dma> + 'd,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode, dma)
    }

    fn new_inner(
        tim: impl Peripheral<P = T> + 'd,
        freq: Hertz,
        counting_mode: CountingMode,
        dma: impl Peripheral<P = Dma> + 'd,
    ) -> Self {
        into_ref!(tim, dma);

        T::enable_and_reset();

        let mut this = Self { inner: tim, dma };

        this.inner.set_counting_mode(counting_mode);
        this.set_frequency(freq);
        this.inner.start();

        this.inner.enable_outputs();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
                this.inner.set_output_compare_preload(channel, true)
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
}

impl<'d, T: CaptureCompare16bitInstance + Basic16bitInstance, Dma> SimplePwm<'d, T, Dma>
where
    Dma: super::UpDma<T>,
{
    /// Generate a sequence of PWM waveform
    pub async fn gen_waveform(&mut self, channel: Channel, duty: &[u16]) {
        assert!(duty.iter().all(|v| *v <= self.get_max_duty()));

        #[cfg_attr(any(stm32f334, stm32f378), allow(clippy::let_unit_value))]
        let req = self.dma.request();

        #[cfg(not(any(bdma, gpdma)))]
        let (isr_bit, isr_reg, ifcr_reg) = {
            let dma_regs = self.dma.regs();
            let isr_num = self.dma.num() / 4;
            let isr_bit = self.dma.num() % 4;
            let isr_reg = dma_regs.isr(isr_num);
            let ifcr_reg = dma_regs.ifcr(isr_num);
            (isr_bit, isr_reg, ifcr_reg)
        };

        #[cfg(not(any(bdma, gpdma)))]
        // clean DMA FIFO error before a transfer
        if isr_reg.read().feif(isr_bit) {
            ifcr_reg.write(|v| v.set_feif(isr_bit, true));
        }

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
                &mut self.dma,
                req,
                duty,
                T::regs_gp16().ccr(channel.index()).as_ptr() as *mut _,
                dma_transfer_option,
            )
            .await
        };

        // restore output compare state
        if !original_enable_state {
            self.disable(channel);
        }
        self.set_duty(channel, original_duty_state);
        if !original_update_dma_state {
            self.inner.enable_update_dma(false);

            #[cfg(not(any(bdma, gpdma)))]
            // Since DMA could be closed before timer update event trigger DMA is turn off, this can almost always trigger a DMA FIFO error.
            // Thus, we will try clean DMA FEIF after each transfer
            if isr_reg.read().feif(isr_bit) {
                ifcr_reg.write(|v| v.set_feif(isr_bit, true));
            }
        }
    }
}

impl<'d, T: CaptureCompare16bitInstance, Dma> embedded_hal_02::Pwm for SimplePwm<'d, T, Dma> {
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
        self.inner.get_frequency().into()
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
