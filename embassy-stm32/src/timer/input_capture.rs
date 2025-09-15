//! Input capture driver.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::low_level::{CountingMode, FilterValue, InputCaptureMode, InputTISelection, Timer};
use super::{CaptureCompareInterruptHandler, Channel, GeneralInstance4Channel, TimerPin};
pub use super::{Ch1, Ch2, Ch3, Ch4};
use crate::gpio::{AfType, AnyPin, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::time::Hertz;
use crate::timer::TimerChannel;
use crate::Peri;

/// Capture pin wrapper.
///
/// This wraps a pin to make it usable with capture.
pub struct CapturePin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pin: Peri<'d, AnyPin>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}
impl<'d, T: GeneralInstance4Channel, C: TimerChannel, #[cfg(afio)] A> if_afio!(CapturePin<'d, T, C, A>) {
    /// Create a new capture pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl TimerPin<T, C, A>)>, pull: Pull) -> Self {
        set_as_af!(pin, AfType::input(pull));
        CapturePin {
            pin: pin.into(),
            phantom: PhantomData,
        }
    }
}

/// Input capture driver.
pub struct InputCapture<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> InputCapture<'d, T> {
    /// Create a new input capture driver.
    #[allow(unused)]
    pub fn new<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: Option<if_afio!(CapturePin<'d, T, Ch1, A>)>,
        ch2: Option<if_afio!(CapturePin<'d, T, Ch2, A>)>,
        ch3: Option<if_afio!(CapturePin<'d, T, Ch3, A>)>,
        ch4: Option<if_afio!(CapturePin<'d, T, Ch4, A>)>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: Peri<'d, T>, freq: Hertz, counting_mode: CountingMode) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_counting_mode(counting_mode);
        this.inner.set_tick_freq(freq);
        this.inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        this.inner.start();

        // enable NVIC interrupt
        T::CaptureCompareInterrupt::unpend();
        unsafe { T::CaptureCompareInterrupt::enable() };

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

    /// Set the input capture mode for a given channel.
    pub fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
        self.inner.set_input_capture_mode(channel, mode);
    }

    /// Set input TI selection.
    pub fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
        self.inner.set_input_ti_selection(channel, tisel)
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: Channel) -> u32 {
        self.inner.get_capture_value(channel)
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        self.inner.get_input_interrupt(channel)
    }

    fn new_future(&self, channel: Channel, mode: InputCaptureMode, tisel: InputTISelection) -> InputCaptureFuture<T> {
        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.5
        // or ST RM0008 (STM32F103) chapter 15.3.5 Input capture mode
        self.inner.set_input_ti_selection(channel, tisel);
        self.inner.set_input_capture_filter(channel, FilterValue::NO_FILTER);
        self.inner.set_input_capture_mode(channel, mode);
        self.inner.set_input_capture_prescaler(channel, 0);
        self.inner.enable_channel(channel, true);
        self.inner.enable_input_interrupt(channel, true);

        InputCaptureFuture {
            channel,
            phantom: PhantomData,
        }
    }

    /// Asynchronously wait until the pin sees a rising edge.
    pub async fn wait_for_rising_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Rising, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin sees a falling edge.
    pub async fn wait_for_falling_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Falling, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin sees any edge.
    pub async fn wait_for_any_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::BothEdges, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees a rising edge.
    pub async fn wait_for_rising_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Rising, InputTISelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees a falling edge.
    pub async fn wait_for_falling_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Falling, InputTISelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees any edge.
    pub async fn wait_for_any_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::BothEdges, InputTISelection::Alternate)
            .await
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputCaptureFuture<T: GeneralInstance4Channel> {
    channel: Channel,
    phantom: PhantomData<T>,
}

impl<T: GeneralInstance4Channel> Drop for InputCaptureFuture<T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

            // disable interrupt enable
            regs.dier().modify(|w| w.set_ccie(self.channel.index(), false));
        });
    }
}

impl<T: GeneralInstance4Channel> Future for InputCaptureFuture<T> {
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
