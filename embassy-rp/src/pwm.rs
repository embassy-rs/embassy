//! Pulse Width Modulation (PWM)

use core::ops::Div;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::channel;
use embedded_hal_1::pwm::{ErrorKind, ErrorType, SetDutyCycle};
use fixed::traits::ToFixed;
use fixed::FixedU16;
use pac::pwm::regs::{ChDiv, Intr};
use pac::pwm::vals::Divmode;

use crate::clocks::clk_sys_freq;
use crate::gpio::{AnyPin, Pin as GpioPin, Pull, SealedPin as _};
use crate::{pac, peripherals, RegExt};

const MIN_PWM_FREQ: f32 = 7.5;

/// The configuration of a PWM slice.
/// Note the period in clock cycles of a slice can be computed as:
/// `(top + 1) * (phase_correct ? 1 : 2) * divider`
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Inverts the PWM output signal on channel A.
    pub invert_a: bool,
    /// Inverts the PWM output signal on channel B.
    pub invert_b: bool,
    /// Enables phase-correct mode for PWM operation.
    /// In phase-correct mode, the PWM signal is generated in such a way that
    /// the pulse is always centered regardless of the duty cycle.
    /// The output frequency is halved when phase-correct mode is enabled.
    pub phase_correct: bool,
    /// Enables the PWM slice, allowing it to generate an output.
    pub enable: bool,
    /// A fractional clock divider, represented as a fixed-point number with
    /// 8 integer bits and 4 fractional bits. It allows precise control over
    /// the PWM output frequency by gating the PWM counter increment.
    /// A higher value will result in a slower output frequency.
    pub divider: fixed::FixedU16<fixed::types::extra::U4>,
    /// The output on channel A goes high when `compare_a` is higher than the
    /// counter. A compare of 0 will produce an always low output, while a
    /// compare of `top + 1` will produce an always high output.
    pub compare_a: u16,
    /// The output on channel B goes high when `compare_b` is higher than the
    /// counter. A compare of 0 will produce an always low output, while a
    /// compare of `top + 1` will produce an always high output.
    pub compare_b: u16,
    /// The point at which the counter wraps, representing the maximum possible
    /// period. The counter will either wrap to 0 or reverse depending on the
    /// setting of `phase_correct`.
    pub top: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            invert_a: false,
            invert_b: false,
            phase_correct: false,
            enable: true, // differs from reset value
            divider: 1.to_fixed(),
            compare_a: 0,
            compare_b: 0,
            top: 0xffff,
        }
    }
}

/// Represents a frequency in Hz, KHz, or MHz.
pub enum Frequency {
    /// Frequency in Hz.
    Hz(u32),
    /// Frequency in KHz.
    KHz(f32),
    /// Frequency in MHz.
    MHz(f32),
}

/// PWM input mode.
pub enum InputMode {
    /// The fractional divider operation is gated by the PWM B pin.
    Level,
    /// The counter advances with each rising edge of the PWM B pin.
    RisingEdge,
    /// The counter advances with each falling edge of the PWM B pin.
    FallingEdge,
}

/// Which edge to trigger on in edge-sensitive input mode.
pub enum EdgeSensitivity {
    /// The counter advances with each rising edge of the PWM B pin.
    Rising,
    /// The counter advances with each falling edge of the PWM B pin.
    Falling,
}

impl From<InputMode> for Divmode {
    fn from(value: InputMode) -> Self {
        match value {
            InputMode::Level => Divmode::LEVEL,
            InputMode::RisingEdge => Divmode::RISE,
            InputMode::FallingEdge => Divmode::FALL,
        }
    }
}

/// TODO
pub mod builder {
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
        ChannelAPin, ChannelBPin, EdgeSensitivity, Frequency, InputMode, PwmFreeRunningSlice, PwmInputOutputSlice,
        Slice,
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

