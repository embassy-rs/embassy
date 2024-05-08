use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use rp_pac::pwm::vals::Divmode;

use super::{BuilderState, ChannelConfig, ConfigureDMA, ConfigureDivider, ConfigurePhaseCorrect, DivMode, PwmBuilder, SliceConfig};
use crate::builder_state;
use crate::gpio::{SealedPin, AnyPin};
use crate::pwm::v2::{EdgeSensitivity, PwmError, PwmInputOutputSlice};
use crate::pwm::{ChannelAPin, ChannelBPin, Slice};

builder_state!(LevelOrEdgeSensitive);
builder_state!(LevelOrEdgeSensitiveInput);
builder_state!(LevelOrEdgeSensitiveOutput);
builder_state!(LevelOrEdgeSensitiveInputOutput);

/// Configuration object for a PWM channel within a free-running slice.
pub struct LevelEdgeSensitiveChannelConfig {
    invert: bool,
}

impl Default for LevelEdgeSensitiveChannelConfig {
    fn default() -> Self {
        LevelEdgeSensitiveChannelConfig {
            invert: false,
        }
    }
}

impl LevelEdgeSensitiveChannelConfig {
    /// Sets whether or not the PWM signal for this channel should be
    /// inverted. Defaults to `false`.
    pub fn invert(&mut self, invert: bool) -> &mut Self {
        self.invert = invert;
        self
    }
}

impl Into<ChannelConfig> for LevelEdgeSensitiveChannelConfig {
    fn into(self) -> ChannelConfig {
        ChannelConfig {
            invert: self.invert,
            ..ChannelConfig::default()
        }
    }
}

impl PwmBuilder<DivMode> {
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
    pub fn level_sensitive(&self) -> PwmBuilder<LevelOrEdgeSensitive> {
        PwmBuilder {
            config: SliceConfig {
                div_mode: Divmode::LEVEL,
                ..SliceConfig::default()
            },
            _phantom: PhantomData,
        }
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
    pub fn edge_sensitive(&self, edge: EdgeSensitivity) -> PwmBuilder<LevelOrEdgeSensitive> {
        match edge {
            EdgeSensitivity::Rising => PwmBuilder {
                config: SliceConfig {
                    div_mode: Divmode::RISE,
                    ..SliceConfig::default()
                },
                _phantom: PhantomData,
            },
            EdgeSensitivity::Falling => PwmBuilder {
                config: SliceConfig {
                    div_mode: Divmode::FALL,
                    ..SliceConfig::default()
                },
                _phantom: PhantomData,
            },
        }
    }
}

impl PwmBuilder<LevelOrEdgeSensitive> {
    /// Configure the input pin for this slice.
    pub fn with_input(
        self,
        input: impl FnOnce(&mut LevelEdgeSensitiveChannelConfig) -> &mut LevelEdgeSensitiveChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInput> {
        let mut config = self.get_config_owned();
        let mut channel_config = LevelEdgeSensitiveChannelConfig::default();
        input(&mut channel_config);
        config.a = Some(channel_config.into());
        PwmBuilder::new(config)
    }

    /// Configure this slice for input using defaults.
    pub fn with_input_defaults(self) -> PwmBuilder<LevelOrEdgeSensitiveInput> {
        self.with_input(|input| input)
    }

    /// Assign the specified pin as an output for this slice. The pin must
    /// be a valid PWM output pin for the slice (only A-pins are supported
    /// as output pins in level- and edge-sensitive slice modes).
    pub fn with_output(
        self,
        pin: impl FnOnce(&mut LevelEdgeSensitiveChannelConfig) -> &mut LevelEdgeSensitiveChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = LevelEdgeSensitiveChannelConfig::default();
        pin(&mut channel_config);
        config.b = Some(channel_config.into());
        PwmBuilder::new(config)
    }

    /// Configure this slice for output using defaults.
    pub fn with_output_defaults(self) -> PwmBuilder<LevelOrEdgeSensitiveOutput> {
        self.with_output(|output| output)
    }
}

impl PwmBuilder<LevelOrEdgeSensitiveInput> {
    /// Assign the specified pin as an output for this slice. The pin must
    /// be a valid PWM output pin for the slice (only A-pins are supported
    /// as output pins in level- and edge-sensitive slice modes).
    pub fn with_output(
        self,
        pin: impl FnOnce(&mut LevelEdgeSensitiveChannelConfig) -> &mut LevelEdgeSensitiveChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = LevelEdgeSensitiveChannelConfig::default();
        pin(&mut channel_config);
        config.b = Some(channel_config.into());
        PwmBuilder::new(config)
    }

    /// Configure this slice for output using defaults.
    pub fn with_output_defaults(self) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        self.with_output(|output| output)
    }

    /// Apply the configuration to the provided slice and GPIO pin.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmInputOutputSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`] function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> Result<PwmInputOutputSlice<'a, S>, PwmError> {
        into_ref!(pin_b);
        PwmInputOutputSlice::new_from_config(self.get_config_owned(), slice, Some(pin_b.map_into()), None::<AnyPin>)
    }
}

