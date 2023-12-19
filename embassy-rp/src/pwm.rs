//! Pulse Width Modulation (PWM)

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use fixed::traits::ToFixed;
use fixed::FixedU16;
use pac::pwm::regs::{ChDiv, Intr};
use pac::pwm::vals::Divmode;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::{pac, peripherals, RegExt};

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

/// PWM input mode.
pub enum InputMode {
    /// Level mode.
    Level,
    /// Rising edge mode.
    RisingEdge,
    /// Falling edge mode.
    FallingEdge,
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

/// PWM driver.
pub struct Pwm<'d, T: Channel> {
    inner: PeripheralRef<'d, T>,
    pin_a: Option<PeripheralRef<'d, AnyPin>>,
    pin_b: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Channel> Pwm<'d, T> {
    fn new_inner(
        inner: impl Peripheral<P = T> + 'd,
        a: Option<PeripheralRef<'d, AnyPin>>,
        b: Option<PeripheralRef<'d, AnyPin>>,
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
        }
        Self {
            inner,
            pin_a: a.into(),
            pin_b: b.into(),
        }
    }

    /// Create PWM driver without any configured pins.
    #[inline]
    pub fn new_free(inner: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        Self::new_inner(inner, None, None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'a' as output.
    #[inline]
    pub fn new_output_a(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl PwmPinA<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a);
        Self::new_inner(inner, Some(a.map_into()), None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'b' pin as output.
    #[inline]
    pub fn new_output_b(
        inner: impl Peripheral<P = T> + 'd,
        b: impl Peripheral<P = impl PwmPinB<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(b);
        Self::new_inner(inner, None, Some(b.map_into()), config, Divmode::DIV)
    }

    /// Create PWM driver with a 'a' and 'b' pins as output.
    #[inline]
    pub fn new_output_ab(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl PwmPinA<T>> + 'd,
        b: impl Peripheral<P = impl PwmPinB<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a, b);
        Self::new_inner(inner, Some(a.map_into()), Some(b.map_into()), config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'b' as input pin.
    #[inline]
    pub fn new_input(
        inner: impl Peripheral<P = T> + 'd,
        b: impl Peripheral<P = impl PwmPinB<T>> + 'd,
        mode: InputMode,
        config: Config,
    ) -> Self {
        into_ref!(b);
        Self::new_inner(inner, None, Some(b.map_into()), config, mode.into())
    }

    /// Create PWM driver with a 'a' and 'b' pins in the desired input mode.
    #[inline]
    pub fn new_output_input(
        inner: impl Peripheral<P = T> + 'd,
        a: impl Peripheral<P = impl PwmPinA<T>> + 'd,
        b: impl Peripheral<P = impl PwmPinB<T>> + 'd,
        mode: InputMode,
        config: Config,
    ) -> Self {
        into_ref!(a, b);
        Self::new_inner(inner, Some(a.map_into()), Some(b.map_into()), config, mode.into())
    }

    /// Set the PWM config.
    pub fn set_config(&mut self, config: &Config) {
        Self::configure(self.inner.regs(), config);
    }

    fn configure(p: pac::pwm::Channel, config: &Config) {
        if config.divider > FixedU16::<fixed::types::extra::U4>::from_bits(0xFF_F) {
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

    #[inline]
    fn bit(&self) -> u32 {
        1 << self.inner.number() as usize
    }
}

/// Batch representation of PWM channels.
pub struct PwmBatch(u32);

impl PwmBatch {
    #[inline]
    /// Enable a PWM channel in this batch.
    pub fn enable(&mut self, pwm: &Pwm<'_, impl Channel>) {
        self.0 |= pwm.bit();
    }

    #[inline]
    /// Enable channels in this batch in a PWM.
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

impl<'d, T: Channel> Drop for Pwm<'d, T> {
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

mod sealed {
    pub trait Channel {}
}

/// PWM Channel.
pub trait Channel: Peripheral<P = Self> + sealed::Channel + Sized + 'static {
    /// Channel number.
    fn number(&self) -> u8;

    /// Channel register block.
    fn regs(&self) -> pac::pwm::Channel {
        pac::PWM.ch(self.number() as _)
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl sealed::Channel for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }
    };
}

channel!(PWM_CH0, 0);
channel!(PWM_CH1, 1);
channel!(PWM_CH2, 2);
channel!(PWM_CH3, 3);
channel!(PWM_CH4, 4);
channel!(PWM_CH5, 5);
channel!(PWM_CH6, 6);
channel!(PWM_CH7, 7);

/// PWM Pin A.
pub trait PwmPinA<T: Channel>: GpioPin {}
/// PWM Pin B.
pub trait PwmPinB<T: Channel>: GpioPin {}

macro_rules! impl_pin {
    ($pin:ident, $channel:ident, $kind:ident) => {
        impl $kind<peripherals::$channel> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, PWM_CH0, PwmPinA);
impl_pin!(PIN_1, PWM_CH0, PwmPinB);
impl_pin!(PIN_2, PWM_CH1, PwmPinA);
impl_pin!(PIN_3, PWM_CH1, PwmPinB);
impl_pin!(PIN_4, PWM_CH2, PwmPinA);
impl_pin!(PIN_5, PWM_CH2, PwmPinB);
impl_pin!(PIN_6, PWM_CH3, PwmPinA);
impl_pin!(PIN_7, PWM_CH3, PwmPinB);
impl_pin!(PIN_8, PWM_CH4, PwmPinA);
impl_pin!(PIN_9, PWM_CH4, PwmPinB);
impl_pin!(PIN_10, PWM_CH5, PwmPinA);
impl_pin!(PIN_11, PWM_CH5, PwmPinB);
impl_pin!(PIN_12, PWM_CH6, PwmPinA);
impl_pin!(PIN_13, PWM_CH6, PwmPinB);
impl_pin!(PIN_14, PWM_CH7, PwmPinA);
impl_pin!(PIN_15, PWM_CH7, PwmPinB);
impl_pin!(PIN_16, PWM_CH0, PwmPinA);
impl_pin!(PIN_17, PWM_CH0, PwmPinB);
impl_pin!(PIN_18, PWM_CH1, PwmPinA);
impl_pin!(PIN_19, PWM_CH1, PwmPinB);
impl_pin!(PIN_20, PWM_CH2, PwmPinA);
impl_pin!(PIN_21, PWM_CH2, PwmPinB);
impl_pin!(PIN_22, PWM_CH3, PwmPinA);
impl_pin!(PIN_23, PWM_CH3, PwmPinB);
impl_pin!(PIN_24, PWM_CH4, PwmPinA);
impl_pin!(PIN_25, PWM_CH4, PwmPinB);
impl_pin!(PIN_26, PWM_CH5, PwmPinA);
impl_pin!(PIN_27, PWM_CH5, PwmPinB);
impl_pin!(PIN_28, PWM_CH6, PwmPinA);
impl_pin!(PIN_29, PWM_CH6, PwmPinB);