        /// Builds the PWM slice with the provided configuration. Note that this
        /// will *not* start the slice. You must call [`PwmInputOutputSlice::enable`] to
        pub fn configure(self) -> PwmInputOutputSlice<'a, T> {
            // B-pin is required for controlling the counter.
            if self.pin_b.is_none() {
                panic!("B-pin must be set for input mode");
            }

            let regs = self.inner.regs();

            let slice = PwmInputOutputSlice::new(
                self.inner, 
                self.pin_a, 
                self.pin_b
            );

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

        /// TODO
        pub fn configure(mut self) -> PwmFreeRunningSlice<'a, T> {
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
}

/// Represents a configured free-running PWM slice.
pub struct PwmFreeRunningSlice<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    frequency_hz: u32,
    phase_correct: bool,
    duty_a: Option<f32>,
    duty_b: Option<f32>,
    pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pin_b: Option<PeripheralRef<'a, AnyPin>>,
    div: u8,
    top: u16,
}

impl<'a, T: Slice> PwmFreeRunningSlice<'a, T> {
    pub(crate) fn new(
        slice: PeripheralRef<'a, T>,
        frequency_hz: u32,
        phase_correct: bool,
        duty_a: Option<f32>,
        duty_b: Option<f32>,
        pin_a: Option<PeripheralRef<'a, AnyPin>>,
        pin_b: Option<PeripheralRef<'a, AnyPin>>,
    ) -> Self {
        Self {
            inner: slice,
            frequency_hz,
            phase_correct,
            duty_a,
            duty_b,
            pin_a,
            pin_b,
            div: 1,
            top: u16::MAX
        }
    }

    /// TODO DUTY CYCLE
    fn reconfigure(
        &mut self, 
        channel: Channel, 
        freq_hz: u32, 
        duty: f32, 
        phase_correct: bool
    ) -> Result<(), ErrorKind> {
        // Check for changes and assert that the provided channel is configured
        // prior to reconfiguring.
        if !self.reconfigure_precheck(channel, freq_hz, duty, phase_correct)? {
            return Ok(());
        }

        // Recalculate the divider and top values based on the requested changes.
        self.calculate_div_and_top(freq_hz);

        // Update the DIV register with the new divider value.
        self.inner.regs().div().write_set(|w| w.set_int(self.div));
        // Update the TOP register with the new top (wrap) value.
        self.inner.regs().top().write_set(|w| w.set_top(self.top));

        // If phase correct mode is changed, update the CSR register.
        if phase_correct != self.phase_correct {
            self.inner.regs().csr().modify(|w| w.set_ph_correct(phase_correct));
            self.phase_correct = phase_correct;
        }
        

        Ok(())
    }

    fn reconfigure_precheck(
        &mut self, 
        channel: Channel, 
        freq_hz: u32, 
        duty: f32, 
        phase_correct: bool
    ) -> Result<bool, ErrorKind> {
        // Check for changes and assert that the provided channel is configured
        // prior to reconfiguring.
        match channel {
            Channel::A => {
                if let Some(duty_a) = self.duty_a {
                    if duty_a == duty 
                        && freq_hz == self.frequency_hz 
                        && phase_correct == self.phase_correct 
                    {
                        debug!("No changes have been made, skipping reconfiguration.");
                        return Ok(false);
                    }
                } else {
                    // TODO: Use proper error
                    return Err(ErrorKind::Other);
                }
            },
            Channel::B => {
                if let Some(duty_b) = self.duty_b {
                    if duty_b == duty && freq_hz == self.frequency_hz && phase_correct == self.phase_correct{
                        debug!("No changes have been made, skipping reconfiguration.");
                        return Ok(false);
                    }
                } else {
                    // TODO: Use proper error
                    return Err(ErrorKind::Other);
                }
            }
        }

        Ok(true)
    }

