use core::marker::PhantomData;

use rp_pac::pwm::vals::Divmode;

use super::v2::Frequency;
use crate::clocks::clk_sys_freq;

/// Module containing the builders for configuring a 32-bit PWM counter using DMA.
pub mod counter;
/// Module containing the builders for configuring free-running slices.
pub mod free_running;
/// Module containing the builders for configuring level- and edge-sensitive slices.
pub mod level_edge_sensitive;

/// Configuration object for a PWM slice.
pub struct SliceConfig {
    div: u32,
    #[allow(dead_code)] // TODO: Temporary, to be used in level/edge-sensitive slices
    div_mode: Divmode,
    frequency_hz: u32,
    phase_correct: bool,
    a: Option<ChannelConfig>,
    b: Option<ChannelConfig>,
    enable_dma: bool,
}

impl Default for SliceConfig {
    fn default() -> Self {
        SliceConfig {
            div: 1,
            div_mode: Divmode::DIV,
            frequency_hz: clk_sys_freq(),
            phase_correct: false,
            a: None,
            b: None,
            enable_dma: false,
        }
    }
}

/// Configuration object for PWM channels (A + B pins) within a slice.
pub struct ChannelConfig {
    duty_percent: f32,
    invert: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        ChannelConfig {
            duty_percent: 100.0,
            invert: false,
        }
    }
}

/// Builder object for configuring a PWM slice.
pub struct PwmBuilder<STATE: BuilderState> {
    config: SliceConfig,
    _phantom: PhantomData<STATE>,
}

impl<STATE: BuilderState> PwmBuilder<STATE> {
    pub(crate) fn new(config: SliceConfig) -> Self {
        PwmBuilder {
            config,
            _phantom: PhantomData,
        }
    }
}

impl<STATE: BuilderState> BuilderState for PwmBuilder<STATE> {
    fn get_config(&mut self) -> &mut SliceConfig {
        &mut self.config
    }

    fn get_config_owned(self) -> SliceConfig {
        self.config
    }
}

/// Trait which is implemented for all states of the PWM builder which allows
/// internals to retrieve the current configuration via other traits.
pub trait BuilderState {
    /// Get a mutable reference to the configuration object.
    fn get_config(&mut self) -> &mut SliceConfig;

    /// Get the owned configuration object, consuming the state object.
    fn get_config_owned(self) -> SliceConfig;
}

/// Trait for configuring the phase-correct mode for the slice.
pub trait ConfigurePhaseCorrect
where
    Self: Sized + BuilderState,
{
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
    fn phase_correct(mut self, phase_correct: bool) -> Self {
        self.get_config().phase_correct = phase_correct;
        self
    }
}

/// Trait for configuring the divider for the slice. This is only valid on
/// level- and edge-sensitive slices.
///
pub trait ConfigureDivider
where
    Self: Sized + BuilderState,
{
    /// Sets the fractional divider for this slice. The divider is a 16-bit
    /// fixed-point number with 4 fractional bits. The divider controls the
    /// rate at which the counter increments.
    fn divider(mut self, div: u8) -> Self {
        self.get_config().div = div as u32;
        self
    }
}

/// Trait for configuring DMA for the slice.
pub trait ConfigureDMA
where
    Self: Sized + BuilderState,
{
    /// Enable a DMA side-channel for this PWM slice. When enabled,
    /// a DMA channel will be created and its 32-bit down-counter used to
    /// measure the PWM signal on the input pin.
    fn enable_dma(mut self) -> Self {
        self.get_config().enable_dma = true;
        self
    }
}

/// Trait for configuring the frequency for the slice. This is only valid on
/// free-running slices.
pub trait ConfigureFrequency
where
    Self: Sized + BuilderState,
{
    /// Sets the frequency for this PWM slice. The frequency can be set in
    /// Hz, KHz, or MHz. The frequency must be between 8 Hz and the system
    /// clock frequency.
    fn frequency(mut self, freq: Frequency) -> Self {
        self.get_config().frequency_hz = match freq {
            Frequency::Hz(hz) => hz,
            Frequency::KHz(khz) => (khz * 1000.0) as u32,
            Frequency::MHz(mhz) => (mhz * 1000000.0) as u32,
        };
        self
    }
}

/// Macro for generating a builder state struct.
#[macro_export]
macro_rules! builder_state {
    ($name:ident) => {
        /// $name state object for the PWM builder.
        pub struct $name(SliceConfig);
        impl BuilderState for $name {
            fn get_config(&mut self) -> &mut SliceConfig {
                &mut self.0
            }
            fn get_config_owned(self) -> SliceConfig {
                self.0
            }
        }
    };
}

builder_state!(DivMode);
builder_state!(PhaseCorrect);

#[test]
fn test() {
    let pwm = super::v2::Pwm::builder()
        .free_running()
        .frequency(Frequency::Hz(500))
        //.with_output_b(|b| b)
        .with_output_a(|a| a)
        .with_output_b(|b| b)
        //.apply(slice, pin_a, pin_b)
        ;
}
