//! Simple PWM driver.

use core::marker::PhantomData;
use core::mem::ManuallyDrop;

use super::low_level::{CountingMode, OutputCompareMode, OutputPolarity, Timer};
use super::{Channel, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin, GeneralInstance4Channel, TimerBits};
#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, AnyPin, OutputType, Speed};
use crate::time::Hertz;
use crate::Peri;

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
    _pin: Peri<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

/// PWM pin config
///
/// This configures the pwm pin settings
pub struct PwmPinConfig {
    /// PWM Pin output type
    pub output_type: OutputType,
    /// PWM Pin speed
    pub speed: Speed,
    /// PWM Pin pull type
    #[cfg(gpio_v2)]
    pub pull: Pull,
}

macro_rules! channel_impl {
    ($new_chx:ident, $new_chx_with_config:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: GeneralInstance4Channel> PwmPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: Peri<'d, impl $pin_trait<T>>, output_type: OutputType) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AfType::output(output_type, Speed::VeryHigh));
                });
                PwmPin {
                    _pin: pin.into(),
                    phantom: PhantomData,
                }
            }

            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance with config.")]
            pub fn $new_chx_with_config(pin: Peri<'d, impl $pin_trait<T>>, pin_config: PwmPinConfig) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(
                        pin.af_num(),
                        #[cfg(gpio_v1)]
                        AfType::output(pin_config.output_type, pin_config.speed),
                        #[cfg(gpio_v2)]
                        AfType::output_pull(pin_config.output_type, pin_config.speed, pin_config.pull),
                    );
                });
                PwmPin {
                    _pin: pin.into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, new_ch1_with_config, Ch1, Channel1Pin);
channel_impl!(new_ch2, new_ch2_with_config, Ch2, Channel2Pin);
channel_impl!(new_ch3, new_ch3_with_config, Ch3, Channel3Pin);
channel_impl!(new_ch4, new_ch4_with_config, Ch4, Channel4Pin);

/// A single channel of a pwm, obtained from [`SimplePwm::split`],
/// [`SimplePwm::channel`], [`SimplePwm::ch1`], etc.
///
/// It is not possible to change the pwm frequency because
/// the frequency configuration is shared with all four channels.
pub struct SimplePwmChannel<'d, T: GeneralInstance4Channel> {
    timer: ManuallyDrop<Timer<'d, T>>,
    channel: Channel,
}

// TODO: check for RMW races
impl<'d, T: GeneralInstance4Channel> SimplePwmChannel<'d, T> {
    /// Enable the given channel.
    pub fn enable(&mut self) {
        self.timer.enable_channel(self.channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self) {
        self.timer.enable_channel(self.channel, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.timer.get_channel_enable_state(self.channel)
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_duty_cycle(&self) -> u16 {
        let max = self.timer.get_max_compare_value();
        assert!(max < u16::MAX as u32);
        max as u16 + 1
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty_cycle`](Self::max_duty_cycle) for 100% duty, both included.
    pub fn set_duty_cycle(&mut self, duty: u16) {
        assert!(duty <= (*self).max_duty_cycle());
        self.timer.set_compare_value(self.channel, duty.into())
    }

    /// Set the duty cycle to 0%, or always inactive.
    pub fn set_duty_cycle_fully_off(&mut self) {
        self.set_duty_cycle(0);
    }

    /// Set the duty cycle to 100%, or always active.
    pub fn set_duty_cycle_fully_on(&mut self) {
        self.set_duty_cycle((*self).max_duty_cycle());
    }

    /// Set the duty cycle to `num / denom`.
    ///
    /// The caller is responsible for ensuring that `num` is less than or equal to `denom`,
    /// and that `denom` is not zero.
    pub fn set_duty_cycle_fraction(&mut self, num: u16, denom: u16) {
        assert!(denom != 0);
        assert!(num <= denom);
        let duty = u32::from(num) * u32::from(self.max_duty_cycle()) / u32::from(denom);

        // This is safe because we know that `num <= denom`, so `duty <= self.max_duty_cycle()` (u16)
        #[allow(clippy::cast_possible_truncation)]
        self.set_duty_cycle(duty as u16);
    }

    /// Set the duty cycle to `percent / 100`
    ///
    /// The caller is responsible for ensuring that `percent` is less than or equal to 100.
    pub fn set_duty_cycle_percent(&mut self, percent: u8) {
        self.set_duty_cycle_fraction(u16::from(percent), 100)
    }

    /// Get the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty_cycle`](Self::max_duty_cycle) for 100% duty, both included.
    pub fn current_duty_cycle(&self) -> u16 {
        unwrap!(self.timer.get_compare_value(self.channel).try_into())
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, polarity: OutputPolarity) {
        self.timer.set_output_polarity(self.channel, polarity);
    }

    /// Set the output compare mode for a given channel.
    pub fn set_output_compare_mode(&mut self, mode: OutputCompareMode) {
        self.timer.set_output_compare_mode(self.channel, mode);
    }
}

/// A group of four [`SimplePwmChannel`]s, obtained from [`SimplePwm::split`].
pub struct SimplePwmChannels<'d, T: GeneralInstance4Channel> {
    /// Channel 1
    pub ch1: SimplePwmChannel<'d, T>,
    /// Channel 2
    pub ch2: SimplePwmChannel<'d, T>,
    /// Channel 3
    pub ch3: SimplePwmChannel<'d, T>,
    /// Channel 4
    pub ch4: SimplePwmChannel<'d, T>,
}

/// Simple PWM driver.
pub struct SimplePwm<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> SimplePwm<'d, T> {
    /// Create a new simple PWM driver.
    pub fn new(
        tim: Peri<'d, T>,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: Peri<'d, T>, freq: Hertz, counting_mode: CountingMode) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_counting_mode(counting_mode);
        this.set_frequency(freq);
        this.inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        this.inner.start();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);

