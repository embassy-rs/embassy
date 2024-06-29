//! PWM Input driver.

use embassy_hal_internal::into_ref;

use super::ll_async::TimerEventFuture;
use super::low_level::{CountingMode, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use super::{Channel, Channel1Pin, Channel2Pin, GeneralInstance4Channel};
use crate::gpio::{AfType, Pull};
use crate::time::Hertz;
use crate::Peripheral;

/// PWM Input driver.
pub struct PwmInput<'d, T: GeneralInstance4Channel> {
    ch1: Channel,
    ch2: Channel,
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> PwmInput<'d, T> {
    /// Create a new PWM input driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        into_ref!(pin);

        pin.set_as_af(pin.af_num(), AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch1, Channel::Ch2)
    }

    /// Create a new PWM input driver.
    pub fn new_alt(
        tim: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        into_ref!(pin);

        pin.set_as_af(pin.af_num(), AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch2, Channel::Ch1)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, ch1: Channel, ch2: Channel) -> Self {
        let mut inner = Timer::new(tim);

        inner.set_counting_mode(CountingMode::EdgeAlignedUp);
        inner.set_tick_freq(freq);
        inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
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

        // enable NVIC interrupt
        T::CaptureCompareInterrupt::unpend();
        unsafe { T::CaptureCompareInterrupt::enable() };

        Self { ch1, ch2, inner }
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
        self.inner.get_capture_value(self.ch1)
    }

    /// Get the pulse width tick count
    pub fn get_width_ticks(&self) -> u32 {
        self.inner.get_capture_value(self.ch2)
    }

    /// Get the duty cycle in 100%
    pub fn get_duty_cycle(&self) -> f32 {
        let period = self.get_period_ticks();
        if period == 0 {
            return 0.;
        }
        100. * (self.get_width_ticks() as f32) / (period as f32)
    }

    /// Asynchronously wait until the pin sees a rising edge (period measurement).
    pub async fn wait_for_rising_edge(&self) -> u32 {
        self.inner.clear_input_interrupt(self.ch1);
        self.inner.enable_input_interrupt(self.ch1, true);

        // Rising edge is always on the main channel
        let future: TimerEventFuture<T> = TimerEventFuture::new(self.ch1.into());
        future.await
    }

    /// Asynchronously wait until the pin sees a falling edge (width measurement).
    pub async fn wait_for_falling_edge(&self) -> u32 {
        // Falling edge is always on the alternate channel
        self.inner.clear_input_interrupt(self.ch2);
        self.inner.enable_input_interrupt(self.ch2, true);

        let future: TimerEventFuture<T> = TimerEventFuture::new(self.ch2.into());
        future.await
    }
}
