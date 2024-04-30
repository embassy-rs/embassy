#[cfg(feature = "defmt")]
use defmt::Format;
use embassy_hal_internal::PeripheralRef;
use embedded_hal_1::pwm::ErrorKind;
use rp_pac::pwm::regs::ChTop;

use super::builder::{DivMode, PwmBuilder, SliceConfig};
use super::Slice;
use crate::clocks::clk_sys_freq;
use crate::gpio::AnyPin;
use crate::RegExt;

/// Error type for PWM operations.
#[derive(Debug)]
pub enum PwmError {
    /// A generic PWM error has occurred.
    Other(ErrorKind),
    /// An operation was attempted on a channel that has not been configured.
    ChannelNotConfigured(Channel),
    /// A configuration error has occurred.
    Configuration(&'static str),
}

impl embedded_hal_1::pwm::Error for PwmError {
    fn kind(&self) -> ErrorKind {
        todo!()
    }
}

#[cfg(feature = "defmt")]
impl Format for PwmError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            PwmError::Other(_) => defmt::write!(f, "A generic PWM error has occurred."),
            PwmError::ChannelNotConfigured(channel) => defmt::write!(f, "Channel {} is not configured", channel),
            PwmError::Configuration(msg) => defmt::write!(f, "Configuration error: {}", msg),
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

/// PWM channel. Each slice has two channels, A and B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Channel {
    /// Channel A of a slice.
    A,
    /// Channel B of a slice.
    B,
}

impl core::fmt::Display for Channel {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Channel::A => write!(f, "A"),
            Channel::B => write!(f, "B"),
        }
    }
}

#[cfg(feature = "defmt")]
impl Format for Channel {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Channel::A => defmt::write!(f, "A"),
            Channel::B => defmt::write!(f, "B"),
        }
    }
}

/// Which edge to trigger on in edge-sensitive input mode.
pub enum EdgeSensitivity {
    /// The counter advances with each rising edge of the PWM B pin.
    Rising,
    /// The counter advances with each falling edge of the PWM B pin.
    Falling,
}

/// Entry point for configuring PWM slices.
pub struct Pwm;
impl Pwm {
    /// Returns a builder for configuring a PWM slice.
    pub fn builder() -> PwmBuilder<DivMode> {
        PwmBuilder::new(SliceConfig::default())
    }
}

/// Represents a configured free-running PWM slice.
pub struct PwmFreeRunningSlice<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    frequency_hz: u32,
    phase_correct: bool,
    duty_a: Option<f32>,
    duty_b: Option<f32>,
    pub(crate) pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pub(crate) pin_b: Option<PeripheralRef<'a, AnyPin>>,
    div: u32,
    top: u32,
}

impl<'a, T: Slice> PwmFreeRunningSlice<'a, T> {
    pub(crate) fn new(
        slice: PeripheralRef<'a, T>,
        pin_a: Option<PeripheralRef<'a, AnyPin>>,
        pin_b: Option<PeripheralRef<'a, AnyPin>>,
    ) -> Self {
        Self {
            inner: slice,
            frequency_hz: clk_sys_freq(),
            phase_correct: false,
            duty_a: None,
            duty_b: None,
            pin_a,
            pin_b,
            div: 1,
            top: u16::MAX as u32 + 1,
        }
    }