    fn calculate_div_and_top(&mut self, freq_hz: u32) {
        let mut clk_divider = 0;
        let mut wrap = 0;
        let mut clock_div;
        let clock = clk_sys_freq();

        for div in 1..u8::MAX as u32 {
            clk_divider = div;
            // Find clock_division to fit current frequency.
            clock_div = clock.div(div);
            wrap = clock_div / freq_hz;
            if clock_div / u16::MAX as u32 <= freq_hz && wrap <= u8::MAX as u32 {
                break;
            }
        }

        if self.phase_correct {
            wrap = wrap / 2;
        }

        self.div = clk_divider as u8;
        self.top = wrap as u16;
        
        debug!("Changing frequency to {} Hz (from {}), using divider {} and top {}", freq_hz, self.frequency_hz, clk_divider, wrap);

        self.frequency_hz = freq_hz;
    }

    /// Sets the duty cycle for this channel. The duty cycle is a percentage
    /// value between 0.0 and 100.0. Defaults to 0.0.
    pub fn set_duty_cycle(&mut self, channel: Channel, duty: f32) -> Result<(), ErrorKind> {
        self.reconfigure(channel, self.frequency_hz, duty, self.phase_correct)
    }
}

/// Represents a configured slice in level- or edge-sensitive mode.
pub struct PwmInputOutputSlice<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pin_b: Option<PeripheralRef<'a, AnyPin>>,
}

impl<'a, T: Slice> PwmInputOutputSlice<'a, T> {
    pub(crate) fn new(
        slice: PeripheralRef<'a, T>,
        pin_a: Option<PeripheralRef<'a, AnyPin>>,
        pin_b: Option<PeripheralRef<'a, AnyPin>>,
    ) -> Self {
        Self {
            inner: slice,
            pin_a,
            pin_b,
        }
    }
}

/// Trait encapsulating common logic for the different PWM slice implementations.
pub trait AsPwmSlice<T: Slice> {
    /// Returns the slice instance.
    fn slice(&self) -> &T;
    /// Enables the PWM slice, starting sampling/generation.
    ///
    /// Note that if you need to start multiple slices simultaneously so that
    /// they run in perfect sync, you should use [`enable_pwm_slices`] instead.
    /// Starting multiple slices individually will result in the counters
    /// starting at different clock cycles.
    fn enable(&self) {
        self.slice().regs().csr().modify(|w| w.set_en(true));
    }

    /// Disables the PWM slice, stopping sampling/generation.
    fn disable(&self) {
        self.slice().regs().csr().modify(|w| w.set_en(false));
    }

    /// Advances the phase of the counter by 1 count while it is still running.
    fn phase_advance(&self) {
        self.slice().regs().csr().modify(|w| w.set_ph_adv(true));
    }

    /// Retards the phase of the counter by 1 count while it is still running.
    fn phase_retard(&self) {
        self.slice().regs().csr().modify(|w| w.set_ph_ret(true));
    }
}

impl<'a, T: Slice> AsPwmSlice<T> for PwmInputOutputSlice<'a, T> {
    fn slice(&self) -> &T {
        &self.inner
    }
}

impl<'a, T: Slice> AsPwmSlice<T> for PwmFreeRunningSlice<'a, T> {
    fn slice(&self) -> &T {
        &self.inner
    }
}

/// PWM slice.
pub struct PwmSlice;

impl PwmSlice {
    /// Get a builder for configuring a PWM slice.
    pub fn builder<'a, T: Slice>(slice: &'a T) -> builder::PwmSliceBuilder<'a, T> {
        builder::PwmSliceBuilder::new(slice)
    }

    // /// Enable multiple PWM slices at once. This is more efficient than
    // /// enabling each slice individually and results in all slice counters
    // /// starting at the same clock cycle.
    // pub fn enable(slices: impl IntoIterator<Item = impl AsPwmSliceNumber>) {
    //     let mut mask: u32 = 0;
    //     for slice in slices.into_iter() {
    //         let slice: u8 = slice.slice_number();
    //         mask |= 1u32 << slice;
    //     }
    //     pac::PWM.en().write_set(|w| w.0 = mask);
    // }
}

