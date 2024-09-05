//! PWM driver.

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::timer::{ChannelDirection, Timer};
use super::{Channel, Channel1Pin, Channel2Pin, Instance};
use crate::gpio::{AfType, AnyPin, OutputType, Speed};
use crate::time::Hertz;
use crate::Peripheral;

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct PwmPin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: Instance> PwmPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(
                        pin.af_num(),
                        AfType::output(OutputType::PushPull, Speed::VeryHigh),
                    );
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

/// PWM driver.
pub struct Pwm<'d, T: Instance> {
    inner: Timer<'d, T>, // _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Pwm<'d, T> {
    /// Create a new PWM driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1_pin: Option<PwmPin<'d, T, Ch1>>,
        _ch2_pin: Option<PwmPin<'d, T, Ch2>>,
        freq: Hertz,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.enable();
        this.set_frequency(freq);

        #[cfg(any(lptim_v2a, lptim_v2b))]
        [Channel::Ch1, Channel::Ch2].iter().for_each(|&channel| {
            this.inner.set_channel_direction(channel, ChannelDirection::OutputPwm);
        });

        this.inner.continuous_mode_start();

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
    pub fn set_frequency(&mut self, frequency: Hertz) {
        self.inner.set_frequency(frequency);
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
}
