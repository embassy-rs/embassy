//! Input capture driver.

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::low_level::{CountingMode, InputCaptureMode, InputTISelection, Timer};
use super::{Channel, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin, GeneralInstance4Channel};
use crate::gpio::{AFType, AnyPin, Pull};
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
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, counting_mode: CountingMode) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_counting_mode(counting_mode);
        this.set_tick_freq(freq);
        this.inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        this.inner.start();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_input_capture_mode(channel, InputCaptureMode::Rising);

                this.inner.set_input_ti_selection(channel, InputTISelection::Normal);
            });

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
}
