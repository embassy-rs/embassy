//! Input capture driver.

use core::future::Future;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::low_level::{CountingMode, FilterValue, InputCaptureMode, InputCaptureSelection, Timer};
use super::{CaptureCompareInterruptHandler, Channel, GeneralInstance4Channel, TimerPin};
pub use super::{Ch1, Ch2, Ch3, Ch4};
use crate::Peri;
#[cfg(not(stm32c5))]
use crate::dma;
use crate::gpio::{AfType, Flex, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::time::Hertz;
use crate::timer::{TimerChannel, TimerInputTrigger};

enum InputType<'d> {
    #[allow(dead_code)]
    Pin(Flex<'d>),
    #[allow(dead_code)]
    Trigger(u8),
}

/// Capture pin wrapper.
pub struct CapturePin {
    _private: (),
}

impl CapturePin {
    /// Create a new capture pin instance.
    #[deprecated = "use `CaptureInput::from_pin`"]
    pub fn new<'d, T: GeneralInstance4Channel, C: TimerChannel, #[cfg(afio)] A>(
        pin: Peri<'d, if_afio!(impl TimerPin<T, C, A>)>,
        pull: Pull,
    ) -> if_afio!(CaptureInput<'d, T, C, A>) {
        CaptureInput::from_pin(pin, pull).unwrap()
    }
}

/// Capture pin wrapper.
///
/// This wraps a pin or trigger to make it usable with capture.
pub struct CaptureInput<'d, T, C, #[cfg(afio)] A> {
    input: InputType<'d>,
    _marker: PhantomData<if_afio!((T, C, A))>,
}
impl<'d, T: GeneralInstance4Channel, C: TimerChannel, #[cfg(afio)] A> if_afio!(CaptureInput<'d, T, C, A>) {
    /// Create a new capture pin instance.
    pub fn from_pin(pin: Peri<'d, if_afio!(impl TimerPin<T, C, A>)>, pull: Pull) -> Option<Self> {
        set_as_af!(pin, AfType::input(pull));
        Some(Self {
            input: InputType::Pin(Flex::new(pin)),
            _marker: PhantomData,
        })
    }

    /// Create a new capture pin instance.
    pub fn from_trigger(trigger: impl TimerInputTrigger<T, C>) -> Option<Self> {
        Some(Self {
            input: InputType::Trigger(trigger.signal()),
            _marker: PhantomData,
        })
    }
}

/// Input capture driver.
pub struct InputCapture<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
    _ch1: Option<InputType<'d>>,
    _ch2: Option<InputType<'d>>,
    _ch3: Option<InputType<'d>>,
    _ch4: Option<InputType<'d>>,
}

