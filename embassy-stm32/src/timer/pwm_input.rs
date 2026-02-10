//! PWM Input driver.

use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::low_level::{CountingMode, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource};
use super::{CaptureCompareInterruptHandler, Ch1, Ch2, Channel, GeneralInstance4Channel, TimerPin};
use crate::Peri;
use crate::gpio::{AfType, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::time::Hertz;

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
    pub fn new_ch1<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        pin: Peri<'d, if_afio!(impl TimerPin<T, Ch1, A>)>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        set_as_af!(pin, AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch1, Channel::Ch2)
    }

    /// Create a new PWM input driver.
    pub fn new_ch2<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        pin: Peri<'d, if_afio!(impl TimerPin<T, Ch2, A>)>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        pull: Pull,
        freq: Hertz,
    ) -> Self {
        set_as_af!(pin, AfType::input(pull));

        Self::new_inner(tim, freq, Channel::Ch2, Channel::Ch1)
    }

    fn new_inner(tim: Peri<'d, T>, freq: Hertz, ch1: Channel, ch2: Channel) -> Self {
        let mut inner = Timer::new(tim);

        inner.set_counting_mode(CountingMode::EdgeAlignedUp);
        inner.set_tick_freq(freq);
        inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        inner.generate_update_event();
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

        // enable NVIC interrupt
        T::CaptureCompareInterrupt::unpend();
        unsafe { T::CaptureCompareInterrupt::enable() };

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
        self.inner.get_capture_value(self.channel).into()
    }

    /// Get the pulse width tick count
    pub fn get_width_ticks(&self) -> u32 {
        self.inner
            .get_capture_value(match self.channel {
                Channel::Ch1 => Channel::Ch2,
                Channel::Ch2 => Channel::Ch1,
                _ => panic!("Invalid channel for PWM input"),
            })
            .into()
    }

    /// Get the duty cycle in 100%
    pub fn get_duty_cycle(&self) -> f32 {
        let period = self.get_period_ticks();
        if period == 0 {
            return 0.;
        }
        100. * (self.get_width_ticks() as f32) / (period as f32)
    }

    fn new_future(&self, channel: Channel) -> PwmInputFuture<T> {
        self.inner.enable_channel(channel, true);
        self.inner.enable_input_interrupt(channel, true);

        PwmInputFuture {
            channel,
            phantom: PhantomData,
        }
    }

    /// Asynchronously wait until the pin sees a rising edge (period measurement).
    pub async fn wait_for_period(&self) -> u32 {
        self.new_future(self.channel.into()).await
    }

    /// Asynchronously wait until the pin sees a falling edge (width measurement).
    pub async fn wait_for_width(&self) -> u32 {
        self.new_future(
            match self.channel {
                Channel::Ch1 => Channel::Ch2,
                Channel::Ch2 => Channel::Ch1,
                _ => panic!("Invalid channel for PWM input"),
            }
            .into(),
        )
        .await
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct PwmInputFuture<T: GeneralInstance4Channel> {
    channel: Channel,
    phantom: PhantomData<T>,
}

impl<T: GeneralInstance4Channel> Drop for PwmInputFuture<T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

            // disable interrupt enable
            regs.dier().modify(|w| w.set_ccie(self.channel.index(), false));
        });
    }
}

impl<T: GeneralInstance4Channel> Future for PwmInputFuture<T> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::state().cc_waker[self.channel.index()].register(cx.waker());

        let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

        let dier = regs.dier().read();
        if !dier.ccie(self.channel.index()) {
            let val = regs.ccr(self.channel.index()).read().0;
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