/// Enable multiple PWM slices simultaneously, causing them to start on the
/// same clock cycle.
pub fn enable_pwm_slices(slices: impl FnOnce(&mut SliceMask) -> &mut SliceMask) {
    let mut mask = SliceMask::default();
    slices(&mut mask);
    pac::PWM.en().write_set(|w| w.0 = mask.mask);
}

/// Disable multiple PWM slices simultaneously.
/// TODO: Implement this function.
pub fn disable_pwm_slices(slices: impl FnOnce(&mut SliceMask) -> &mut SliceMask) {
    let mut mask = SliceMask::default();
    slices(&mut mask);
    todo!("Negate the current mask with the selected slices");
}

/// Mask of PWM slice numbers used for enabling or disabling multiple slices
/// simultaneously.
pub struct SliceMask {
    mask: u32,
}

impl Default for SliceMask {
    fn default() -> Self {
        Self { mask: 0 }
    }
}

impl SliceMask {
    /// Adds slice 0 to the mask.
    pub fn slice_0(&mut self) -> &mut Self {
        self.mask |= 1 << 0;
        self
    }
    /// Adds slice 1 to the mask.
    pub fn slice_1(&mut self) -> &mut Self {
        self.mask |= 1 << 1;
        self
    }
    /// Adds slice 2 to the mask.
    pub fn slice_2(&mut self) -> &mut Self {
        self.mask |= 1 << 2;
        self
    }
    /// Adds slice 3 to the mask.
    pub fn slice_3(&mut self) -> &mut Self {
        self.mask |= 1 << 3;
        self
    }
    /// Adds slice 4 to the mask.
    pub fn slice_4(&mut self) -> &mut Self {
        self.mask |= 1 << 4;
        self
    }
    /// Adds slice 5 to the mask.
    pub fn slice_5(&mut self) -> &mut Self {
        self.mask |= 1 << 5;
        self
    }
    /// Adds slice 6 to the mask.
    pub fn slice_6(&mut self) -> &mut Self {
        self.mask |= 1 << 6;
        self
    }
    /// Adds slice 7 to the mask.
    pub fn slice_7(&mut self) -> &mut Self {
        self.mask |= 1 << 7;
        self
    }

    /// Adds a slice instance to the mask.
    pub fn add(&mut self, slice: impl AsPwmSliceNumber) {
        self.mask |= 1 << slice.slice_number();
    }
}

/// Helper trait for getting the slice number of a PWM slice.
pub trait AsPwmSliceNumber {
    /// Returns the slice number.
    fn slice_number(&self) -> u8;
}

impl<'a, T: Slice> AsPwmSliceNumber for PwmFreeRunningSlice<'a, T> {
    fn slice_number(&self) -> u8 {
        self.inner.number()
    }
}

impl<'a, T: Slice> AsPwmSliceNumber for &PwmFreeRunningSlice<'a, T> {
    fn slice_number(&self) -> u8 {
        self.inner.number()
    }
}

/// PWN channel. Each slice has two channels, A and B.
#[derive(PartialEq, Eq)]
pub enum Channel {
    /// Channel A of a slice.
    A,
    /// Channel B of a slice.
    B,
}

/// PWM driver.
pub struct Pwm<'d, T: Slice> {
    inner: PeripheralRef<'d, T>,
    pin_a: Option<PeripheralRef<'d, AnyPin>>,
    pin_b: Option<PeripheralRef<'d, AnyPin>>,
    freq: u32,
    duty_a: Option<u16>,
    duty_b: Option<u16>,
}