                this.inner.set_output_compare_preload(channel, true);
            });

        this
    }

    /// Get a single channel
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn channel(&mut self, channel: Channel) -> SimplePwmChannel<'_, T> {
        SimplePwmChannel {
            timer: unsafe { self.inner.clone_unchecked() },
            channel,
        }
    }

    /// Channel 1
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch1(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch1)
    }

    /// Channel 2
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch2(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch2)
    }

    /// Channel 3
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch3(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch3)
    }

    /// Channel 4
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch4(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch4)
    }

    /// Splits a [`SimplePwm`] into four pwm channels.
    ///
    /// This returns all four channels, including channels that
    /// aren't configured with a [`PwmPin`].
    // TODO: I hate the name "split"
    pub fn split(self) -> SimplePwmChannels<'static, T>
    where
        // must be static because the timer will never be dropped/disabled
        'd: 'static,
    {
        // without this, the timer would be disabled at the end of this function
        let timer = ManuallyDrop::new(self.inner);

        let ch = |channel| SimplePwmChannel {
            timer: unsafe { timer.clone_unchecked() },
            channel,
        };

        SimplePwmChannels {
            ch1: ch(Channel::Ch1),
            ch2: ch(Channel::Ch2),
            ch3: ch(Channel::Ch3),
            ch4: ch(Channel::Ch4),
        }
    }

    /// Set PWM frequency.
    ///
    /// Note: when you call this, the max duty value changes, so you will have to
    /// call `set_duty` on all channels with the duty calculated based on the new max duty.
    pub fn set_frequency(&mut self, freq: Hertz) {
        // TODO: prevent ARR = u16::MAX?
        let multiplier = if self.inner.get_counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.inner.set_frequency_internal(freq * multiplier, 16);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_duty_cycle(&self) -> u16 {
        let max = self.inner.get_max_compare_value();
        assert!(max < u16::MAX as u32);
        max as u16 + 1
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub async fn waveform_up(&mut self, dma: Peri<'_, impl super::UpDma<T>>, channel: Channel, duty: &[u16]) {
        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        let original_duty_state = self.channel(channel).current_duty_cycle();
        let original_enable_state = self.channel(channel).is_enabled();
        let original_update_dma_state = self.inner.get_update_dma_state();

        if !original_update_dma_state {
            self.inner.enable_update_dma(true);
        }

        if !original_enable_state {
            self.channel(channel).enable();
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
                dma,
                req,
                duty,
                self.inner.regs_1ch().ccr(channel.index()).as_ptr() as *mut u16,
                dma_transfer_option,
            )
            .await
        };

        // restore output compare state
        if !original_enable_state {
            self.channel(channel).disable();
        }

        self.channel(channel).set_duty_cycle(original_duty_state);

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
        impl<'d, T: GeneralInstance4Channel> SimplePwm<'d, T> {
            /// Generate a sequence of PWM waveform
            pub async fn $fn_name(&mut self, dma: Peri<'_, impl super::$dma_ch<T>>, duty: &[u16]) {
                use crate::pac::timer::vals::Ccds;

                #[allow(clippy::let_unit_value)] // eg. stm32f334
                let req = dma.request();

                let cc_channel = Channel::$cc_ch;

                let original_duty_state = self.channel(cc_channel).current_duty_cycle();
                let original_enable_state = self.channel(cc_channel).is_enabled();
                let original_cc_dma_on_update = self.inner.get_cc_dma_selection() == Ccds::ON_UPDATE;
                let original_cc_dma_enabled = self.inner.get_cc_dma_enable_state(cc_channel);

                // redirect CC DMA request onto Update Event
                if !original_cc_dma_on_update {
                    self.inner.set_cc_dma_selection(Ccds::ON_UPDATE)
                }

                if !original_cc_dma_enabled {
                    self.inner.set_cc_dma_enable_state(cc_channel, true);
                }

                if !original_enable_state {
                    self.channel(cc_channel).enable();
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

                    match self.inner.bits() {
                        TimerBits::Bits16 => {
                            Transfer::new_write(
                                dma,
                                req,
                                duty,
                                self.inner.regs_gp16().ccr(cc_channel.index()).as_ptr() as *mut u16,
                                dma_transfer_option,
                            )
                            .await
                        }
                        #[cfg(not(any(stm32l0)))]
                        TimerBits::Bits32 => {
                            #[cfg(not(any(bdma, gpdma)))]
                            panic!("unsupported timer bits");

                            #[cfg(any(bdma, gpdma))]
                            Transfer::new_write(
                                dma,
                                req,
                                duty,
                                self.inner.regs_gp16().ccr(cc_channel.index()).as_ptr() as *mut u32,
                                dma_transfer_option,
                            )
                            .await
                        }
                    };
                };

                // restore output compare state
                if !original_enable_state {
                    self.channel(cc_channel).disable();
                }

                self.channel(cc_channel).set_duty_cycle(original_duty_state);

                // Since DMA is closed before timer Capture Compare Event trigger DMA is turn off,
                // this can almost always trigger a DMA FIFO error.
                //
                // optional TODO:
                // clean FEIF after disable UDE
                if !original_cc_dma_enabled {
                    self.inner.set_cc_dma_enable_state(cc_channel, false);
                }

                if !original_cc_dma_on_update {
                    self.inner.set_cc_dma_selection(Ccds::ON_COMPARE)
                }
            }
        }
    };
}

