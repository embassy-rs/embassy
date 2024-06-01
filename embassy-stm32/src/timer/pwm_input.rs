//! PWM Input driver.

use super::low_level::{InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use super::raw::{RawTimer, RawTimerPin};
use super::{Ch1, Ch2, Channel, General2ChInstance, General2ChTim, TimerPin};
use crate::gpio::{AfType, Pull};
use crate::time::Hertz;
use crate::Peripheral;

/// PWM Input driver.
pub struct PwmInput<'d> {
    inner: Timer<'d, General2ChTim>,
    _pin: RawTimerPin<'d>,
    channel: Channel,
}

impl<'d> PwmInput<'d> {
    /// Create a new PWM input driver.
    pub fn new<T: General2ChInstance>(
        tim: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl TimerPin<T, Ch1>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        let raw = RawTimer::new_general_2ch(tim);
        let pin = RawTimerPin::new(pin, AfType::input(pull));
        Self::new_inner(raw, pin, freq, Channel::Ch1, Channel::Ch2)
    }

    /// Create a new PWM input driver.
    pub fn new_alt<T: General2ChInstance>(
        tim: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl TimerPin<T, Ch2>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        let raw = RawTimer::new_general_2ch(tim);
        let pin = RawTimerPin::new(pin, AfType::input(pull));
        Self::new_inner(raw, pin, freq, Channel::Ch2, Channel::Ch1)
    }

    fn new_inner(
        raw: RawTimer<'d, General2ChTim>,
        pin: RawTimerPin<'d>,
        freq: Hertz,
        ch1: Channel,
        ch2: Channel,
    ) -> Self {
        let inner = Timer::new(raw);

        inner.set_tick_frequency(freq);
        inner.enable_outputs(); // Required for advanced timers, see [`RawTimer::enable_outputs()`] for details
        inner.start();

        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.6
        // or ST RM0008 (STM32F103) chapter 15.3.6 Input capture mode
        inner.set_input_ti_selection(ch1, InputTISelection::Normal);
        inner.set_input_capture_mode(ch1, InputCaptureMode::Rising);

        inner.set_input_ti_selection(ch2, InputTISelection::Alternate);
        inner.set_input_capture_mode(ch2, InputCaptureMode::Falling);

        inner.set_trigger_source(match ch1 {
            Channel::Ch1 => TriggerSource::TI1FP1,
            Channel::Ch2 => TriggerSource::TI2FP2,
            _ => panic!("Invalid channel for PWM input"),
        });

        inner.set_slave_mode(SlaveMode::RESET_MODE);

        // Must call the `enable` function after

        Self {
            inner,
            _pin: pin,
            channel: ch1,
        }
    }

    /// Enable the given channel.
    pub fn enable(&mut self) {
        self.inner.enable_channel(Channel::Ch1, true);
        self.inner.enable_channel(Channel::Ch2, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self) {
        self.inner.enable_channel(Channel::Ch1, false);
        self.inner.enable_channel(Channel::Ch2, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.inner.channel_enable_state(Channel::Ch1)
    }

    /// Get the period tick count
    pub fn period_ticks(&self) -> u32 {
        self.inner.capture_value(self.channel)
    }

    /// Get the pulse width tick count
    pub fn width_ticks(&self) -> u32 {
        self.inner.capture_value(match self.channel {
            Channel::Ch1 => Channel::Ch2,
            Channel::Ch2 => Channel::Ch1,
            _ => panic!("Invalid channel for PWM input"),
        })
    }

    /// Get the duty cycle in 100%
    pub fn duty_cycle(&self) -> f32 {
        let period = self.period_ticks();
        if period == 0 {
            return 0.;
        }
        100. * (self.width_ticks() as f32) / (period as f32)
    }
}