impl<'d, T: Slice> Pwm<'d, T> {
    fn new_inner(
        inner: impl Peripheral<P = T> + 'd,
        a: Option<PeripheralRef<'d, AnyPin>>,
        b: Option<PeripheralRef<'d, AnyPin>>,
        b_pull: Pull,
        config: Config,
        divmode: Divmode,
    ) -> Self {
        into_ref!(inner);

        let p = inner.regs();
        p.csr().modify(|w| {
            w.set_divmode(divmode);
            w.set_en(false);
        });
        p.ctr().write(|w| w.0 = 0);
        Self::configure(p, &config);

        if let Some(pin) = &a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
        }
        if let Some(pin) = &b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
            pin.pad_ctrl().modify(|w| {
                w.set_pue(b_pull == Pull::Up);
                w.set_pde(b_pull == Pull::Down);
            });
        }
        Self {
            inner,
            pin_a: a,
            pin_b: b,
            freq: clk_sys_freq(),
            duty_a: None,
            duty_b: None,
        }
    }

    /// Create PWM driver without any configured pins.
    #[inline]
    pub fn new_free(inner: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        Self::new_inner(inner, None, None, Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'a' as output.
    #[inline]
    pub fn new_output_a(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl ChannelAPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a);
        Self::new_inner(inner, Some(a.map_into()), None, Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'b' pin as output.
    #[inline]
    pub fn new_output_b(
        inner: impl Peripheral<P = T> + 'd,
        b: impl Peripheral<P = impl ChannelBPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(b);
        Self::new_inner(inner, None, Some(b.map_into()), Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a 'a' and 'b' pins as output.
    #[inline]
    pub fn new_output_ab(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl ChannelAPin<T>> + 'd,
        b: impl Peripheral<P = impl ChannelBPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a, b);
        Self::new_inner(
            inner,
            Some(a.map_into()),
            Some(b.map_into()),
            Pull::None,
            config,
            Divmode::DIV,
        )
    }

    /// Create PWM driver with a single 'b' as input pin.
    #[inline]
    pub fn new_input(
        inner: impl Peripheral<P = T> + 'd,
        b: impl Peripheral<P = impl ChannelBPin<T>> + 'd,
        b_pull: Pull,
        mode: InputMode,
        config: Config,
    ) -> Self {
        into_ref!(b);
        Self::new_inner(inner, None, Some(b.map_into()), b_pull, config, mode.into())
    }

    /// Create PWM driver with a 'a' and 'b' pins in the desired input mode.
    #[inline]
    pub fn new_output_input(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl ChannelAPin<T>> + 'd,
        b: impl Peripheral<P = impl ChannelBPin<T>> + 'd,
        b_pull: Pull,
        mode: InputMode,
        config: Config,
    ) -> Self {
        into_ref!(a, b);
        Self::new_inner(
            inner,
            Some(a.map_into()),
            Some(b.map_into()),
            b_pull,
            config,
            mode.into(),
        )
    }

    /// Set the PWM config.
    #[inline]
    pub fn set_config(&mut self, config: &Config) {
        Self::configure(self.inner.regs(), config);
    }

    #[inline]
    fn configure(p: pac::pwm::Channel, config: &Config) {
        if config.divider > FixedU16::<fixed::types::extra::U4>::from_bits(0xFFF) {
            panic!("Requested divider is too large");
        }

        p.div().write_value(ChDiv(config.divider.to_bits() as u32));
        p.cc().write(|w| {
            w.set_a(config.compare_a);
            w.set_b(config.compare_b);
        });
        p.top().write(|w| w.set_top(config.top));
        p.csr().modify(|w| {
            w.set_a_inv(config.invert_a);
            w.set_b_inv(config.invert_b);
            w.set_ph_correct(config.phase_correct);
            w.set_en(config.enable);
        });
    }

    /// Sets the PWM frequency for this slice. Note that this affects
    /// both channels (A and B).
    ///
    /// @param freq: The frequency in Hz.
    ///
    /// TODO: Add support for fractional hz
    #[inline]
    pub fn set_freq(&mut self, freq: u32) -> Result<(), ErrorKind> {
        let clock_hz = clk_sys_freq() / 1_000_000;
        let min_hz = clock_hz / 125;
        let max_hz = clock_hz / 2;

        // The frequency must be between 8 and the clock frequency
        if freq < MIN_PWM_FREQ as u32 || freq > max_hz as u32 {
            return Err(ErrorKind::Other);
        }

        // Only perform recalculations if the frequency has changed
        if freq == self.freq {
            return Ok(());
        }

        self.freq = freq;

        self.recalculate_div_wrap(freq)?;

        if let Some(duty_a) = self.duty_a {
            self.recalculate_duty(Channel::A, duty_a)?;
        }

        if let Some(duty_b) = self.duty_b {
            self.recalculate_duty(Channel::B, duty_b)?;
        }
        Ok(())
    }

    /// Get the current PWM frequency for this slice.
    #[inline]
    pub fn freq(&self) -> u32 {
        self.freq
    }

    /// Set the PWM duty cycle for channel A. Returns an error if the slice
    /// does not have a channel A configured.
    #[inline]
    pub fn set_duty_a(&mut self, duty: u16) -> Result<(), ErrorKind> {
        if self.pin_a.is_none() {
            return Err(ErrorKind::Other);
        }

        self.recalculate_duty(Channel::A, duty)?;
        self.duty_a = Some(duty);
        Ok(())
    }

    /// Set the PWM duty cycle for channel B. Returns an error if the slice
    /// does not have a channel B configured.
    #[inline]
    pub fn set_duty_b(&mut self, duty: u16) -> Result<(), ErrorKind> {
        if self.pin_b.is_none() {
            return Err(ErrorKind::Other);
        }

        self.recalculate_duty(Channel::B, duty)?;
        self.duty_b = Some(duty);
        Ok(())
    }

    /// Set the PWM duty cycle for both channels A and B. Returns an error if
    /// the slice does not have both A & B channels configured.
    #[inline]
    pub fn set_duty_ab(&mut self, duty: u16) -> Result<(), ErrorKind> {
        if self.pin_a.is_none() || self.pin_b.is_none() {
            return Err(ErrorKind::Other);
        }

        self.set_duty_a(duty)?;
        self.set_duty_b(duty)
    }

    /// Advances a slice’s output phase by one count while it is running
    /// by inserting a pulse into the clock enable. The counter
    /// will not count faster than once per cycle.
    #[inline]
    pub fn phase_advance(&mut self) {
        let p = self.inner.regs();
        p.csr().write_set(|w| w.set_ph_adv(true));
        while p.csr().read().ph_adv() {}
    }

    /// Retards a slice’s output phase by one count while it is running
    /// by deleting a pulse from the clock enable. The counter will not
    /// count backward when clock enable is permenantly low.
    #[inline]
    pub fn phase_retard(&mut self) {
        let p = self.inner.regs();
        p.csr().write_set(|w| w.set_ph_ret(true));
        while p.csr().read().ph_ret() {}
    }

    /// Read PWM counter.
    #[inline]
    pub fn counter(&self) -> u16 {
        self.inner.regs().ctr().read().ctr()
    }

    /// Write PWM counter.
    #[inline]
    pub fn set_counter(&self, ctr: u16) {
        self.inner.regs().ctr().write(|w| w.set_ctr(ctr))
    }

    /// Wait for channel interrupt.
    #[inline]
    pub fn wait_for_wrap(&mut self) {
        while !self.wrapped() {}
        self.clear_wrapped();
    }

    /// Check if interrupt for channel is set.
    #[inline]
    pub fn wrapped(&mut self) -> bool {
        pac::PWM.intr().read().0 & self.bit() != 0
    }

    #[inline]
    /// Clear interrupt flag.
    pub fn clear_wrapped(&mut self) {
        pac::PWM.intr().write_value(Intr(self.bit() as _));
    }

    /// Enables the PWM counter for this slice. Note that enablement can only
    /// be specified on the slice-level, so this affects both PWM channels
    /// (A & B).
    #[inline]
    pub fn enable(&mut self) {
        self.inner.regs().csr().write(|w| w.set_en(true));
    }

    /// Disables the PWM counter for this slice. Note that enablement can only
    /// be specified on the slice-level, so this affects both PWM channels
    /// (A & B).
    #[inline]
    pub fn disable(&mut self) {
        self.inner.regs().csr().write(|w| w.set_en(false));
    }

    #[inline]
    fn bit(&self) -> u32 {
        1 << self.inner.number() as usize
    }

    /// Recalculates the TOP and DIV values and updates the PWM slice registers.
    #[inline]
    fn recalculate_div_wrap(&self, freq: u32) -> Result<(), ErrorKind> {
        let mut clk_divider = 0;
        let mut wrap = 0;
        let mut clock_div;
        let clock = clk_sys_freq();

        // Only recalculate divider if frequency has changed
        for div in 1..u8::MAX as u32 {
            clk_divider = div;
            // Find clock_division to fit current frequency
            clock_div = clock.div(div);
            wrap = clock_div / freq;
            if clock_div / u16::MAX as u32 <= freq && wrap <= u16::MAX as u32 {
                break;
            }
        }

        if clk_divider < u8::MAX as u32 {
            // Only update divider and top registers if the frequency has
            // changed
            self.inner.regs().div().write(|w| w.set_int(clk_divider as u8));
            self.inner.regs().top().write(|w| w.set_top(wrap as u16));
            Ok(())
        } else {
            Err(ErrorKind::Other)
        }
    }

    /// Recalculates the duty cycle for a channel and updates the PWM slice CC
    /// register with the new values for both A and B channels.
    #[inline]
    fn recalculate_duty(&self, channel: Channel, duty: u16) -> Result<(), ErrorKind> {
        // Get the current `DIV` register value for this slice (only the
        // integer part is used)
        let wrap = self.inner.regs().div().read().int();

        // Update the `CC` register.
        self.inner.regs().cc().write(|w| {
            let compare = ((((wrap + if duty == 100 { 1 } else { 0 }) as u16) * duty) / 100) as u16;

            if self.pin_a.is_some() && channel == Channel::A {
                w.set_a(compare);
            } else if self.pin_b.is_some() && channel == Channel::B {
                w.set_b(compare);
            } else {
                // We shouldn't have been able to get here if the `Pwm`
                // constructors are doing their job.
                return Err(ErrorKind::Other);
            }
            Ok(())
        })
    }
}

/// Batch representation of PWM slices.
pub struct PwmBatch(u32);

impl PwmBatch {
    #[inline]
    /// Enable a PWM slice in this batch.
    pub fn enable(&mut self, pwm: &Pwm<'_, impl Slice>) {
        self.0 |= pwm.bit();
    }

    #[inline]
    /// Enable slices in this batch in a PWM.
    pub fn set_enabled(enabled: bool, batch: impl FnOnce(&mut PwmBatch)) {
        let mut en = PwmBatch(0);
        batch(&mut en);
        if enabled {
            pac::PWM.en().write_set(|w| w.0 = en.0);
        } else {
            pac::PWM.en().write_clear(|w| w.0 = en.0);
        }
    }
}

impl<'d, T: Slice> Drop for Pwm<'d, T> {
    fn drop(&mut self) {
        self.inner.regs().csr().write_clear(|w| w.set_en(false));
        if let Some(pin) = &self.pin_a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(31));
        }
        if let Some(pin) = &self.pin_b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(31));
        }
    }
}