impl_waveform_chx!(waveform_ch1, Ch1Dma, Ch1);
impl_waveform_chx!(waveform_ch2, Ch2Dma, Ch2);
impl_waveform_chx!(waveform_ch3, Ch3Dma, Ch3);
impl_waveform_chx!(waveform_ch4, Ch4Dma, Ch4);

impl<'d, T: GeneralInstance4Channel> embedded_hal_1::pwm::ErrorType for SimplePwmChannel<'d, T> {
    type Error = core::convert::Infallible;
}

impl<'d, T: GeneralInstance4Channel> embedded_hal_1::pwm::SetDutyCycle for SimplePwmChannel<'d, T> {
    fn max_duty_cycle(&self) -> u16 {
        self.max_duty_cycle()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.set_duty_cycle(duty);
        Ok(())
    }

    fn set_duty_cycle_fully_off(&mut self) -> Result<(), Self::Error> {
        self.set_duty_cycle_fully_off();
        Ok(())
    }

    fn set_duty_cycle_fully_on(&mut self) -> Result<(), Self::Error> {
        self.set_duty_cycle_fully_on();
        Ok(())
    }

    fn set_duty_cycle_fraction(&mut self, num: u16, denom: u16) -> Result<(), Self::Error> {
        self.set_duty_cycle_fraction(num, denom);
        Ok(())
    }

    fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), Self::Error> {
        self.set_duty_cycle_percent(percent);
        Ok(())
    }
}

impl<'d, T: GeneralInstance4Channel> embedded_hal_02::Pwm for SimplePwm<'d, T> {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u32;

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
        assert!(duty <= self.max_duty_cycle() as u32);
        self.inner.set_compare_value(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into());
    }
}
