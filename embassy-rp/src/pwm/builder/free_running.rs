use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use rp_pac::pwm::vals::Divmode;

use super::{BuilderState, ChannelConfig, ConfigureFrequency, ConfigurePhaseCorrect, DivMode, PwmBuilder, SliceConfig};
use crate::builder_state;
use crate::gpio::{AnyPin, SealedPin};
use crate::pwm::v2::{Channel, PwmError, PwmFreeRunningSlice, UncheckedPwmFreeRunningSlice};
use crate::pwm::{ChannelAPin, ChannelBPin, Slice};

builder_state!(FreeRunning);
builder_state!(FreeRunningChannels);
builder_state!(FreeRunningChannel);
builder_state!(FreeRunningChannelA);
builder_state!(FreeRunningChannelB);
builder_state!(FreeRunningChannelAB);

impl ConfigurePhaseCorrect for PwmBuilder<FreeRunning> {}
impl ConfigurePhaseCorrect for PwmBuilder<FreeRunningChannelA> {}
impl ConfigurePhaseCorrect for PwmBuilder<FreeRunningChannelB> {}
impl ConfigurePhaseCorrect for PwmBuilder<FreeRunningChannelAB> {}

impl ConfigureFrequency for PwmBuilder<FreeRunning> {}
impl ConfigureFrequency for PwmBuilder<FreeRunningChannelA> {}
impl ConfigureFrequency for PwmBuilder<FreeRunningChannelB> {}
impl ConfigureFrequency for PwmBuilder<FreeRunningChannelAB> {}

/// Configuration object for a PWM channel within a free-running slice.
pub struct FreeRunningChannelConfig {
    duty_percent: f32,
    invert: bool,
}

impl Default for FreeRunningChannelConfig {
    fn default() -> Self {
        FreeRunningChannelConfig {
            duty_percent: 0.0,
            invert: false,
        }
    }
}

impl FreeRunningChannelConfig {
    /// Sets the duty cycle for this channel. The duty cycle is a percentage
    /// value between 0.0 and 100.0. Defaults to 0.0.
    pub fn duty_cycle(&mut self, duty_percent: f32) -> &mut Self {
        self.duty_percent = duty_percent;
        self
    }

    /// Sets whether or not the PWM signal for this channel should be
    /// inverted. Defaults to `false`.
    pub fn invert(&mut self, invert: bool) -> &mut Self {
        self.invert = invert;
        self
    }
}

impl Into<ChannelConfig> for FreeRunningChannelConfig {
    fn into(self) -> ChannelConfig {
        ChannelConfig {
            duty_percent: self.duty_percent,
            invert: self.invert,
        }
    }
}

impl PwmBuilder<DivMode> {
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
    pub fn free_running(&self) -> PwmBuilder<FreeRunning> {
        PwmBuilder {
            config: SliceConfig {
                div_mode: Divmode::DIV,
                ..SliceConfig::default()
            },
            _phantom: PhantomData,
        }
    }
}

impl PwmBuilder<FreeRunning> {
    /// Configures channel A for this slice.
    pub fn with_output_a(
        mut self,
        a: impl FnOnce(&mut FreeRunningChannelConfig) -> &mut FreeRunningChannelConfig,
    ) -> PwmBuilder<FreeRunningChannelA> {
        let mut channel_config = FreeRunningChannelConfig::default();
        a(&mut channel_config);
        self.get_config().a = Some(channel_config.into());
        PwmBuilder::new(self.get_config_owned())
    }

    /// Configures channel B for this slice.
    pub fn with_output_b(
        mut self,
        b: impl FnOnce(&mut FreeRunningChannelConfig) -> &mut FreeRunningChannelConfig,
    ) -> PwmBuilder<FreeRunningChannelB> {
        let mut channel_config = FreeRunningChannelConfig::default();
        b(&mut channel_config);
        self.get_config().b = Some(channel_config.into());
        PwmBuilder::new(self.get_config_owned())
    }
}