    /// Reconfigures the slice and specified channel
    pub(crate) fn reconfigure(
        &mut self,
        channel: Channel,
        freq_hz: u32,
        duty: f32,
        phase_correct: bool,
    ) -> Result<(), PwmError> {
        // Check for changes and assert that the provided channel is configured
        // prior to reconfiguring.
        if !self.reconfigure_precheck(channel.clone(), freq_hz, duty, phase_correct)? {
            return Ok(());
        }

        // Update the phase-correct mode if it has changed. Note that we need to
        // do this first as it affects the TOP value calculation.
        if phase_correct != self.phase_correct {
            debug!(
                "Changing phase-correct mode to {} (from {})",
                phase_correct, self.phase_correct
            );
            self.inner.regs().csr().modify(|w| w.set_ph_correct(phase_correct));
            self.phase_correct = phase_correct;

            // Adjust the TOP value if phase-correct mode has changed. When
            // phase-correct is enabled, this value is the wrap value, otherwise
            // it wrap value is divided by two.
            if phase_correct {
                self.top = self.top * 2;
                self.inner.regs().top().write_value(ChTop(self.top));
            } else {
                self.top = self.top / 2;
                self.inner.regs().top().write_value(ChTop(self.top));
            }
        }

        let (div, top) = self.calculate_div_and_top(freq_hz);

        // If the frequency has changed then we need to recalculate the divider
        // and top values.
        if div != self.div || top != self.top {
            debug!(
                "Changing frequency to {}Hz (from Hz={}, TOP={}, DIV={})",
                freq_hz, self.frequency_hz, self.top, self.div,
            );

            self.div = div;
            self.top = top;

            // Update the DIV register with the new divider value.
            self.inner.regs().div().write_set(|w| {
                w.set_int((div >> 4) as u8);
                w.set_frac((div & 0xF) as u8);
            });
            // Update the TOP register with the new top (wrap) value.
            self.inner.regs().top().write_value(ChTop(top));

            debug!(
                "Frequency changed to {}Hz (TOP={}, DIV={})",
                freq_hz, self.top, self.div
            );
        } else {
            debug!("No changes have been made to the frequency ({}Hz).", freq_hz);
        }

        let compare = (duty / 100f32 * self.top as f32) as u16;
        debug!("Updating compare value for channel {} to {}", channel, compare);

        match channel {
            Channel::A => {
                self.inner.regs().cc().modify(|w| w.set_a(compare));
                self.duty_a = Some(duty);
            }
            Channel::B => {
                self.inner.regs().cc().modify(|w| w.set_b(compare));
                self.duty_b = Some(duty);
            }
        }

        Ok(())
    }

    /// Checks for changes in the requested configuration and asserts that the
    /// provided channel is configured prior to reconfiguring.
    ///
    /// Returns `true` if changes are detected, otherwise `false`.
    /// Will return an error if the specified channel is not configured.
    fn reconfigure_precheck(
        &mut self,
        channel: Channel,
        freq_hz: u32,
        duty: f32,
        phase_correct: bool,
    ) -> Result<bool, PwmError> {
        // Check for changes and assert that the provided channel is configured
        // prior to reconfiguring.
        let (pin, current_duty) = match channel {
            Channel::A => (&self.pin_a, &self.duty_a),
            Channel::B => (&self.pin_b, &self.duty_b),
        };

        if pin.is_none() {
            return Err(PwmError::ChannelNotConfigured(channel));
        }

        if freq_hz == self.frequency_hz && phase_correct == self.phase_correct {
            return Ok(true);
        }

        if let Some(current_duty) = current_duty {
            if duty != *current_duty {
                return Ok(true);
            }
        } else {
            return Ok(true);
        }

        debug!("No changes have been made, skipping reconfiguration.");
        Ok(false)
    }

    /// Calculates the divider and top values for the PWM slice based on the
    /// provided frequency and whether or not phase-correct is enabled for
    /// this slice.
    fn calculate_div_and_top(&mut self, freq_hz: u32) -> (u32, u32) {
        const TOP_MAX: u32 = 65534;
        const DIV_MIN: u32 = (0x01 << 4) + 0x0; // 0x01.0
        const DIV_MAX: u32 = (0xFF << 4) + 0xF; // 0xFF.F

        //let freq_hz = freq_hz / 2;

        let clock = clk_sys_freq();
        let div = (clock << 4) / freq_hz / (TOP_MAX + 1);
        let div = if div < DIV_MIN { DIV_MIN } else { div };
        let mut period = (clock << 4) / div / freq_hz;
        while (period > (TOP_MAX + 1)) && (div <= DIV_MAX) {
            period = (clock << 4) / (div + 1) / freq_hz;
        }

        if period <= 1 {
            panic!("Frequency below is too high ...");
        } else if div > DIV_MAX {
            panic!("Frequency below is too low ...");
        }

        let mut top = period - 1;

        let out = (clock << 4) / div / (top + 1);

        debug!(
            "\nFreq = {}\nTop = {}\nDiv = 0x{:02X}.{:X}\nOut = {}",
            freq_hz,
            top,
            div >> 4,
            div & 0xF,
            out
        );

        if self.phase_correct {
            top = top / 2;
        }

        (div, top)
    }

