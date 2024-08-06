//! Input capture driver.

use core::array;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::low_level::{CountingMode, FilterValue, InputCaptureMode, InputTISelection, Timer};
use super::raw::{RawTimer, RawTimerPin};
use super::{
    CaptureCompareInterruptHandler, Ch1, Ch2, Ch3, Ch4, Channel, ChannelMarker, CoreInstance, General1ChInstance,
    General1ChTim, General4ChInstance, General4ChTim, IsGeneral1ChTim, State, TimerPin,
};
use crate::gpio::{AfType, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt as _};
use crate::interrupt::{Interrupt, InterruptExt};
use crate::time::Hertz;
use crate::Peripheral;

/// Builder for [`InputCapture`] driver.
pub struct Builder<'d, T> {
    tim: PeripheralRef<'d, T>,
    channel_pins: [Option<RawTimerPin<'d>>; 4],
}

impl<'d, T: General1ChInstance> Builder<'d, T> {
    /// Start building an input capture driver from a timer peripheral.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(tim);
        let channel_pins = array::from_fn(|_| None);
        Self { tim, channel_pins }
    }

    /// Attach an input pin to the input capture driver.
    ///
    /// You may use convenience methods [`ch1_pin()`][Self::ch1_pin()] to `ch4_pin()` to aid type
    /// inference.
    pub fn pin<C: ChannelMarker>(mut self, pin: impl Peripheral<P = impl TimerPin<T, C>> + 'd, pull: Pull) -> Self {
        let pin = RawTimerPin::new(pin, AfType::input(pull));
        self.channel_pins[C::CHANNEL.index()] = Some(pin);
        self
    }
}

#[rustfmt::skip]
macro_rules! channel_impl {
    ($chx_pin:ident, $channel:ident) => {
        impl<'d, T: General1ChInstance> Builder<'d, T> {
            #[doc = concat!(
                "Attach an input pin for channel ",
                stringify!($channel),
                " to the input capture driver.\n\nSee [`pin()`][Self::pin()] for details.",
            )]
            pub fn $chx_pin(
                self,
                pin: impl Peripheral<P = impl TimerPin<T, $channel>> + 'd,
                pull: Pull,
            ) -> Self {
                self.pin::<$channel>(pin, pull)
            }
        }
    };
}
channel_impl!(ch1_pin, Ch1);
channel_impl!(ch2_pin, Ch2);
channel_impl!(ch3_pin, Ch3);
channel_impl!(ch4_pin, Ch4);

impl<'d, T: CoreInstance> Builder<'d, T>
where
    PeripheralRef<'d, T>: Peripheral<P = T> + 'd,
{
    /// Initialize the input capture driver with tick frequency `freq`.
    pub fn build(self, freq: Hertz) -> InputCapture<'d, General1ChTim>
    where
        T: General1ChInstance,
    {
        let raw = RawTimer::new_general_1ch(self.tim);
        let cc_interrupt = T::CaptureCompareInterrupt::IRQ;
        let state = T::state();
        InputCapture::new_inner(raw, self.channel_pins, cc_interrupt, state, |inner| {
            inner.set_tick_frequency(freq);
        })
    }

    /// Initialize the input capture driver with tick frequency `freq` and `counting_mode`.
    ///
    /// This method is only available if the timer peripheral implements [`General4ChInstance`],
    /// because less capable timers do not implement counting mode.
    pub fn build_4ch(self, freq: Hertz, counting_mode: CountingMode) -> InputCapture<'d, General4ChTim>
    where
        T: General4ChInstance,
    {
        let raw = RawTimer::new_general_4ch(self.tim);
        let cc_interrupt = T::CaptureCompareInterrupt::IRQ;
        let state = T::state();
        InputCapture::new_inner(raw, self.channel_pins, cc_interrupt, state, |inner| {
            inner.set_counting_mode(counting_mode);
            inner.set_tick_frequency(freq);
        })
    }
}

/// Input capture driver.
///
/// Use [`Builder`] to build an instance of this driver.
pub struct InputCapture<'d, Tim> {
    inner: Timer<'d, Tim>,
    channel_pins: [Option<RawTimerPin<'d>>; 4],
    cc_interrupt: Interrupt,
    state: &'d State,
}

impl<'d, Tim: IsGeneral1ChTim> InputCapture<'d, Tim> {
    fn new_inner(
        raw: RawTimer<'d, Tim>,
        channel_pins: [Option<RawTimerPin<'d>>; 4],
        cc_interrupt: Interrupt,
        state: &'d State,
        config_fn: impl FnOnce(&Timer<'d, Tim>),
    ) -> Self {
        let this = Self {
            inner: Timer::new(raw),
            channel_pins,
            cc_interrupt,
            state,
        };

        config_fn(&this.inner);
        this.inner.enable_outputs(); // Required for advanced timers, see [`RawTimer::enable_outputs()`] for details
        this.inner.start();

        // enable NVIC interrupt
        this.cc_interrupt.unpend();
        unsafe { this.cc_interrupt.enable() };

        this
    }

    /// Enable the given channel.
    pub fn enable(&mut self, channel: Channel) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.enable_channel(channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self, channel: Channel) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.enable_channel(channel, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self, channel: Channel) -> bool {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.channel_enable_state(channel)
    }

    /// Set the input capture mode for a given channel.
    pub fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.set_input_capture_mode(channel, mode);
    }

    /// Set input TI selection.
    pub fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.set_input_ti_selection(channel, tisel)
    }

    /// Get capture value for a channel.
    pub fn capture_value(&self, channel: Channel) -> u32 {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.capture_value(channel)
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.get_input_interrupt(channel)
    }

    fn new_future<'f>(
        &'f self,
        channel: Channel,
        mode: InputCaptureMode,
        tisel: InputTISelection,
    ) -> InputCaptureFuture<'f, Tim> {
        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.5
        // or ST RM0008 (STM32F103) chapter 15.3.5 Input capture mode
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.set_input_ti_selection(channel, tisel);
        self.inner.set_input_capture_filter(channel, FilterValue::NOFILTER);
        self.inner.set_input_capture_mode(channel, mode);
        self.inner.set_input_capture_prescaler(channel, 0);
        self.inner.enable_channel(channel, true);
        self.inner.enable_input_interrupt(channel, true);

        InputCaptureFuture { driver: self, channel }
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
struct InputCaptureFuture<'f, Tim: IsGeneral1ChTim> {
    driver: &'f InputCapture<'f, Tim>,
    channel: Channel,
}

impl<'f, Tim: IsGeneral1ChTim> Drop for InputCaptureFuture<'f, Tim> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            // disable interrupt enable
            self.driver
                .inner
                .raw
                .dier_1ch()
                .modify(|w| w.set_ccie(self.channel.index(), false));
        });
    }
}

impl<'f, Tim: IsGeneral1ChTim> Future for InputCaptureFuture<'f, Tim> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.driver.state.cc_waker[self.channel.index()].register(cx.waker());

        let dier = self.driver.inner.raw.dier_1ch().read();
        if !dier.ccie(self.channel.index()) {
            let val = self.driver.inner.raw.ccr(self.channel.index()).read().0;
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
