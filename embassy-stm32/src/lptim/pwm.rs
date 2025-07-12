//! PWM driver.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use super::timer::Timer;
#[cfg(not(any(lptim_v2a, lptim_v2b)))]
use super::OutputPin;
#[cfg(any(lptim_v2a, lptim_v2b))]
use super::{channel::Channel, timer::ChannelDirection, Channel1Pin, Channel2Pin};
use super::{BasicInstance, Instance};
#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, AnyPin, OutputType, Speed};
use crate::time::Hertz;

/// Output marker type.
pub enum Output {}
/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}

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
        impl<'d, T: BasicInstance> PwmPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: Peri<'d, impl $pin_trait<T>>) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(
                        pin.af_num(),
                        AfType::output(OutputType::PushPull, Speed::VeryHigh),
                    );
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

#[cfg(not(any(lptim_v2a, lptim_v2b)))]
channel_impl!(new, new_with_config, Output, OutputPin);
#[cfg(any(lptim_v2a, lptim_v2b))]
channel_impl!(new_ch1, new_ch1_with_config, Ch1, Channel1Pin);
#[cfg(any(lptim_v2a, lptim_v2b))]
channel_impl!(new_ch2, new_ch2_with_config, Ch2, Channel2Pin);

/// PWM driver.
pub struct Pwm<'d, T: Instance> {
    inner: Timer<'d, T>,
}

#[cfg(not(any(lptim_v2a, lptim_v2b)))]
impl<'d, T: Instance> Pwm<'d, T> {
    /// Create a new PWM driver.
    pub fn new(tim: Peri<'d, T>, _output_pin: PwmPin<'d, T, Output>, freq: Hertz) -> Self {
        Self::new_inner(tim, freq)
    }

    /// Set the duty.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, duty: u16) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(duty)
    }

    /// Get the duty.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn get_duty(&self) -> u16 {
        self.inner.get_compare_value()
    }

    fn post_init(&mut self) {}
}

#[cfg(any(lptim_v2a, lptim_v2b))]
impl<'d, T: Instance> Pwm<'d, T> {
    /// Create a new PWM driver.
    pub fn new(
        tim: Peri<'d, T>,
        _ch1_pin: Option<PwmPin<'d, T, Ch1>>,
        _ch2_pin: Option<PwmPin<'d, T, Ch2>>,
        freq: Hertz,
    ) -> Self {
        Self::new_inner(tim, freq)
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

    fn post_init(&mut self) {
        [Channel::Ch1, Channel::Ch2].iter().for_each(|&channel| {
            self.inner.set_channel_direction(channel, ChannelDirection::OutputPwm);
        });
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    fn new_inner(tim: Peri<'d, T>, freq: Hertz) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.enable();
        this.set_frequency(freq);

        this.post_init();

        this.inner.continuous_mode_start();

        this
    }

    /// Set PWM frequency.
    ///
    /// Note: when you call this, the max duty value changes, so you will have to
    /// call `set_duty` on all channels with the duty calculated based on the new max duty.
    pub fn set_frequency(&mut self, frequency: Hertz) {
        self.inner.set_frequency(frequency);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }
}
