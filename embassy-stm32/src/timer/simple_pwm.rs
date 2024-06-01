//! Simple PWM driver.

use core::array;

use super::low_level::{CountingMode, OutputCompareMode, OutputPolarity, Timer};
use super::raw::{self, RawTimer, RawTimerPin};
use super::{
    CcDma, CcDmaInstance, CcDmaTim, Ch1, Ch2, Ch3, Ch4, Channel, ChannelMarker, CoreInstance, General1ChInstance,
    General1ChTim, General4ChInstance, General4ChTim, IsCcDmaTim, IsGeneral1ChTim, IsGeneral4ChTim, TimerPin, UpDma,
};
use crate::gpio::{AfType, OutputType, Speed};
use crate::time::Hertz;
use crate::{dma, into_ref, Peripheral, PeripheralRef};

/// Builder for [`SimplePwm`] driver.
///
/// Create the builder using [`Builder::new()`], then attach output pins and DMAs using methods on
/// the builder, and finally build the [`SimplePwm`] driver using one of the `build` methods().
pub struct Builder<'d, T> {
    tim: PeripheralRef<'d, T>,
    up_dma: Option<dma::ChannelAndRequest<'d>>,
    channel_pins: [Option<RawTimerPin<'d>>; 4],
    cc_dmas: [Option<dma::ChannelAndRequest<'d>>; 4],
}

impl<'d, T: CoreInstance> Builder<'d, T> {
    /// Start building a PWM driver from a timer peripheral.
    pub fn new(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);
        Self {
            tim,
            up_dma: None,
            channel_pins: array::from_fn(|_| None),
            cc_dmas: array::from_fn(|_| None),
        }
    }

    /// Attach an output pin to the PWM driver.
    ///
    /// You may use convenience methods [`ch1_pin()`][Self::ch1_pin()] to `ch4_pin()` to aid type
    /// inference.
    pub fn pin<C: ChannelMarker>(
        &mut self,
        pin: impl Peripheral<P = impl TimerPin<T, C>> + 'd,
        output_type: OutputType,
    ) -> &mut Self {
        let pin = RawTimerPin::new(pin, AfType::output(output_type, Speed::VeryHigh));
        self.channel_pins[C::CHANNEL.index()] = Some(pin);
        self
    }

    /// Attach update DMA to the PWM driver.
    ///
    /// This enables you to use [`SimplePwm::waveform_up_dma()`].
    pub fn up_dma(&mut self, dma: impl Peripheral<P = impl UpDma<T>> + 'd) -> &mut Self {
        self.up_dma = Some(raw::up_dma(dma));
        self
    }

    /// Attach capture/compare DMA to the PWM driver.
    ///
    /// This enables you to use [`SimplePwm::waveform_cc_dma()`] with the given channel. You may
    /// use convenience methods [`ch1_cc_dma()`][Self::ch1_cc_dma()] to `ch4_cc_dma()`] to aid type
    /// inference.
    pub fn cc_dma<C: ChannelMarker>(&mut self, dma: impl Peripheral<P = impl CcDma<T, C>> + 'd) -> &mut Self {
        self.cc_dmas[C::CHANNEL.index()] = Some(raw::cc_dma(dma));
        self
    }
}

#[rustfmt::skip]
macro_rules! channel_impl {
    ($chx_pin:ident, $chx_cc_dma:ident, $channel:ident) => {
        impl<'d, T: CoreInstance> Builder<'d, T> {
            #[doc = concat!(
                "Attach an output pin for channel ",
                stringify!($channel),
                " to the PWM driver.\n\nSee [`pin()`][Self::pin()] for details.",
            )]
            pub fn $chx_pin(
                &mut self,
                pin: impl Peripheral<P = impl TimerPin<T, $channel>> + 'd,
                output_type: OutputType,
            ) -> &mut Self {
                self.pin::<$channel>(pin, output_type)
            }

            #[doc = concat!(
                "Attach capture/compare DMA for channel ",
                stringify!($channel),
                " to the PWM driver.\n\nSee [`cc_dma()`][Self::cc_dma()] for details.",
            )]
            pub fn $chx_cc_dma(&mut self, dma: impl Peripheral<P = impl CcDma<T, $channel>> + 'd) -> &mut Self {
                self.cc_dma::<$channel>(dma)
            }
        }
    };
}
channel_impl!(ch1_pin, ch1_cc_dma, Ch1);
channel_impl!(ch2_pin, ch2_cc_dma, Ch2);
channel_impl!(ch3_pin, ch3_cc_dma, Ch3);
channel_impl!(ch4_pin, ch4_cc_dma, Ch4);

