use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use rp_pac::pwm::vals::Divmode;

use crate::{
    clocks::clk_sys_freq,
    gpio::{AnyPin, SealedPin},
    peripherals::{PWM_SLICE0, PWM_SLICE1, PWM_SLICE2, PWM_SLICE3, PWM_SLICE4, PWM_SLICE5, PWM_SLICE6, PWM_SLICE7},
    Peripherals, RegExt,
};

use super::{
    v2::{EdgeSensitivity, Frequency, PwmFreeRunningSlice, PwmInputOutputSlice},
    ChannelAPin, ChannelBPin, InputMode, Slice,
};

/// TODO
pub struct PwmSliceBuilder<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
}

impl<'a, T: Slice> PwmSliceBuilder<'a, T> {
    pub(crate) fn new(slice: &'a T) -> Self {
        into_ref!(slice);
        Self { inner: slice }
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
    pub fn free_running(self) -> PwmFreeRunningSliceBuilder<'a, T> {
        PwmFreeRunningSliceBuilder::new(self.inner)
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
    pub fn level_sensitive(self) -> PwmInputOutputSliceBuilder<'a, T> {
        PwmInputOutputSliceBuilder::new(self.inner, InputMode::Level)
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
    pub fn edge_sensitive(self, edge: EdgeSensitivity) -> PwmInputOutputSliceBuilder<'a, T> {
        match edge {
            EdgeSensitivity::Rising => PwmInputOutputSliceBuilder::new(self.inner, InputMode::RisingEdge),
            EdgeSensitivity::Falling => PwmInputOutputSliceBuilder::new(self.inner, InputMode::FallingEdge),
        }
    }
}

/// Builder for configuring a PWM slice in either level- or edge-sensitive
/// mode.
pub struct PwmInputOutputSliceBuilder<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pin_b: Option<PeripheralRef<'a, AnyPin>>,
    divider_int: u8,
    divider_frac: u8,
    input_mode: InputMode,
    phase_correct: bool,
}