impl<'d, T: GeneralInstance4Channel> InputCapture<'d, T> {
    /// Create a new input capture driver.
    #[allow(unused)]
    pub fn new<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: Option<if_afio!(CaptureInput<'d, T, Ch1, A>)>,
        ch2: Option<if_afio!(CaptureInput<'d, T, Ch2, A>)>,
        ch3: Option<if_afio!(CaptureInput<'d, T, Ch3, A>)>,
        ch4: Option<if_afio!(CaptureInput<'d, T, Ch4, A>)>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(
            tim,
            ch1.map(|input| input.input),
            ch2.map(|input| input.input),
            ch3.map(|input| input.input),
            ch4.map(|input| input.input),
            freq,
            counting_mode,
        )
    }

    fn new_inner(
        tim: Peri<'d, T>,
        _ch1: Option<InputType<'d>>,
        _ch2: Option<InputType<'d>>,
        _ch3: Option<InputType<'d>>,
        _ch4: Option<InputType<'d>>,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        let mut this = Self {
            inner: Timer::new(tim),
            _ch1,
            _ch2,
            _ch3,
            _ch4,
        };

        this.inner.set_counting_mode(counting_mode);
        this.inner.set_tick_freq(freq);
        for (ch, _pin_trigger) in [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .zip([&this._ch1, &this._ch2, &this._ch3, &this._ch4])
        {
            #[cfg(not(stm32l0))]
            if let Some(pin_trigger) = _pin_trigger {
                let tisel = match pin_trigger {
                    InputType::Pin(_) => 0,
                    InputType::Trigger(trigger) => *trigger,
                };

                this.inner.set_input_ti_seletion(*ch, tisel);
            }

            this.inner.set_input_capture_filter(*ch, FilterValue::NoFilter);
            this.inner.set_input_capture_prescaler(*ch, 0);
        }
        this.inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        this.inner.generate_update_event();
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

    /// Set input capture selection.
    pub fn set_input_capture_selection(&mut self, channel: Channel, icsel: InputCaptureSelection) {
        self.inner.set_input_capture_selection(channel, icsel)
    }

    /// Set the input capture filter for a given channel.
    pub fn set_input_capture_filter(&mut self, channel: Channel, filter: FilterValue) {
        self.inner.set_input_capture_filter(channel, filter);
    }

    /// Set the input capture prescaler for a given channel.
    pub fn set_input_capture_prescaler(&mut self, channel: Channel, factor: u8) {
        self.inner.set_input_capture_prescaler(channel, factor);
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: Channel) -> T::Word {
        self.inner.get_capture_value(channel)
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        self.inner.get_input_interrupt(channel)
    }

    /// Asynchronously wait until the pin or trigger sees a rising edge.
    pub async fn wait_for_rising_edge(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_rising_edge().await
    }

    /// Asynchronously wait until the pin or trigger sees a falling edge.
    pub async fn wait_for_falling_edge(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_falling_edge().await
    }

    /// Asynchronously wait until the pin or trigger sees any edge.
    pub async fn wait_for_any_edge(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_any_edge().await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees a rising edge.
    pub async fn wait_for_rising_edge_alternate(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_rising_edge_alternate().await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees a falling edge.
    pub async fn wait_for_falling_edge_alternate(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_falling_edge_alternate().await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees any edge.
    pub async fn wait_for_any_edge_alternate(&mut self, channel: Channel) -> T::Word {
        self.channel(channel).wait_for_any_edge_alternate().await
    }

    /// Get a single channel.
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn channel(&mut self, channel: Channel) -> InputCaptureChannel<'_, T> {
        InputCaptureChannel {
            inner: unsafe { self.inner.clone_unchecked() },
            channel,
            _pin: None,
        }
    }

    /// Channel 1
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch1(&mut self) -> InputCaptureChannel<'_, T> {
        self.channel(Channel::Ch1)
    }

    /// Channel 2
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch2(&mut self) -> InputCaptureChannel<'_, T> {
        self.channel(Channel::Ch2)
    }

    /// Channel 3
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch3(&mut self) -> InputCaptureChannel<'_, T> {
        self.channel(Channel::Ch3)
    }

    /// Channel 4
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch4(&mut self) -> InputCaptureChannel<'_, T> {
        self.channel(Channel::Ch4)
    }

    /// Splits an [`InputCapture`] into four capture channels.
    pub fn split(self) -> InputCaptureChannels<'static, T>
    where
        // must be static because the timer will never be dropped/disabled
        'd: 'static,
    {
        // without this, the timer would be disabled at the end of this function
        let timer = ManuallyDrop::new(self.inner);

        let ch = |channel, pin| InputCaptureChannel {
            inner: unsafe { timer.clone_unchecked() },
            channel,
            _pin: pin,
        };

        InputCaptureChannels {
            ch1: ch(Channel::Ch1, self._ch1),
            ch2: ch(Channel::Ch2, self._ch2),
            ch3: ch(Channel::Ch3, self._ch3),
            ch4: ch(Channel::Ch4, self._ch4),
        }
    }

    #[cfg(not(stm32c5))]
    /// Capture a sequence of timer input edges into a buffer using DMA.
    ///
    /// Note: DMA capture is only available on `InputCapture`, not on the per-channel
    /// [`InputCaptureChannel`] handles returned by [`Self::split`].
    pub async fn receive_waveform<M, D: super::Dma<T, M>>(
        &mut self,
        dma: Peri<'_, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>,
        channel: M,
        buf: &mut [u16],
    ) where
        M: TimerChannel,
    {
        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();
        let _ = channel;

        let original_enable_state = self.is_enabled(M::CHANNEL);
        let original_cc_dma_enable_state = self.inner.get_cc_dma_enable_state(M::CHANNEL);

        self.inner
            .set_input_capture_selection(M::CHANNEL, InputCaptureSelection::Normal);
        self.inner
            .set_input_capture_mode(M::CHANNEL, InputCaptureMode::BothEdges);

        if !original_cc_dma_enable_state {
            self.inner.set_cc_dma_enable_state(M::CHANNEL, true);
        }

        if !original_enable_state {
            self.enable(M::CHANNEL);
        }

        unsafe {
            let mut dma_channel = dma::Channel::new(dma, irq);
            dma_channel
                .read(
                    req,
                    self.inner.regs_gp16().ccr(M::CHANNEL.index()).as_ptr() as *mut u16,
                    buf,
                    dma::TransferOptions::default(),
                )
                .await
        };

        // restore output compare state
        if !original_enable_state {
            self.disable(M::CHANNEL);
        }
    }
}

/// A group of four [`InputCaptureChannel`]s, obtained from [`InputCapture::split`].
pub struct InputCaptureChannels<'d, T: GeneralInstance4Channel> {
    /// Channel 1
    pub ch1: InputCaptureChannel<'d, T>,
    /// Channel 2
    pub ch2: InputCaptureChannel<'d, T>,
    /// Channel 3
    pub ch3: InputCaptureChannel<'d, T>,
    /// Channel 4
    pub ch4: InputCaptureChannel<'d, T>,
}