impl<'d, T> Builder<'d, T>
where
    PeripheralRef<'d, T>: Peripheral<P = T> + 'd,
{
    /// Initialize the PWM driver for any timer peripheral with channels.
    ///
    /// PWM driver created using this method works with any timer peripheral, but it does not
    /// support generating PWM waveforms using DMA and does not support changing the [counting
    /// mode](CountingMode).
    pub fn build(self, freq: Hertz) -> SimplePwm<'d, General1ChTim>
    where
        T: General1ChInstance,
    {
        let raw = RawTimer::new_general_1ch(self.tim);
        SimplePwm::new_inner(raw, self.up_dma, self.channel_pins, self.cc_dmas, freq)
    }

    /// Initialize the PWM driver for a timer peripheral with DMA.
    ///
    /// Drivers created using this method support PWM waveforms using DMA.
    pub fn build_dma(self, freq: Hertz) -> SimplePwm<'d, CcDmaTim>
    where
        T: CcDmaInstance,
    {
        let raw = RawTimer::new_cc_dma(self.tim);
        SimplePwm::new_inner(raw, self.up_dma, self.channel_pins, self.cc_dmas, freq)
    }

    /// Initialize the PWM driver for a 4-channel timer peripheral.
    ///
    /// Drivers created using this method support all features. The driver starts with
    /// [`EdgeAlignedUp`](CountingMode::EdgeAlignedUp) counting mode, but you can call
    /// [`SimplePwm::set_frequency_counting_mode()`] to change it.
    pub fn build_4ch(self, freq: Hertz) -> SimplePwm<'d, General4ChTim>
    where
        T: General4ChInstance,
    {
        let raw = RawTimer::new_general_4ch(self.tim);
        SimplePwm::new_inner(raw, self.up_dma, self.channel_pins, self.cc_dmas, freq)
    }
}