impl<'a, T: Slice> PwmInputOutputSliceBuilder<'a, T> {
    pub(crate) fn new(slice: PeripheralRef<'a, T>, input_mode: InputMode) -> Self {
        Self {
            inner: slice,
            pin_a: None,
            pin_b: None,
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
    pub fn with_output(mut self, pin: impl Peripheral<P = impl ChannelAPin<T>> + 'a) -> Self {
        into_ref!(pin);
        self.pin_a = Some(pin.map_into());
        self
    }

    /// Assign the specified pin as an input for this slice. The pin must be
    /// a valid PWM input pin for the slice (only B-pins are supported as
    /// input pins in level- and edge-sensitive slice modes).
    pub fn with_input(mut self, pin: impl Peripheral<P = impl ChannelBPin<T>> + 'a) -> Self {
        into_ref!(pin);
        self.pin_b = Some(pin.map_into());
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
    pub fn apply(self) -> PwmInputOutputSlice<'a, T> {
        // B-pin is required for controlling the counter.
        if self.pin_b.is_none() {
            panic!("B-pin must be set for input mode");
        }

        let regs = self.inner.regs();

        let slice = PwmInputOutputSlice::new(self.inner, self.pin_a, self.pin_b);

        // Set the fractional divider (CH0_DIV..CH7_DIV).
        regs.div().write_set(|w| {
            w.set_int(self.divider_int);
            w.set_frac(self.divider_frac);
        });

        // Set the control and status register values.
        // (CH0_CSR..CH7_CSR).
        regs.csr().write_set(|w| {
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
pub struct PwmFreeRunningSliceBuilder<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    builder_a: Option<PwmFreeRunningChannelBuilder<'a, T>>,
    builder_b: Option<PwmFreeRunningChannelBuilder<'a, T>>,
    frequency: Frequency,
    frequency_hz: u32,
    phase_correct: bool,
}

impl<'a, T: Slice> PwmFreeRunningSliceBuilder<'a, T> {
    pub(crate) fn new(slice: PeripheralRef<'a, T>) -> Self {
        Self {
            inner: slice,
            builder_a: None,
            builder_b: None,
            frequency: Frequency::Hz(clk_sys_freq()),
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

        self.frequency = freq;
        self
    }

    /// TODO
    pub fn with_channel_a(
        mut self,
        pin: impl Peripheral<P = impl ChannelAPin<T>> + 'a,
        cfg: impl FnOnce(PwmFreeRunningChannelBuilder<'a, T>) -> PwmFreeRunningChannelBuilder<'a, T>,
    ) -> Self {
        let mut builder = PwmFreeRunningChannelBuilder::<T>::new_a(pin);
        builder = cfg(builder);
        self.builder_a = Some(builder);
        self
    }

    /// TODO
    pub fn with_channel_b(
        mut self,
        pin: impl Peripheral<P = impl ChannelBPin<T>> + 'a,
        cfg: impl FnOnce(PwmFreeRunningChannelBuilder<'a, T>) -> PwmFreeRunningChannelBuilder<'a, T>,
    ) -> Self {
        let mut builder = PwmFreeRunningChannelBuilder::<T>::new_b(pin);
        builder = cfg(builder);
        self.builder_b = Some(builder);
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

    /// Applies the configuration of this builder to the PWM slice and
    /// GPIO pins. This method will return a configured PWM slice that can
    /// be enabled to start generating PWM signals.
    ///
    /// Note that this will not enable the slice, only configure it. You must
    /// call [`PwmInputOutputSlice::enable`], or alternatively use the
    /// [`enable_pwm_slices`] function, to start the slice.
    pub fn apply(mut self) -> PwmFreeRunningSlice<'a, T> {
        // Require that at least one of A or B is configured.
        if self.builder_a.is_none() && self.builder_b.is_none() {
            panic!("At least one channel must be configured");
        }

        let regs = self.inner.regs();

        let slice: PwmFreeRunningSlice<'_, T> = PwmFreeRunningSlice::new(
            self.inner,
            self.frequency_hz,
            self.phase_correct,
            self.builder_a.as_ref().map(|a| a.duty_percent),
            self.builder_b.as_ref().map(|b| b.duty_percent),
            self.builder_a.take().map(|b| b.pin),
            self.builder_b.take().map(|b| b.pin),
        );

        // If channel A is configured, set the pin function to PWM.
        if let Some(pin) = &slice.pin_a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
        }

        // If channel B is configured, set the pin function to PWM.
        if let Some(pin) = &slice.pin_b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
        }

        // Set the control and status register values.
        // (CH0_CSR..CH7_CSR).
        regs.csr().write_set(|w| {
            // TODO: Calculate div/top for slice

            if let Some(a) = self.builder_a {
                w.set_a_inv(a.invert);

                // TODO: Calculate compare value for A
            }
            if let Some(b) = self.builder_b {
                w.set_b_inv(b.invert);

                // TODO: Calculate compare value for B
            }
            w.set_ph_correct(self.phase_correct);
        });

        slice
    }
}

/// Builder for configuring the PWM input channel (`B`) for a level- or
/// edge-sensitive slice.
pub struct PwmInputChannelBuilder<'a, T: Slice> {
    _slice: PhantomData<T>,
    pin: PeripheralRef<'a, AnyPin>,
}

impl<'a, T: Slice> PwmInputChannelBuilder<'a, T> {
    pub(crate) fn new(pin: PeripheralRef<'a, AnyPin>) -> Self {
        Self {
            _slice: PhantomData,
            pin,
        }
    }
}

/// Builder for configuring a PWM output channel (`A`) in a level- or
/// edge-sensitive slice.
pub struct PwmOutputChannelBuilder<'a, T: Slice> {
    _slice: PhantomData<T>,
    pin: PeripheralRef<'a, AnyPin>,
}

impl<'a, T: Slice> PwmOutputChannelBuilder<'a, T> {
    pub(crate) fn new(pin: PeripheralRef<'a, AnyPin>) -> Self {
        Self {
            _slice: PhantomData,
            pin,
        }
    }
}

/// Builder for configuring a PWM channel (`A` or `B`) in a free-running
/// slice.
pub struct PwmFreeRunningChannelBuilder<'a, T: Slice> {
    _slice: PhantomData<T>,
    pin: PeripheralRef<'a, AnyPin>,
    duty_percent: f32,
    invert: bool,
}

impl<'a, T: Slice> PwmFreeRunningChannelBuilder<'a, T> {
    pub(crate) fn new_a(pin: impl Peripheral<P = impl ChannelAPin<T>> + 'a) -> Self {
        into_ref!(pin);
        Self::new_inner(pin.map_into())
    }

    pub(crate) fn new_b(pin: impl Peripheral<P = impl ChannelBPin<T>> + 'a) -> Self {
        into_ref!(pin);
        Self::new_inner(pin.map_into())
    }

    fn new_inner(pin: PeripheralRef<'a, AnyPin>) -> Self {
        into_ref!(pin);
        Self {
            _slice: PhantomData,
            pin,
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

/// Extension trait for the RP2040 [`Peripherals`] struct, providing
/// convenience methods for configuring PWM slices.
pub trait PeripheralsExt {
    /// Returns a builder for configuring PWM slice 0.
    fn pwm_0(&self) -> PwmSliceBuilder<PWM_SLICE0>;
    /// Returns a builder for configuring PWM slice 1.
    fn pwm_1(&self) -> PwmSliceBuilder<PWM_SLICE1>;
    /// Returns a builder for configuring PWM slice 2.
    fn pwm_2(&self) -> PwmSliceBuilder<PWM_SLICE2>;
    /// Returns a builder for configuring PWM slice 3.
    fn pwm_3(&self) -> PwmSliceBuilder<PWM_SLICE3>;
    /// Returns a builder for configuring PWM slice 4.
    fn pwm_4(&self) -> PwmSliceBuilder<PWM_SLICE4>;
    /// Returns a builder for configuring PWM slice 5.
    fn pwm_5(&self) -> PwmSliceBuilder<PWM_SLICE5>;
    /// Returns a builder for configuring PWM slice 6.
    fn pwm_6(&self) -> PwmSliceBuilder<PWM_SLICE6>;
    /// Returns a builder for configuring PWM slice 7.
    fn pwm_7(&self) -> PwmSliceBuilder<PWM_SLICE7>;
}

impl PeripheralsExt for Peripherals {
    fn pwm_0(&self) -> PwmSliceBuilder<PWM_SLICE0> {
        PwmSliceBuilder::new(&self.PWM_SLICE0)
    }
    fn pwm_1(&self) -> PwmSliceBuilder<PWM_SLICE1> {
        PwmSliceBuilder::new(&self.PWM_SLICE1)
    }
    fn pwm_2(&self) -> PwmSliceBuilder<PWM_SLICE2> {
        PwmSliceBuilder::new(&self.PWM_SLICE2)
    }
    fn pwm_3(&self) -> PwmSliceBuilder<PWM_SLICE3> {
        PwmSliceBuilder::new(&self.PWM_SLICE3)
    }
    fn pwm_4(&self) -> PwmSliceBuilder<PWM_SLICE4> {
        PwmSliceBuilder::new(&self.PWM_SLICE4)
    }
    fn pwm_5(&self) -> PwmSliceBuilder<PWM_SLICE5> {
        PwmSliceBuilder::new(&self.PWM_SLICE5)
    }
    fn pwm_6(&self) -> PwmSliceBuilder<PWM_SLICE6> {
        PwmSliceBuilder::new(&self.PWM_SLICE6)
    }
    fn pwm_7(&self) -> PwmSliceBuilder<PWM_SLICE7> {
        PwmSliceBuilder::new(&self.PWM_SLICE7)
    }
}