    /*void SetPwmFreq(float freq) {
    #define TOP_MAX 65534
    #define DIV_MIN ((0x01 << 4) + 0x0) // 0x01.0
    #define DIV_MAX ((0xFF << 4) + 0xF) // 0xFF.F
    uint32_t clock = 125000000;
    // Calculate a div value for frequency desired
    uint32_t div = (clock << 4) / freq / (TOP_MAX + 1);
    if (div < DIV_MIN) {
        div = DIV_MIN;
    }
    // Determine what period that gives us
    uint32_t period = (clock << 4) / div / freq;
    // We may have had a rounding error when calculating div so it may
    // be lower than it should be, which in turn causes the period to
    // be higher than it should be, higher than can be used. In which
    // case we increase the div until the period becomes usable.
    while ((period > (TOP_MAX+1)) && (div <= DIV_MAX)) {
        period = (clock << 4) / ++div / freq;
    }
    // Check if the result is usable
    if (period <= 1) {
        printf("Freq below is too high ...\n");
    } else if (div > DIV_MAX) {
        printf("Freq below is too low ...\n");
    }
    // Determine the top value we will be setting
    uint32_t top = period - 1;
    // Determine what output frequency that will generate
    float out = (float)(clock << 4) / div / (top + 1);
    // Report the results
    printf("Freq = %f\t",         freq);
    printf("Top = %ld\t",         top);
    printf("Div = 0x%02lX.%lX\t", div >> 4, div & 0xF);
    printf("Out = %f\n",          out);
    } */

    /// Sets the duty cycle for this channel. The duty cycle is a percentage
    /// value between 0.0 and 100.0. Defaults to 0.0.
    pub fn set_duty_cycle(&mut self, channel: Channel, duty: f32) -> Result<(), PwmError> {
        self.reconfigure(channel, self.frequency_hz, duty, self.phase_correct)
    }

    /// Sets whether or not the specified channel should be inverted or not.
    pub fn invert(&mut self, channel: Channel, inverted: bool) -> &mut Self {
        match channel {
            Channel::A => {
                self.inner.regs().csr().modify(|w| w.set_a_inv(inverted));
            }
            Channel::B => {
                self.inner.regs().csr().modify(|w| w.set_b_inv(inverted));
            }
        }
        self
    }

    /// Gets whether or not channel A has been configured
    pub fn is_channel_configured(&self, channel: Channel) -> bool {
        match channel {
            Channel::A => self.pin_a.is_some(),
            Channel::B => self.pin_b.is_some(),
        }
    }
}

/// Represents a configured slice in level- or edge-sensitive mode.
#[allow(dead_code)] // TODO: Temporary, to be used in level- and edge-sensitive slices
pub struct PwmInputOutputSlice<'a, T: Slice> {
    inner: PeripheralRef<'a, T>,
    pub(crate) pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pub(crate) pin_b: Option<PeripheralRef<'a, AnyPin>>,
}

#[allow(dead_code)] // TODO: Temporary, to be used in level- and edge-sensitive slices
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
    fn enable(&self) -> &Self {
        self.slice().regs().csr().modify(|w| w.set_en(true));
        self
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

/// Enable multiple PWM slices simultaneously, causing them to start on the
/// same clock cycle.
pub fn enable_pwm_slices(slices: impl FnOnce(&mut SliceMask) -> &mut SliceMask) {
    let mut mask = SliceMask::default();
    slices(&mut mask);
    crate::pac::PWM.en().write_set(|w| w.0 = mask.mask);
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
