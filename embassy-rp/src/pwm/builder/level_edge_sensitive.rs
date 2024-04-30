use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use rp_pac::pwm::vals::Divmode;

use super::{BuilderState, ChannelConfig, ConfigureDivider, ConfigurePhaseCorrect, DivMode, PwmBuilder, SliceConfig};
use crate::builder_state;
use crate::gpio::AnyPin;
use crate::pwm::v2::{EdgeSensitivity, PwmError, PwmInputOutputSlice};
use crate::pwm::{ChannelAPin, ChannelBPin, Slice};

builder_state!(LevelOrEdgeSensitive);
builder_state!(LevelOrEdgeSensitiveInput);
builder_state!(LevelOrEdgeSensitiveOutput);
builder_state!(LevelOrEdgeSensitiveInputOutput);

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
    pub fn level_sensitive() -> PwmBuilder<LevelOrEdgeSensitive> {
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
    pub fn edge_sensitive(edge: EdgeSensitivity) -> PwmBuilder<LevelOrEdgeSensitive> {
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
    /// Assign the specified pin as an input for this slice. The pin must be
    /// a valid PWM input pin for the slice (only B-pins are supported as
    /// input pins in level- and edge-sensitive slice modes).
    pub fn with_input(
        self,
        input: impl FnOnce(&mut ChannelConfig) -> &mut ChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInput> {
        let mut config = self.get_config_owned();
        let mut channel_config = ChannelConfig::default();
        input(&mut channel_config);
        config.a = Some(channel_config);
        PwmBuilder::new(config)
    }

    /// Assign the specified pin as an output for this slice. The pin must
    /// be a valid PWM output pin for the slice (only A-pins are supported
    /// as output pins in level- and edge-sensitive slice modes).
    pub fn with_output(
        self,
        pin: impl FnOnce(&mut ChannelConfig) -> &mut ChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = ChannelConfig::default();
        pin(&mut channel_config);
        config.b = Some(channel_config);
        PwmBuilder::new(config)
    }
}

impl PwmBuilder<LevelOrEdgeSensitiveInput> {
    /// Assign the specified pin as an output for this slice. The pin must
    /// be a valid PWM output pin for the slice (only A-pins are supported
    /// as output pins in level- and edge-sensitive slice modes).
    pub fn with_output(
        self,
        pin: impl FnOnce(&mut ChannelConfig) -> &mut ChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = ChannelConfig::default();
        pin(&mut channel_config);
        config.b = Some(channel_config);
        PwmBuilder::new(config)
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
        pin_b: impl Peripheral<P = impl ChannelAPin<S>> + 'static,
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
        input: impl FnOnce(&mut ChannelConfig) -> &mut ChannelConfig,
    ) -> PwmBuilder<LevelOrEdgeSensitiveInputOutput> {
        let mut config = self.get_config_owned();
        let mut channel_config = ChannelConfig::default();
        input(&mut channel_config);
        config.a = Some(channel_config);
        PwmBuilder::new(config)
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
        todo!()
    }
}