/// A single channel of an input capture-configured timer, obtained from
/// [`InputCapture::split`], [`InputCapture::channel`], [`InputCapture::ch1`], etc.
pub struct InputCaptureChannel<'d, T: GeneralInstance4Channel> {
    inner: ManuallyDrop<Timer<'d, T>>,
    channel: Channel,
    _pin: Option<InputType<'d>>,
}

impl<'d, T: GeneralInstance4Channel> InputCaptureChannel<'d, T> {
    /// Enable this channel.
    pub fn enable(&mut self) {
        self.inner.enable_channel(self.channel, true);
    }

    /// Disable this channel.
    pub fn disable(&mut self) {
        self.inner.enable_channel(self.channel, false);
    }

    /// Check whether this channel is enabled.
    pub fn is_enabled(&self) -> bool {
        self.inner.get_channel_enable_state(self.channel)
    }

    /// Set the input capture mode for this channel.
    pub fn set_input_capture_mode(&mut self, mode: InputCaptureMode) {
        self.inner.set_input_capture_mode(self.channel, mode);
    }

    /// Set input capture selection for this channel.
    pub fn set_input_capture_selection(&mut self, icsel: InputCaptureSelection) {
        self.inner.set_input_capture_selection(self.channel, icsel);
    }

    /// Set the input capture filter for this channel.
    pub fn set_input_capture_filter(&mut self, filter: FilterValue) {
        self.inner.set_input_capture_filter(self.channel, filter);
    }

    /// Set the input capture prescaler for this channel.
    pub fn set_input_capture_prescaler(&mut self, factor: u8) {
        self.inner.set_input_capture_prescaler(self.channel, factor);
    }

    /// Get capture value for this channel.
    pub fn get_capture_value(&self) -> T::Word {
        self.inner.get_capture_value(self.channel)
    }

    /// Get input interrupt for this channel.
    pub fn get_input_interrupt(&self) -> bool {
        self.inner.get_input_interrupt(self.channel)
    }

    fn new_future(&self, mode: InputCaptureMode, icsel: InputCaptureSelection) -> InputCaptureFuture<T> {
        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.5
        // or ST RM0008 (STM32F103) chapter 15.3.5 Input capture mode
        self.inner.set_input_capture_selection(self.channel, icsel);
        self.inner.set_input_capture_mode(self.channel, mode);
        self.inner.enable_channel(self.channel, true);
        self.inner.clear_input_interrupt(self.channel);
        self.inner.enable_input_interrupt(self.channel, true);

        InputCaptureFuture {
            channel: self.channel,
            phantom: PhantomData,
        }
    }

    /// Asynchronously wait until the pin or trigger sees a rising edge.
    pub async fn wait_for_rising_edge(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::Rising, InputCaptureSelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin or trigger sees a falling edge.
    pub async fn wait_for_falling_edge(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::Falling, InputCaptureSelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin or trigger sees any edge.
    pub async fn wait_for_any_edge(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::BothEdges, InputCaptureSelection::Normal)
            .await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees a rising edge.
    pub async fn wait_for_rising_edge_alternate(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::Rising, InputCaptureSelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees a falling edge.
    pub async fn wait_for_falling_edge_alternate(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::Falling, InputCaptureSelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin or trigger sees any edge.
    pub async fn wait_for_any_edge_alternate(&mut self) -> T::Word {
        self.new_future(InputCaptureMode::BothEdges, InputCaptureSelection::Alternate)
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
    type Output = T::Word;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[cfg(stm32l0)]
        use crate::pac::timer::TimGp16 as timer;
        #[cfg(not(stm32l0))]
        use crate::pac::timer::TimGp32 as timer;

        T::state().cc_waker[self.channel.index()].register(cx.waker());

        let regs = unsafe { timer::from_ptr(T::regs()) };

        let dier = regs.dier().read();
        if !dier.ccie(self.channel.index()) {
            #[cfg(not(stm32l0))]
            let val = unwrap!(regs.ccr(self.channel.index()).read().try_into());
            #[cfg(stm32l0)]
            let val = unwrap!(regs.ccr(self.channel.index()).read().ccr().try_into());
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