impl PwmBuilder<FreeRunningChannelA> {
    /// Configures channel B for this slice.
    pub fn with_output_b(
        mut self,
        b: impl FnOnce(&mut FreeRunningChannelConfig) -> &mut FreeRunningChannelConfig,
    ) -> PwmBuilder<FreeRunningChannelAB> {
        let mut channel_config = FreeRunningChannelConfig::default();
        b(&mut channel_config);
        self.config.b = Some(channel_config.into());
        PwmBuilder::new(self.config)
    }

    /// Apply the configuration to the provided slice and GPIO pin.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmFreeRunningSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`] function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_a: impl Peripheral<P = impl ChannelAPin<S>> + 'static,
    ) -> Result<PwmFreeRunningSlice<'a, S>, PwmError> {
        into_ref!(pin_a);
        PwmFreeRunningSlice::new_from_config(self.get_config_owned(), slice, Some(pin_a.map_into()), None::<AnyPin>)
    }
}

impl PwmBuilder<FreeRunningChannelB> {
    /// Configures channel A for this slice.
    pub fn with_ouput_a(
        mut self,
        a: impl FnOnce(&mut FreeRunningChannelConfig) -> &mut FreeRunningChannelConfig,
    ) -> PwmBuilder<FreeRunningChannelAB> {
        let mut channel_config = FreeRunningChannelConfig::default();
        a(&mut channel_config);
        self.config.a = Some(channel_config.into());
        PwmBuilder::new(self.config)
    }

    /// Apply the configuration to the provided slice and GPIO pin.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmFreeRunningSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`] function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> Result<PwmFreeRunningSlice<'a, S>, PwmError> {
        into_ref!(pin_b);
        PwmFreeRunningSlice::new_from_config(self.get_config_owned(), slice, None::<AnyPin>, Some(pin_b.map_into()))
    }
}

impl PwmBuilder<FreeRunningChannelAB> {
    /// Apply the configuration to the provided slice and GPIO pins.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`PwmFreeRunningSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`](crate::pwm::v2::enable_pwm_slices) function to enable multiple
    /// slices simultaneously.
    pub fn apply<'a, S: Slice>(
        self,
        slice: impl Peripheral<P = S> + 'static,
        pin_a: impl Peripheral<P = impl ChannelAPin<S>> + 'static,
        pin_b: impl Peripheral<P = impl ChannelBPin<S>> + 'static,
    ) -> Result<PwmFreeRunningSlice<'a, S>, PwmError> {
        into_ref!(pin_a);
        into_ref!(pin_b);
        PwmFreeRunningSlice::new_from_config(
            self.get_config_owned(),
            slice,
            Some(pin_a.map_into()),
            Some(pin_b.map_into()),
        )
    }

    /// **Advanced Method**
    ///
    /// Apply the configuration to the provided slice and GPIO pins without
    /// checking if the pins are compatible with the slice. This can easily
    /// result in an incorrect configuration, so use with caution.
    ///
    /// Note that this will not automatically enable the PWM slice. You must
    /// call [`UncheckedPwmFreeRunningSlice::enable`] to start the PWM output, or
    /// alternatively use the [`enable_pwm_slices`](crate::pwm::v2::enable_pwm_slices) function to enable multiple
    /// slices simultaneously.
    #[allow(unused)]
    pub fn apply_unchecked(
        self,
        slice_number: usize,
        pin_a_number: usize,
        pin_b_number: usize,
    ) -> Result<UncheckedPwmFreeRunningSlice, PwmError> {
        let slice = rp_pac::PWM.ch(slice_number);
        let pin_a = rp_pac::IO_BANK0.gpio(pin_a_number);
        let pin_b = rp_pac::IO_BANK0.gpio(pin_b_number);

        todo!()
    }
}

impl<'a, S: Slice> PwmFreeRunningSlice<'a, S> {
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
            w.set_divmode(Divmode::DIV);
            w.set_en(false);
            w.set_ph_correct(config.phase_correct);
        });

        let mut pwm_slice = PwmFreeRunningSlice::new(
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
            pwm_slice.reconfigure(
                Channel::A,
                config.frequency_hz,
                a_conf.duty_percent,
                config.phase_correct,
            )?;
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
            pwm_slice.reconfigure(
                Channel::B,
                config.frequency_hz,
                b_conf.duty_percent,
                config.phase_correct,
            )?;
        }

        Ok(pwm_slice)
    }
}
