use embassy_hal_internal::{into_ref, Peripheral};
use rp_pac::pwm::vals::Divmode;

use super::v2::{AsPwmSlice, Channel, EdgeSensitivity, Frequency, PwmInputOutputSlice};
use super::{ChannelAPin, ChannelBPin, InputMode, Slice};
use crate::clocks::clk_sys_freq;
use crate::gpio::SealedPin;

trait GetPwmSliceConfig {
    fn get_config(&mut self) -> &mut PwmSliceConfig;
}

struct PwmSliceConfig {
    top: u32,
    div: u32,
    a: Option<PwmChannelConfig>,
    b: Option<PwmChannelConfig>,
}

impl Default for PwmSliceConfig {
    fn default() -> Self {
        Self {
            top: u16::MAX as u32,
            div: 1,
            a: None,
            b: None,
        }
    }
}

struct PwmChannelConfig {
    duty: f32,
    phase_correct: bool,
    invert: bool,
}

impl Default for PwmChannelConfig {
    fn default() -> Self {
        Self {
            duty: 0.0,
            phase_correct: false,
            invert: false,
        }
    }
}

/// TODO
pub struct PwmSliceBuilder {
    config: PwmSliceConfig,
}

impl PwmSliceBuilder {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
        }
    }

    /// Returns a builder for configuring a free-running PWM slice. A
    /// free-running slice runs the counter continuously at a rate dictated
    /// by the fractional divider.
    ///
    /// Free-running slices use both A and B channels as outputs, where each
    /// channel can be configured with a duty cycle, phase-correct mode, and
    /// inversion. The frequency of each channel will be the slice's
    /// frequency / 2.
    ///
    /// By default, the counter wraps back to `0` when the counter reaches
    /// the value set in the `top` register. This behavior can be changed
    /// by setting `phase_correct` to `true` in the channel configuration.
    pub fn free_running(self) -> PwmFreeRunningSliceBuilder {
        PwmFreeRunningSliceBuilder::new(self.config)
    }

    /// Returns a builder for configuring a level-sensitive PWM slice. A
    /// level-sensitive slice runs the counter continuously at a rate dictated
    /// by the fractional divider _when a high-level is detected on the `B`
    /// channel input pin_.
    ///
    /// By allowing the slice to run for a fixed amount of time in
    /// edge-sensitive mode, it's possible to measure the duty cycle or
    /// frequency of an input signal.
    ///
    /// The clock divider is still operational in level-sensitive mode. At
    /// maximum division (`255`), the counter will only advance once per 256
    /// _high_ input cycles.
    ///
    /// The default divider is `1` and can be changed by using the
    /// [`PwmInputOutputSliceBuilder::divider`] method.
    pub fn level_sensitive(self) -> PwmInputOutputSliceBuilder {
        PwmInputOutputSliceBuilder::new(self.config, InputMode::Level)
    }

    /// Returns a builder for configuring an edge-sensitive PWM slice. An
    /// edge-sensitive slice will increment the counter once per detected
    /// edge on the `B` channel input pin, either on the
    /// [`Rising`](EdgeSensitivity::Rising) or
    /// [`Falling`](EdgeSensitivity::Falling) edge.
    ///
    /// By allowing the slice to run for a fixed amount of time in
    /// edge-sensitive mode, it's possible to measure the duty cycle or
    /// frequency of an input signal.
    ///
    /// The clock divider is still operational in edge-sensitive mode. At
    /// maximum division (`255`), the counter will only advance once per 256
    /// edges.
    ///
    /// Note that due to the type of edge-detect circuit used in the RP2040,
    /// the _low_ period and _high_ period of the measured signal must both
    /// be strictly greater than the system clock period when taking
    /// frequency measurements.
    pub fn edge_sensitive(self, edge: EdgeSensitivity) -> PwmInputOutputSliceBuilder {
        match edge {
            EdgeSensitivity::Rising => PwmInputOutputSliceBuilder::new(self.config, InputMode::RisingEdge),
            EdgeSensitivity::Falling => PwmInputOutputSliceBuilder::new(self.config, InputMode::FallingEdge),
        }
    }
}

