//! One pulse mode driver.

use core::future::Future;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::low_level::{
    CountingMode, FilterValue, InputCaptureMode, InputTISelection, SlaveMode, Timer, TriggerSource,
};
use super::{
    CaptureCompareInterruptHandler, Channel, Channel1Pin, Channel2Pin, ExternalTriggerPin, GeneralInstance4Channel,
};
pub use super::{Ch1, Ch2};
use crate::gpio::{AfType, AnyPin, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::timer::vals::Etp;
use crate::time::Hertz;
use crate::Peri;

/// External input marker type.
pub enum Ext {}

/// External trigger pin trigger polarity.
#[derive(Clone, Copy)]
pub enum ExternalTriggerPolarity {
    /// Rising edge only.
    Rising,
    /// Falling edge only.
    Falling,
}

impl From<ExternalTriggerPolarity> for Etp {
    fn from(mode: ExternalTriggerPolarity) -> Self {
        match mode {
            ExternalTriggerPolarity::Rising => 0.into(),
            ExternalTriggerPolarity::Falling => 1.into(),
        }
    }
}

/// Trigger pin wrapper.
///
/// This wraps a pin to make it usable as a timer trigger.
pub struct TriggerPin<'d, T, C> {
    _pin: Peri<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: GeneralInstance4Channel> TriggerPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " trigger pin instance.")]
            pub fn $new_chx(pin: Peri<'d, impl $pin_trait<T>>, pull: Pull) -> Self {
                pin.set_as_af(pin.af_num(), AfType::input(pull));
                TriggerPin {
                    _pin: pin.into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ext, Ext, ExternalTriggerPin);

/// One pulse driver.
///
/// Generates a pulse after a trigger and some configurable delay.
pub struct OnePulse<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> OnePulse<'d, T> {
    /// Create a new one pulse driver.
    ///
    /// The pulse is triggered by a channel 1 input pin on both rising and
    /// falling edges. Channel 1 will unusable as an output.
    pub fn new_ch1_edge_detect(
        tim: Peri<'d, T>,
        _pin: TriggerPin<'d, T, Ch1>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        pulse_end: u32,
        counting_mode: CountingMode,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_trigger_source(TriggerSource::TI1F_ED);
        this.inner
            .set_input_ti_selection(Channel::Ch1, InputTISelection::Normal);
        this.inner
            .set_input_capture_filter(Channel::Ch1, FilterValue::NO_FILTER);
        this.new_inner(freq, pulse_end, counting_mode);

        this
    }

    /// Create a new one pulse driver.
    ///
    /// The pulse is triggered by a channel 1 input pin. Channel 1 will unusable
    /// as an output.
    pub fn new_ch1(
        tim: Peri<'d, T>,
        _pin: TriggerPin<'d, T, Ch1>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        pulse_end: u32,
        counting_mode: CountingMode,
        capture_mode: InputCaptureMode,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_trigger_source(TriggerSource::TI1FP1);
        this.inner
            .set_input_ti_selection(Channel::Ch1, InputTISelection::Normal);
        this.inner
            .set_input_capture_filter(Channel::Ch1, FilterValue::NO_FILTER);
        this.inner.set_input_capture_mode(Channel::Ch1, capture_mode);
        this.new_inner(freq, pulse_end, counting_mode);

        this
    }

    /// Create a new one pulse driver.
    ///
    /// The pulse is triggered by a channel 2 input pin. Channel 2 will unusable
    /// as an output.
    pub fn new_ch2(
        tim: Peri<'d, T>,
        _pin: TriggerPin<'d, T, Ch1>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        pulse_end: u32,
        counting_mode: CountingMode,
        capture_mode: InputCaptureMode,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_trigger_source(TriggerSource::TI2FP2);
        this.inner
            .set_input_ti_selection(Channel::Ch2, InputTISelection::Normal);
        this.inner
            .set_input_capture_filter(Channel::Ch2, FilterValue::NO_FILTER);
        this.inner.set_input_capture_mode(Channel::Ch2, capture_mode);
        this.new_inner(freq, pulse_end, counting_mode);

        this
    }

    /// Create a new one pulse driver.
    ///
    /// The pulse is triggered by a external trigger input pin.
    pub fn new_ext(
        tim: Peri<'d, T>,
        _pin: TriggerPin<'d, T, Ext>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        pulse_end: u32,
        counting_mode: CountingMode,
        polarity: ExternalTriggerPolarity,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.regs_gp16().smcr().modify(|r| {
            r.set_etp(polarity.into());
            // No pre-scaling
            r.set_etps(0.into());
            // No filtering
            r.set_etf(FilterValue::NO_FILTER);
        });
        this.inner.set_trigger_source(TriggerSource::ETRF);
        this.new_inner(freq, pulse_end, counting_mode);

        this
    }

    fn new_inner(&mut self, freq: Hertz, pulse_end: u32, counting_mode: CountingMode) {
        self.inner.set_counting_mode(counting_mode);
        self.inner.set_tick_freq(freq);
        self.inner.set_max_compare_value(pulse_end);
        self.inner.regs_core().cr1().modify(|r| r.set_opm(true));
        // Required for advanced timers, see GeneralInstance4Channel for details
        self.inner.enable_outputs();
        self.inner.set_slave_mode(SlaveMode::TRIGGER_MODE);

        T::CaptureCompareInterrupt::unpend();
        unsafe { T::CaptureCompareInterrupt::enable() };
    }

    /// Get the end of the pulse in ticks from the trigger.
    pub fn pulse_end(&self) -> u32 {
        let max = self.inner.get_max_compare_value();
        assert!(max < u32::MAX);
        max + 1
    }

    /// Set the end of the pulse in ticks from the trigger.
    pub fn set_pulse_end(&mut self, ticks: u32) {
        self.inner.set_max_compare_value(ticks)
    }

    /// Reset the timer on each trigger
    #[cfg(not(stm32l0))]
    pub fn set_reset_on_trigger(&mut self, reset: bool) {
        let slave_mode = if reset {
            SlaveMode::COMBINED_RESET_TRIGGER
        } else {
            SlaveMode::TRIGGER_MODE
        };
        self.inner.set_slave_mode(slave_mode);
    }

    /// Get a single channel
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn channel(&mut self, channel: Channel) -> OnePulseChannel<'_, T> {
        OnePulseChannel {
            inner: unsafe { self.inner.clone_unchecked() },
            channel,
        }
    }

    /// Channel 1
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch1(&mut self) -> OnePulseChannel<'_, T> {
        self.channel(Channel::Ch1)
    }

    /// Channel 2
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch2(&mut self) -> OnePulseChannel<'_, T> {
        self.channel(Channel::Ch2)
    }

    /// Channel 3
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch3(&mut self) -> OnePulseChannel<'_, T> {
        self.channel(Channel::Ch3)
    }

    /// Channel 4
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch4(&mut self) -> OnePulseChannel<'_, T> {
        self.channel(Channel::Ch4)
    }

    /// Splits a [`OnePulse`] into four output channels.
    // TODO: I hate the name "split"
    pub fn split(self) -> OnePulseChannels<'static, T>
    where
        // must be static because the timer will never be dropped/disabled
        'd: 'static,
    {
        // without this, the timer would be disabled at the end of this function
        let timer = ManuallyDrop::new(self.inner);

        let ch = |channel| OnePulseChannel {
            inner: unsafe { timer.clone_unchecked() },
            channel,
        };

        OnePulseChannels {
            ch1: ch(Channel::Ch1),
            ch2: ch(Channel::Ch2),
            ch3: ch(Channel::Ch3),
            ch4: ch(Channel::Ch4),
        }
    }
}