/// Simple PWM driver.
///
/// Use [`Builder`] to build an instance of this driver.
pub struct SimplePwm<'d, Tim> {
    inner: Timer<'d, Tim>,
    up_dma: Option<dma::ChannelAndRequest<'d>>,
    channel_pins: [Option<RawTimerPin<'d>>; 4],
    cc_dmas: [Option<dma::ChannelAndRequest<'d>>; 4],
}

impl<'d, Tim: IsGeneral1ChTim> SimplePwm<'d, Tim> {
    fn new_inner(
        raw: RawTimer<'d, Tim>,
        up_dma: Option<dma::ChannelAndRequest<'d>>,
        channel_pins: [Option<RawTimerPin<'d>>; 4],
        cc_dmas: [Option<dma::ChannelAndRequest<'d>>; 4],
        freq: Hertz,
    ) -> Self {
        let mut this = Self {
            inner: Timer::new(raw),
            up_dma,
            channel_pins,
            cc_dmas,
        };

        this.set_frequency(freq);
        this.inner.enable_outputs(); // Required for advanced timers, see `RawTimer::enable_outputs()` for details
        this.inner.start();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .filter(|&ch| this.channel_pins[ch.index()].is_some())
            .for_each(|&ch| {
                this.inner.set_output_compare_mode(ch, OutputCompareMode::PwmMode1);
                this.inner.set_output_compare_preload(ch, true);
            });

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

    /// Set PWM frequency.
    ///
    /// When you call this, the max duty value changes, so you will have to call
    /// [`set_duty()`][Self::set_duty()] on all channels with the duty calculated based on the new
    /// max duty.
    pub fn set_frequency(&mut self, freq: Hertz) {
        self.inner.set_frequency(freq);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_duty(&self) -> u32 {
        self.inner.max_compare_value() + 1
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty`](Self::max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        assert!(self.channel_pins[channel.index()].is_some());
        assert!(duty <= self.max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    /// Get the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty()`](Self::max_duty()) for 100% duty, both included.
    pub fn duty(&self, channel: Channel) -> u32 {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.compare_value(channel)
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.set_output_polarity(channel, polarity);
    }

    /// Set the output compare mode for a given channel.
    pub fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
        assert!(self.channel_pins[channel.index()].is_some());
        self.inner.set_output_compare_mode(channel, mode);
    }
}

impl<'d, Tim: IsCcDmaTim> SimplePwm<'d, Tim> {
    /// Generate a sequence of PWM values (a waveform) using the update DMA.
    ///
    /// To use this method, you have to pass the update DMA to [`Builder::up_dma()`] and construct
    /// the driver using [`Builder::build_dma()`] or [`Builder::build_4ch()`] (to ensure that the
    /// underlying timer peripheral implements the update DMA).
    pub async fn waveform_up_dma(&mut self, channel: Channel, duty: &[u16]) {
        assert!(self.channel_pins[channel.index()].is_some());

        let original_duty_state = self.duty(channel);
        let original_enable_state = self.is_enabled(channel);
        let original_update_dma_state = self.inner.update_dma_state();

        if !original_update_dma_state {
            self.inner.enable_update_dma(true);
        }

        if !original_enable_state {
            self.enable(channel);
        }

        let up_dma = unwrap!(self.up_dma.as_mut());
        unsafe {
            let dma_transfer_options = dma::TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(dma::FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: dma::Burst::Incr8,
                ..Default::default()
            };

            up_dma
                .write(
                    duty,
                    self.inner.raw.ccr(channel.index()).as_ptr() as *mut u16,
                    dma_transfer_options,
                )
                .await
        };

        // restore output compare state
        if !original_enable_state {
            self.disable(channel);
        }

        self.set_duty(channel, original_duty_state);

        // Since DMA is closed before timer update event trigger DMA is turned off,
        // this can almost always trigger a DMA FIFO error.
        //
        // optional TODO:
        // clean FEIF after disable UDE
        if !original_update_dma_state {
            self.inner.enable_update_dma(false);
        }
    }

    /// Generate a sequence of PWM values (a waveform) using the capture/compare DMA for the
    /// channel.
    ///
    /// To use this method, you have to pass the capture/compare DMA for the channel to
    /// [`Builder::cc_dma()`] (or by using a convenience method such as [`Builder::ch1_cc_dma()`])
    /// and construct the driver using [`Builder::build_dma()`] or [`Builder::build_4ch()`] (to
    /// ensure that the underlying timer peripheral implements the capture/compare DMA).
    pub async fn waveform_cc_dma(&mut self, channel: Channel, duty: &[u16]) {
        use crate::pac::timer::vals::Ccds;
        assert!(self.channel_pins[channel.index()].is_some());

        let original_duty_state = self.duty(channel);
        let original_enable_state = self.is_enabled(channel);
        let original_cc_dma_on_update = self.inner.cc_dma_selection() == Ccds::ONUPDATE;
        let original_cc_dma_enabled = self.inner.cc_dma_enable_state(channel);

        // redirect CC DMA request onto Update Event
        if !original_cc_dma_on_update {
            self.inner.set_cc_dma_selection(Ccds::ONUPDATE)
        }

        if !original_cc_dma_enabled {
            self.inner.set_cc_dma_enable_state(channel, true);
        }

        if !original_enable_state {
            self.enable(channel);
        }

        let cc_dma = unwrap!(self.cc_dmas[channel.index()].as_mut());
        unsafe {
            let dma_transfer_option = dma::TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(dma::FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: dma::Burst::Incr8,
                ..Default::default()
            };

            cc_dma
                .write(
                    duty,
                    self.inner.raw.ccr(channel.index()).as_ptr() as *mut u16,
                    dma_transfer_option,
                )
                .await
        };

        // restore output compare state
        if !original_enable_state {
            self.disable(channel);
        }

        self.set_duty(channel, original_duty_state);

        // Since DMA is closed before timer Capture Compare Event trigger DMA is turn off,
        // this can almost always trigger a DMA FIFO error.
        //
        // optional TODO:
        // clean FEIF after disable UDE
        if !original_cc_dma_enabled {
            self.inner.set_cc_dma_enable_state(channel, false);
        }

        if !original_cc_dma_on_update {
            self.inner.set_cc_dma_selection(Ccds::ONCOMPARE)
        }
    }
}

impl<'d, Tim: IsGeneral4ChTim> SimplePwm<'d, Tim> {
    /// Set PWM frequency and counting mode.
    ///
    /// This method can only be used with [`General4ChInstance`] timer peripherals and you need to
    /// construct the [`SimplePwm`] driver using [`build_4ch()`][Builder::build_4ch()].
    ///
    /// When you call this, the max duty value changes, so you will have to call
    /// [`set_duty()`][Self::set_duty()] on all channels with the duty calculated based on the new
    /// max duty.
    pub fn set_frequency_counting_mode(&mut self, freq: Hertz, counting_mode: CountingMode) {
        let multiplier = match counting_mode.is_center_aligned() {
            true => 2u8,
            false => 1u8,
        };
        self.inner.set_frequency(freq * multiplier);
        self.inner.set_counting_mode(counting_mode);
    }
}

impl<'d, Tim: IsGeneral1ChTim> embedded_hal_02::Pwm for SimplePwm<'d, Tim> {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u32;

    fn disable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, false);
    }

    fn enable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, true);
    }

    fn get_period(&self) -> Self::Time {
        // TODO: we return frequency instead of period here??
        self.inner.frequency()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.inner.compare_value(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.inner.max_compare_value() + 1
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into());
    }
}