/// Builder for configuring a PWM slice in either level- or edge-sensitive
/// mode.
pub struct PwmInputOutputSliceBuilder {
    config: PwmSliceConfig,
    divider_int: u8,
    divider_frac: u8,
    input_mode: InputMode,
    phase_correct: bool,
}

impl GetPwmSliceConfig for PwmInputOutputSliceBuilder {
    fn get_config(&mut self) -> &mut PwmSliceConfig {
        &mut self.config
    }
}

impl PwmInputOutputSliceBuilder {
    fn new(config: PwmSliceConfig, input_mode: InputMode) -> Self {
        Self {
            config,
            divider_int: 1,
            divider_frac: 0,
            input_mode,
            phase_correct: false,
        }
    }

    /// Sets the fractional divider for this slice. The divider is a 16-bit
    /// fixed-point number with 4 fractional bits. The divider controls the
    /// rate at which the counter increments.
    pub fn divider(mut self, int: u8, frac: u8) -> Self {
        self.divider_int = int;
        self.divider_frac = frac;
        self
    }

    /// Assign the specified pin as an output for this slice. The pin must
    /// be a valid PWM output pin for the slice (only A-pins are supported
    /// as output pins in level- and edge-sensitive slice modes).
    pub fn with_output(mut self) -> Self {
        self.config.a = Some(Default::default());
        self
    }

    /// Assign the specified pin as an input for this slice. The pin must be
    /// a valid PWM input pin for the slice (only B-pins are supported as
    /// input pins in level- and edge-sensitive slice modes).
    pub fn with_input(mut self) -> Self {
        self.config.b = Some(Default::default());
        self
    }

    /// Sets whether or not phase-correct modulation should be enabled
    /// for this channel.
    ///
    /// Defaults to `false` (disabled).
    ///
    /// ### When enabled
    /// When phase-correct mode is enabled the channel's counter will
    /// oscillate between 0 and the value set in the `top` register (which
    /// is calculated). This results in a PWM signal that is centered
    /// regardless of the duty cycle. Changes to the duty cycle will be
    /// reflected in the output signal after the next 0-to-0 transition
    /// of the channel's counter.
    ///
    /// ### When disabled (default)
    /// When phase-correct mode is disabled the channel's counter will
    /// wrap back to 0 when it reaches the value set in the `top` register
    /// (which is calculated). Changes to the duty cycle will be reflected
    /// in the output signal when the channel's counter wraps, which occurs
    /// every `TOP` + 1 cycles.
    pub fn phase_correct(mut self, phase_correct: bool) -> Self {
        self.phase_correct = phase_correct;
        self
    }

    /// Applies the configuration of this builder to the PWM slice and GPIO
    /// pins. This method will return a configured PWM slice that can be
    /// enabled to start sampling/generating PWM signals.
    ///
    /// Note that this will not enable the slice, only configure it. You must
    /// call [`PwmInputOutputSlice::enable`], or alternatively use the
    /// [`enable_pwm_slices`] function, to start the slice.
    pub fn apply<S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_a: impl Peripheral<P = impl ChannelAPin<S>> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> PwmInputOutputSlice<'static, S> {
        into_ref!(slice);
        into_ref!(pin_a);
        into_ref!(pin_b);

        // B-pin is required for controlling the counter.
        if self.config.b.is_none() {
            panic!("B-pin must be set for input mode");
        }

        let regs = slice.regs();

        let slice = PwmInputOutputSlice::new(slice, Some(pin_a.map_into()), Some(pin_b.map_into()));

        // Set the fractional divider (CH0_DIV..CH7_DIV).
        regs.div().write(|w| {
            w.set_int(self.divider_int);
            w.set_frac(self.divider_frac);
        });

        // Set the control and status register values.
        // (CH0_CSR..CH7_CSR).
        regs.csr().write(|w| {
            match &self.input_mode {
                InputMode::Level => w.set_divmode(Divmode::LEVEL),
                InputMode::RisingEdge => w.set_divmode(Divmode::RISE),
                InputMode::FallingEdge => w.set_divmode(Divmode::FALL),
            };

            w.set_en(false);
            w.set_ph_correct(self.phase_correct);
        });