impl PwmBuilder<LevelOrEdgeSensitiveOutput> {
    /// Assign the specified pin as an input for this slice. The pin must be
    /// a valid PWM input pin for the slice (only B-pins are supported as
    /// input pins in level- and edge-sensitive slice modes).
    pub fn with_input(
        self,
        input: impl FnOnce(&mut LevelEdgeSensitiveChannelConfig) -> &mut LevelEdgeSensitiveChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = LevelEdgeSensitiveChannelConfig::default();
        input(&mut channel_config);
        config.a = Some(channel_config.into());
        PwmBuilder::new(config)
    }

    /// Configure this slice for input using defaults.
    pub fn with_input_defaults(self) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        self.with_input(|input| input)
    }

    /// Apply the configuration to the provided slice and GPIO pin.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmInputOutputSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`] function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> Result<PwmInputOutputSlice<'a, S>, PwmError> {
        into_ref!(pin_b);
        PwmInputOutputSlice::new_from_config(self.get_config_owned(), slice, None::<AnyPin>, Some(pin_b.map_into()))
    }
}

impl PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
    /// Apply the configuration to the provided slice and GPIO pins.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmInputOutputSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`] function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_a: impl Peripheral<P = impl ChannelAPin<S>> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> Result<PwmInputOutputSlice<'a, S>, PwmError> {
        into_ref!(pin_a);
        into_ref!(pin_b);
        PwmInputOutputSlice::new_from_config(
            self.get_config_owned(),
            slice,
            Some(pin_a.map_into()),
            Some(pin_b.map_into()),
        )
    }
}

impl ConfigureDMA for PwmBuilder<LevelOrEdgeSensitiveInput> {}
impl ConfigureDMA for PwmBuilder<LevelOrEdgeSensitiveOutput> {}
impl ConfigureDMA for PwmBuilder<LevelOrEdgeSensitiveInputOutput> {}

impl ConfigurePhaseCorrect for PwmBuilder<LevelOrEdgeSensitive> {}
impl ConfigurePhaseCorrect for PwmBuilder<LevelOrEdgeSensitiveInput> {}
impl ConfigurePhaseCorrect for PwmBuilder<LevelOrEdgeSensitiveOutput> {}
impl ConfigurePhaseCorrect for PwmBuilder<LevelOrEdgeSensitiveInputOutput> {}

impl ConfigureDivider for PwmBuilder<LevelOrEdgeSensitive> {}
impl ConfigureDivider for PwmBuilder<LevelOrEdgeSensitiveInput> {}
impl ConfigureDivider for PwmBuilder<LevelOrEdgeSensitiveOutput> {}
impl ConfigureDivider for PwmBuilder<LevelOrEdgeSensitiveInputOutput> {}

#[allow(unused_variables)] // TODO: Temporary, to be implemented
impl<'a, S: Slice> PwmInputOutputSlice<'a, S> {
    fn new_from_config(
        config: SliceConfig,
        slice: impl Peripheral<P = S> + 'static,
        pin_a: Option<impl Peripheral<P = AnyPin> + 'static>,
        pin_b: Option<impl Peripheral<P = AnyPin> + 'static>,
    ) -> Result<Self, PwmError> {
        // We shouldn't be able to get here, but we'll check it anyway.
        if config.a.is_none() && config.b.is_none() {
            return Err(PwmError::Configuration("At least one channel must be configured"));
        }

        // Get a reference to the slice peripheral.
        into_ref!(slice);

        // Get an instance of the registers for this slice.
        let regs = slice.regs();

        regs.csr().write(|w| {
            match config.div_mode {
                Divmode::DIV => w.set_divmode(Divmode::DIV),
                Divmode::LEVEL => w.set_divmode(Divmode::LEVEL),
                Divmode::RISE => w.set_divmode(Divmode::RISE),
                Divmode::FALL => w.set_divmode(Divmode::FALL),
            }
            w.set_en(false);
            w.set_ph_correct(config.phase_correct);
        });

        let pwm_slice = PwmInputOutputSlice::new(
            slice,
            pin_a.map(|b| b.into_ref().map_into()),
            pin_b.map(|b| b.into_ref().map_into()),
        );

        // If channel A is configured, set the pin function to PWM.
        if let Some(ref a) = pwm_slice.pin_a {
            let a_conf = config.a.as_ref().ok_or_else(|| {
                PwmError::Configuration("Channel A must be configured if and only if pin A is provided")
            })?;

            into_ref!(a);
            debug!("Setting A pin function to PWM");
            a.gpio().ctrl().write(|w| w.set_funcsel(4));
            regs.csr().modify(|w| w.set_b_inv(a_conf.invert));
        }

        // If channel B is configured, set the pin function to PWM.
        if let Some(ref b) = pwm_slice.pin_b {
            let b_conf = config.b.as_ref().ok_or_else(|| {
                PwmError::Configuration("Channel B must be configured if and only if pin B is provided")
            })?;

            into_ref!(b);
            debug!("Setting B pin function to PWM");
            b.gpio().ctrl().write(|w| w.set_funcsel(4));
            regs.csr().modify(|w| w.set_b_inv(b_conf.invert));
        }

        Ok(pwm_slice)
    }
}