trait SealedSlice {}

/// PWM Slice.
#[allow(private_bounds)]
pub trait Slice: Peripheral<P = Self> + SealedSlice + Sized + 'static {
    /// Slice number.
    fn number(&self) -> u8;

    /// Slice register block.
    fn regs(&self) -> pac::pwm::Channel {
        pac::PWM.ch(self.number() as _)
    }
}

macro_rules! slice {
    ($name:ident, $num:expr) => {
        impl SealedSlice for peripherals::$name {}
        impl Slice for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }
        //impl SealedSlice for PwmFreeRunningSlice<'_, peripherals::$name> {}
    };
}

slice!(PWM_SLICE0, 0);
slice!(PWM_SLICE1, 1);
slice!(PWM_SLICE2, 2);
slice!(PWM_SLICE3, 3);
slice!(PWM_SLICE4, 4);
slice!(PWM_SLICE5, 5);
slice!(PWM_SLICE6, 6);
slice!(PWM_SLICE7, 7);

/// PWM Channel A.
pub trait ChannelAPin<T: Slice>: GpioPin {}
/// PWM Channel B.
pub trait ChannelBPin<T: Slice>: GpioPin {}

macro_rules! impl_pin {
    ($pin:ident, $channel:ident, $kind:ident) => {
        impl $kind<peripherals::$channel> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, PWM_SLICE0, ChannelAPin);
impl_pin!(PIN_1, PWM_SLICE0, ChannelBPin);
impl_pin!(PIN_2, PWM_SLICE1, ChannelAPin);
impl_pin!(PIN_3, PWM_SLICE1, ChannelBPin);
impl_pin!(PIN_4, PWM_SLICE2, ChannelAPin);
impl_pin!(PIN_5, PWM_SLICE2, ChannelBPin);
impl_pin!(PIN_6, PWM_SLICE3, ChannelAPin);
impl_pin!(PIN_7, PWM_SLICE3, ChannelBPin);
impl_pin!(PIN_8, PWM_SLICE4, ChannelAPin);
impl_pin!(PIN_9, PWM_SLICE4, ChannelBPin);
impl_pin!(PIN_10, PWM_SLICE5, ChannelAPin);
impl_pin!(PIN_11, PWM_SLICE5, ChannelBPin);
impl_pin!(PIN_12, PWM_SLICE6, ChannelAPin);
impl_pin!(PIN_13, PWM_SLICE6, ChannelBPin);
impl_pin!(PIN_14, PWM_SLICE7, ChannelAPin);
impl_pin!(PIN_15, PWM_SLICE7, ChannelBPin);
impl_pin!(PIN_16, PWM_SLICE0, ChannelAPin);
impl_pin!(PIN_17, PWM_SLICE0, ChannelBPin);
impl_pin!(PIN_18, PWM_SLICE1, ChannelAPin);
impl_pin!(PIN_19, PWM_SLICE1, ChannelBPin);
impl_pin!(PIN_20, PWM_SLICE2, ChannelAPin);
impl_pin!(PIN_21, PWM_SLICE2, ChannelBPin);
impl_pin!(PIN_22, PWM_SLICE3, ChannelAPin);
impl_pin!(PIN_23, PWM_SLICE3, ChannelBPin);
impl_pin!(PIN_24, PWM_SLICE4, ChannelAPin);
impl_pin!(PIN_25, PWM_SLICE4, ChannelBPin);
impl_pin!(PIN_26, PWM_SLICE5, ChannelAPin);
impl_pin!(PIN_27, PWM_SLICE5, ChannelBPin);
impl_pin!(PIN_28, PWM_SLICE6, ChannelAPin);
impl_pin!(PIN_29, PWM_SLICE6, ChannelBPin);

/// Note that this implementation is not representative of `RP2040`'s PWM, which
/// has two channels per slice. Calling [`SetDutyCycle::set_duty_cycle`]
/// from `embedded-hal` will set the duty cycle for both channels A and B. If
/// you need to set the duty cycle for each channel individually, you can use
/// the [`Pwm::set_duty_a`] and [`Pwm::set_duty_b`] methods.
impl<T: Slice> SetDutyCycle for Pwm<'_, T> {
    fn max_duty_cycle(&self) -> u16 {
        u16::MAX
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.recalculate_duty(Channel::A, duty)?;
        self.recalculate_duty(Channel::B, duty)
    }
}

/// TODO: Expand on actual error types.
impl<T: Slice> ErrorType for Pwm<'_, T> {
    type Error = ErrorKind;
}