        if let Some(pin) = &slice.pin_a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
        }

        if let Some(pin) = &slice.pin_b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
        }

        slice
    }
}

/// TODO
pub struct PwmFreeRunningSliceBuilder {
    config: PwmSliceConfig,
    frequency_hz: u32,
    phase_correct: bool,
}

impl GetPwmSliceConfig for PwmFreeRunningSliceBuilder {
    fn get_config(&mut self) -> &mut PwmSliceConfig {
        &mut self.config
    }
}

impl PwmFreeRunningSliceBuilder {
    fn new(config: PwmSliceConfig) -> Self {
        Self {
            config,
            frequency_hz: clk_sys_freq(),
            phase_correct: false,
        }
    }

    /// Sets the frequency for this PWM slice. The frequency can be set in
    /// Hz, KHz, or MHz. The frequency must be between 8 Hz and the system
    /// clock frequency. The effective per-channel frequency will be this
    /// value / 2.
    pub fn frequency(mut self, freq: Frequency) -> Self {
        let freq_hz = match freq {
            Frequency::Hz(hz) => hz,
            Frequency::KHz(khz) => (khz * 1_000.0) as u32,
            Frequency::MHz(mhz) => (mhz * 1_000_000.0) as u32,
        };

        if freq_hz > clk_sys_freq() {
            panic!("Frequency must be less than the system clock frequency");
        }

        self.frequency_hz = freq_hz;
        self
    }

    /// TODO
    pub fn with_channel_a<'a>(
        self,
        cfg: impl FnOnce(&mut PwmFreeRunningChannelBuilder) -> &'a mut PwmFreeRunningChannelBuilder<'a>,
    ) -> Self {
        {
            let mut builder = PwmFreeRunningChannelBuilder::new(&self.config, Channel::A);
            cfg(&mut builder);
        }
        self
    }

    /// TODO
    pub fn with_channel_b<'a>(
        self,
        cfg: impl FnOnce(&mut PwmFreeRunningChannelBuilder) -> &'a mut PwmFreeRunningChannelBuilder<'a>,
    ) -> Self {
        {
            let mut builder = PwmFreeRunningChannelBuilder::new(&self.config, Channel::B);
            cfg(&mut builder);
        }
        self
    }

    /// Sets whether or not phase-correct modulation should be enabled
    /// for this channel.
    ///
    /// Defaults to `false` (disabled).
    ///
    /// ### When enabled
    /// When phase-correct mode is enabled the channel's counter will
    /// oscillate between 0 and the value set in the `top` register (which
    /// is calculated). This results in a PWM signal that is centered
    /// regardless of the duty cycle. Changes to the duty cycle will be
    /// reflected in the output signal after the next 0-to-0 transition
    /// of the channel's counter.
    ///
    /// ### When disabled (default)
    /// When phase-correct mode is disabled the channel's counter will
    /// wrap back to 0 when it reaches the value set in the `top` register
    /// (which is calculated). Changes to the duty cycle will be reflected
    /// in the output signal when the channel's counter wraps, which occurs
    /// every `TOP` + 1 cycles.
    pub fn phase_correct(mut self, phase_correct: bool) -> Self {
        self.phase_correct = phase_correct;
        self
    }

    // /// Applies the configuration of this builder to the PWM slice and
    // /// GPIO pins. This method will return a configured PWM slice that can
    // /// be enabled to start generating PWM signals.
    // ///
    // /// Note that this will not enable the slice, only configure it. You must
    // /// call [`PwmInputOutputSlice::enable`], or alternatively use the
    // /// [`enable_pwm_slices`] function, to start the slice.
    // pub fn apply(mut self) -> Result<PwmFreeRunningSlice<'a>, PwmError> {
    //     // Require that at least one of A or B is configured.
    //     if self.builder_a.is_none() && self.builder_b.is_none() {
    //         panic!("At least one channel must be configured");
    //     }

    //     // Get an instance of the registers for this slice.
    //     let regs = self.inner.regs();

    //     // Grab our pins.
    //     let pin_a = self.builder_a.take();
    //     let pin_b = self.builder_b.take();

    //     regs.csr().write(|w| {
    //         w.set_divmode(Divmode::DIV);
    //         w.set_en(false);
    //         w.set_ph_correct(self.phase_correct);
    //     });

    //     // If channel A is configured, set the pin function to PWM.
    //     if let Some(a) = &pin_a {
    //         debug!("Setting A pin function to PWM");
    //         a.pin.gpio().ctrl().write(|w| w.set_funcsel(4));
    //         regs.csr().modify(|w| w.set_a_inv(a.invert));
    //     }

    //     // If channel B is configured, set the pin function to PWM.
    //     if let Some(b) = &pin_b {
    //         debug!("Setting B pin function to PWM");
    //         b.pin.gpio().ctrl().write(|w| w.set_funcsel(4));
    //         regs.csr().modify(|w| w.set_b_inv(b.invert));
    //     }

    //     let pin_a_duty = pin_a.as_ref().map(|a| a.duty_percent);
    //     let pin_b_duty = pin_b.as_ref().map(|b| b.duty_percent);

    //     // Create our PWM slice instance.
    //     let mut slice: PwmFreeRunningSlice<'_, T> =
    //         PwmFreeRunningSlice::new(self.inner, pin_a.map(|b| b.pin), pin_b.map(|b| b.pin));

    //     // If channel A is configured, configure PWM for the A pin.
    //     if let Some(duty) = pin_a_duty {
    //         slice.reconfigure(Channel::A, self.frequency_hz, duty, self.phase_correct)?;
    //     }

    //     // If channel B is configured, configure PWM for the B pin.
    //     if let Some(duty) = pin_b_duty {
    //         slice.reconfigure(Channel::B, self.frequency_hz, duty, self.phase_correct)?;
    //     }

    //     Ok(slice)
    // }
}

