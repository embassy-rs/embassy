//! PWM Input driver.

use super::low_level::{CountingMode, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use super::{Ch1, Ch2, Channel, GeneralInstance4Channel, TimerPin};
use crate::gpio::{AfType, Pull};
use crate::time::Hertz;
use crate::Peri;

/// PWM Input driver.
///
/// Only works with CH1 or CH2
/// Note: Not all timer peripherals are supported
/// Double check your chips reference manual
pub struct PwmInput<'d, T: GeneralInstance4Channel> {
    channel: Channel,
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> PwmInput<'d, T> {
    /// Create a new PWM input driver.
    pub fn new_ch1(tim: Peri<'d, T>, pin: Peri<'d, impl TimerPin<T, Ch1>>, pull: Pull, freq: Hertz) -> Self {
        pin.set_as_af(pin.af_num(), AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch1, Channel::Ch2)
    }

    /// Create a new PWM input driver.
    pub fn new_ch2(tim: Peri<'d, T>, pin: Peri<'d, impl TimerPin<T, Ch2>>, pull: Pull, freq: Hertz) -> Self {
        pin.set_as_af(pin.af_num(), AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch2, Channel::Ch1)
    }

    fn new_inner(tim: Peri<'d, T>, freq: Hertz, ch1: Channel, ch2: Channel) -> Self {
        let mut inner = Timer::new(tim);

        inner.set_counting_mode(CountingMode::EdgeAlignedUp);
        inner.set_tick_freq(freq);
        inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        inner.start();

        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.6
        // or ST RM0008 (STM32F103) chapter 15.3.6 Input capture mode
        // or ST RM0440 (STM32G4) chapter 30.4.8 PWM input mode
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

        Self { channel: ch1, inner }
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
        self.inner.get_channel_enable_state(Channel::Ch1)
    }

    /// Get the period tick count
    pub fn get_period_ticks(&self) -> u32 {
        self.inner.get_capture_value(self.channel)
    }

    /// Get the pulse width tick count
    pub fn get_width_ticks(&self) -> u32 {
        self.inner.get_capture_value(match self.channel {
            Channel::Ch1 => Channel::Ch2,
            Channel::Ch2 => Channel::Ch1,
            _ => panic!("Invalid channel for PWM input"),
        })
    }

    /// Get the duty cycle in 100%
    pub fn get_duty_cycle(&self) -> f32 {
        let period = self.get_period_ticks();
        if period == 0 {
            return 0.;
        }
        100. * (self.get_width_ticks() as f32) / (period as f32)
    }
}