/// A group of four [`OnePulseChannel`]s, obtained from [`OnePulse::split`].
pub struct OnePulseChannels<'d, T: GeneralInstance4Channel> {
    /// Channel 1
    pub ch1: OnePulseChannel<'d, T>,
    /// Channel 2
    pub ch2: OnePulseChannel<'d, T>,
    /// Channel 3
    pub ch3: OnePulseChannel<'d, T>,
    /// Channel 4
    pub ch4: OnePulseChannel<'d, T>,
}

/// A single channel of a one pulse-configured timer, obtained from
/// [`OnePulse::split`],[`OnePulse::channel`], [`OnePulse::ch1`], etc.
///
/// It is not possible to change the pulse end tick because the end tick
/// configuration is shared with all four channels.
pub struct OnePulseChannel<'d, T: GeneralInstance4Channel> {
    inner: ManuallyDrop<Timer<'d, T>>,
    channel: Channel,
}

impl<'d, T: GeneralInstance4Channel> OnePulseChannel<'d, T> {
    /// Get the end of the pulse in ticks from the trigger.
    pub fn pulse_end(&self) -> u32 {
        let max = self.inner.get_max_compare_value();
        assert!(max < u32::MAX);
        max + 1
    }

    /// Get the width of the pulse in ticks.
    pub fn pulse_width(&mut self) -> u32 {
        self.pulse_end().saturating_sub(self.pulse_delay())
    }

    /// Get the start of the pulse in ticks from the trigger.
    pub fn pulse_delay(&mut self) -> u32 {
        self.inner.get_compare_value(self.channel)
    }

    /// Set the start of the pulse in ticks from the trigger.
    pub fn set_pulse_delay(&mut self, delay: u32) {
        assert!(delay <= self.pulse_end());
        self.inner.set_compare_value(self.channel, delay);
    }

    /// Set the pulse width in ticks.
    pub fn set_pulse_width(&mut self, width: u32) {
        assert!(width <= self.pulse_end());
        self.set_pulse_delay(self.pulse_end() - width);
    }

    /// Waits until the trigger and following delay has passed.
    pub async fn wait_for_pulse_start(&mut self) {
        self.inner.enable_input_interrupt(self.channel, true);

        OnePulseFuture::<T> {
            channel: self.channel,
            phantom: PhantomData,
        }
        .await
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct OnePulseFuture<T: GeneralInstance4Channel> {
    channel: Channel,
    phantom: PhantomData<T>,
}

impl<'d, T: GeneralInstance4Channel> Drop for OnePulseFuture<T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

            // disable interrupt enable
            regs.dier().modify(|w| w.set_ccie(self.channel.index(), false));
        });
    }
}

impl<'d, T: GeneralInstance4Channel> Future for OnePulseFuture<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::state().cc_waker[self.channel.index()].register(cx.waker());

        let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

        let dier = regs.dier().read();
        if !dier.ccie(self.channel.index()) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