// /// Builder for configuring the PWM input channel (`B`) for a level- or
// /// edge-sensitive slice.
// pub struct PwmInputChannelBuilder<'a, T: Slice> {
//     _slice: PhantomData<T>,
//     pin: PeripheralRef<'a, AnyPin>,
// }

// impl<'a, T: Slice> PwmInputChannelBuilder<'a, T> {
//     pub(crate) fn new(pin: PeripheralRef<'a, AnyPin>) -> Self {
//         Self {
//             _slice: PhantomData,
//             pin,
//         }
//     }
// }

// /// Builder for configuring a PWM output channel (`A`) in a level- or
// /// edge-sensitive slice.
// pub struct PwmOutputChannelBuilder<'a, T: Slice> {
//     _slice: PhantomData<T>,
//     pin: PeripheralRef<'a, AnyPin>,
// }

// impl<'a, T: Slice> PwmOutputChannelBuilder<'a, T> {
//     pub(crate) fn new(pin: PeripheralRef<'a, AnyPin>) -> Self {
//         Self {
//             _slice: PhantomData,
//             pin,
//         }
//     }
// }

/// Builder for configuring a PWM channel (`A` or `B`) in a free-running
/// slice.
pub struct PwmFreeRunningChannelBuilder<'a> {
    config: &'a PwmSliceConfig,
    channel: Channel,
    duty_percent: f32,
    invert: bool,
}

impl<'a> PwmFreeRunningChannelBuilder<'a> {
    fn new(config: &'a PwmSliceConfig, channel: Channel) -> Self {
        Self {
            config,
            channel,
            duty_percent: 0.0,
            invert: false,
        }
    }

    /// Sets the duty cycle for this channel. The duty cycle is a percentage
    /// value between 0.0 and 100.0. Defaults to 0.0.
    pub fn duty_cycle(mut self, duty_percent: f32) -> Self {
        self.duty_percent = duty_percent;
        self
    }

    /// Sets whether or not the PWM signal for this channel should be
    /// inverted. Defaults to `false`.
    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }
}
