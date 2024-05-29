//! Input capture driver.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::low_level::{CountingMode, InputCaptureMode, InputTISelection, Timer};
use super::{
    CaptureCompareInterruptHandler, Channel, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin,
    GeneralInstance4Channel,
};
use crate::gpio::{AFType, AnyPin, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::time::Hertz;
use crate::Peripheral;

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}
/// Channel 3 marker type.
pub enum Ch3 {}
/// Channel 4 marker type.
pub enum Ch4 {}

fn regs_gp16(ptr: *mut ()) -> crate::pac::timer::TimGp16 {
    unsafe { crate::pac::timer::TimGp16::from_ptr(ptr) }
}

/// Capture pin wrapper.
///
/// This wraps a pin to make it usable with capture.
pub struct CapturePin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: GeneralInstance4Channel> CapturePin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " capture pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd, pull_type: Pull) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_as_af_pull(pin.af_num(), AFType::Input, pull_type);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                CapturePin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ch3, Ch3, Channel3Pin);
channel_impl!(new_ch4, Ch4, Channel4Pin);

/// Input capture driver.
pub struct InputCapture<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> InputCapture<'d, T> {
    /// Create a new input capture driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<CapturePin<'d, T, Ch1>>,
        _ch2: Option<CapturePin<'d, T, Ch2>>,
        _ch3: Option<CapturePin<'d, T, Ch3>>,
        _ch4: Option<CapturePin<'d, T, Ch4>>,
        _irq: impl Binding<T::CaptureCompareInterrupt, CaptureCompareInterruptHandler<T>> + 'd,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_counting_mode(counting_mode);
        this.set_tick_freq(freq);
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

    /// Set tick frequency.
    ///
    /// Note: when you call this, the max period value changes
    pub fn set_tick_freq(&mut self, freq: Hertz) {
        let f = freq;
        assert!(f.0 > 0);
        let timer_f = self.inner.get_clock_frequency();

        let pclk_ticks_per_timer_period = timer_f / f;
        let psc: u16 = unwrap!((pclk_ticks_per_timer_period - 1).try_into());

        let regs = self.inner.regs_core();
        regs.psc().write_value(psc);

        // Generate an Update Request
        regs.egr().write(|r| r.set_ug(true));
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
        use stm32_metapac::timer::vals::*;

        let regs = regs_gp16(T::regs());
        let idx = channel.index();

        // Select the active input: TIMx_CCR1 must be linked to the TI1 input, so write the CC1S
        // bits to 01 in the TIMx_CCMR1 register. As soon as CC1S becomes different from 00,
        // the channel is configured in input and the TIMx_CCR1 register becomes read-only.
        regs.ccmr_input(idx / 2)
            .modify(|r| r.set_ccs(idx % 2, CcmrInputCcs::from(tisel)));

        // Program the appropriate input filter duration in relation with the signal connected to the
        // timer (by programming the ICxF bits in the TIMx_CCMRx register if the input is one of
        // the TIx inputs). Let’s imagine that, when toggling, the input signal is not stable during at
        // must 5 internal clock cycles. We must program a filter duration longer than these 5
        // clock cycles. We can validate a transition on TI1 when 8 consecutive samples with the
        // new level have been detected (sampled at fDTS frequency). Then write IC1F bits to
        // 0011 in the TIMx_CCMR1 register.
        regs.ccmr_input(idx / 2)
            .modify(|r| r.set_icf(idx % 2, FilterValue::NOFILTER));

        // Select the edge of the active transition on the TI1 channel by writing the CC1P and
        // CC1NP bits to 00 in the TIMx_CCER register (rising edge in this case).
        let ccpnp = match mode {
            InputCaptureMode::Rising => (false, false),
            InputCaptureMode::Falling => (false, true),
            InputCaptureMode::BothEdges => (true, true),
        };
        regs.ccer().modify(|r| {
            r.set_ccp(idx, ccpnp.0);
            r.set_ccnp(idx, ccpnp.1);
        });

        // Program the input prescaler. In our example, we wish the capture to be performed at
        // each valid transition, so the prescaler is disabled (write IC1PS bits to 00 in the
        // TIMx_CCMR1 register).
        regs.ccmr_input(idx / 2).modify(|r| r.set_icpsc(idx % 2, 0));

        // Enable capture from the counter into the capture register by setting the CC1E bit in the
        // TIMx_CCER register.
        regs.ccer().modify(|r| r.set_cce(idx, true));

        // If needed, enable the related interrupt request by setting the CC1IE bit in the
        // TIMx_DIER register, and/or the DMA request by setting the CC1DE bit in the
        // TIMx_DIER register.
        regs.dier().modify(|r| r.set_ccie(idx, true));

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
            let regs = regs_gp16(T::regs());

            // disable interrupt enable
            regs.dier().modify(|w| w.set_ccie(self.channel.index(), false));
        });
    }
}

impl<T: GeneralInstance4Channel> Future for InputCaptureFuture<T> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::state().cc_waker[self.channel.index()].register(cx.waker());

        let regs = regs_gp16(T::regs());

        let dier = regs.dier().read();
        if !dier.ccie(self.channel.index()) {
            let val = regs.ccr(self.channel.index()).read().0;
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
