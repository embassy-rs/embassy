use core::ops::Div;

use embassy_hal_internal::PeripheralRef;
use embedded_hal_1::pwm::{ErrorKind, ErrorType, SetDutyCycle};

use crate::clocks::clk_sys_freq;
use crate::gpio::AnyPin;
use crate::RegExt;

use super::builder::PwmSliceBuilder;
use super::{Pwm, Slice};

/// Represents a frequency in Hz, KHz, or MHz.
pub enum Frequency {
    /// Frequency in Hz.
    Hz(u32),
    /// Frequency in KHz.
    KHz(f32),
    /// Frequency in MHz.
    MHz(f32),
}

/// Which edge to trigger on in edge-sensitive input mode.
pub enum EdgeSensitivity {
    /// The counter advances with each rising edge of the PWM B pin.
    Rising,
    /// The counter advances with each falling edge of the PWM B pin.
    Falling,
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
            top: u16::MAX,
        }
    }

    /// TODO DUTY CYCLE
    fn reconfigure(&mut self, channel: Channel, freq_hz: u32, duty: f32, phase_correct: bool) -> Result<(), ErrorKind> {
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
    ) -> Result<bool, ErrorKind> {
        // Check for changes and assert that the provided channel is configured
        // prior to reconfiguring.
        match channel {
            Channel::A => {
                if let Some(duty_a) = self.duty_a {
                    if duty_a == duty && freq_hz == self.frequency_hz && phase_correct == self.phase_correct {
                        debug!("No changes have been made, skipping reconfiguration.");
                        return Ok(false);
                    }
                } else {
                    // TODO: Use proper error
                    return Err(ErrorKind::Other);
                }
            }
            Channel::B => {
                if let Some(duty_b) = self.duty_b {
                    if duty_b == duty && freq_hz == self.frequency_hz && phase_correct == self.phase_correct {
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

    /// Calculates the divider and top values for the PWM slice based on the
    /// provided frequency and whether or not phase-correct is enabled for
    /// this slice.
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

        debug!(
            "Changing frequency to {} Hz (from {}), using divider {} and top {}",
            freq_hz, self.frequency_hz, clk_divider, wrap
        );

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
    pub(crate) pin_a: Option<PeripheralRef<'a, AnyPin>>,
    pub(crate) pin_b: Option<PeripheralRef<'a, AnyPin>>,
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
    pub fn builder<'a, T: Slice>(slice: &'a T) -> PwmSliceBuilder<'a, T> {
        PwmSliceBuilder::new(slice)
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

/// PWN channel. Each slice has two channels, A and B.
#[derive(PartialEq, Eq)]
pub enum Channel {
    /// Channel A of a slice.
    A,
    /// Channel B of a slice.
    B,
}

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
