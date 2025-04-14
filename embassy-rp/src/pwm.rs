//! Pulse Width Modulation (PWM)

use embassy_hal_internal::{Peri, PeripheralType};
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};
use fixed::traits::ToFixed;
use fixed::FixedU16;
use pac::pwm::regs::{ChDiv, Intr};
use pac::pwm::vals::Divmode;

use crate::gpio::{AnyPin, Pin as GpioPin, Pull, SealedPin as _};
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

/// PWM error.
#[derive(Debug)]
pub enum PwmError {
    /// Invalid Duty Cycle.
    InvalidDutyCycle,
}

impl Error for PwmError {
    fn kind(&self) -> ErrorKind {
        match self {
            PwmError::InvalidDutyCycle => ErrorKind::Other,
        }
    }
}

/// PWM driver.
pub struct Pwm<'d> {
    pin_a: Option<Peri<'d, AnyPin>>,
    pin_b: Option<Peri<'d, AnyPin>>,
    slice: usize,
}

impl<'d> ErrorType for Pwm<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for Pwm<'d> {
    fn max_duty_cycle(&self) -> u16 {
        pac::PWM.ch(self.slice).top().read().top()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let max_duty = self.max_duty_cycle();
        if duty > max_duty {
            return Err(PwmError::InvalidDutyCycle);
        }

        let p = pac::PWM.ch(self.slice);
        p.cc().modify(|w| {
            w.set_a(duty);
            w.set_b(duty);
        });
        Ok(())
    }
}

impl<'d> Pwm<'d> {
    fn new_inner(
        slice: usize,
        a: Option<Peri<'d, AnyPin>>,
        b: Option<Peri<'d, AnyPin>>,
        b_pull: Pull,
        config: Config,
        divmode: Divmode,
    ) -> Self {
        let p = pac::PWM.ch(slice);
        p.csr().modify(|w| {
            w.set_divmode(divmode);
            w.set_en(false);
        });
        p.ctr().write(|w| w.0 = 0);
        Self::configure(p, &config);

        if let Some(pin) = &a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
            #[cfg(feature = "_rp235x")]
            pin.pad_ctrl().modify(|w| {
                w.set_iso(false);
            });
        }
        if let Some(pin) = &b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(4));
            pin.pad_ctrl().modify(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                #[cfg(feature = "_rp235x")]
                if divmode != Divmode::DIV {
                    // Is in input mode and so must enable input mode for the pin
                    w.set_ie(true);
                }
                w.set_pue(b_pull == Pull::Up);
                w.set_pde(b_pull == Pull::Down);
            });
        }
        Self {
            // inner: p.into(),
            pin_a: a,
            pin_b: b,
            slice,
        }
    }

    /// Create PWM driver without any configured pins.
    #[inline]
    pub fn new_free<T: Slice>(slice: Peri<'d, T>, config: Config) -> Self {
        Self::new_inner(slice.number(), None, None, Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'a' pin as output.
    #[inline]
    pub fn new_output_a<T: Slice>(slice: Peri<'d, T>, a: Peri<'d, impl ChannelAPin<T>>, config: Config) -> Self {
        Self::new_inner(slice.number(), Some(a.into()), None, Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a single 'b' pin as output.
    #[inline]
    pub fn new_output_b<T: Slice>(slice: Peri<'d, T>, b: Peri<'d, impl ChannelBPin<T>>, config: Config) -> Self {
        Self::new_inner(slice.number(), None, Some(b.into()), Pull::None, config, Divmode::DIV)
    }

    /// Create PWM driver with a 'a' and 'b' pins as output.
    #[inline]
    pub fn new_output_ab<T: Slice>(
        slice: Peri<'d, T>,
        a: Peri<'d, impl ChannelAPin<T>>,
        b: Peri<'d, impl ChannelBPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            slice.number(),
            Some(a.into()),
            Some(b.into()),
            Pull::None,
            config,
            Divmode::DIV,
        )
    }

    /// Create PWM driver with a single 'b' as input pin.
    #[inline]
    pub fn new_input<T: Slice>(
        slice: Peri<'d, T>,
        b: Peri<'d, impl ChannelBPin<T>>,
        b_pull: Pull,
        mode: InputMode,
        config: Config,
    ) -> Self {
        Self::new_inner(slice.number(), None, Some(b.into()), b_pull, config, mode.into())
    }

    /// Create PWM driver with a 'a' and 'b' pins in the desired input mode.
    #[inline]
    pub fn new_output_input<T: Slice>(
        slice: Peri<'d, T>,
        a: Peri<'d, impl ChannelAPin<T>>,
        b: Peri<'d, impl ChannelBPin<T>>,
        b_pull: Pull,
        mode: InputMode,
        config: Config,
    ) -> Self {
        Self::new_inner(
            slice.number(),
            Some(a.into()),
            Some(b.into()),
            b_pull,
            config,
            mode.into(),
        )
    }

    /// Set the PWM config.
    pub fn set_config(&mut self, config: &Config) {
        Self::configure(pac::PWM.ch(self.slice), config);
    }

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

    /// Advances a slice's output phase by one count while it is running
    /// by inserting a pulse into the clock enable. The counter
    /// will not count faster than once per cycle.
    #[inline]
    pub fn phase_advance(&mut self) {
        let p = pac::PWM.ch(self.slice);
        p.csr().write_set(|w| w.set_ph_adv(true));
        while p.csr().read().ph_adv() {}
    }

    /// Retards a slice's output phase by one count while it is running
    /// by deleting a pulse from the clock enable. The counter will not
    /// count backward when clock enable is permanently low.
    #[inline]
    pub fn phase_retard(&mut self) {
        let p = pac::PWM.ch(self.slice);
        p.csr().write_set(|w| w.set_ph_ret(true));
        while p.csr().read().ph_ret() {}
    }

    /// Read PWM counter.
    #[inline]
    pub fn counter(&self) -> u16 {
        pac::PWM.ch(self.slice).ctr().read().ctr()
    }

    /// Write PWM counter.
    #[inline]
    pub fn set_counter(&self, ctr: u16) {
        pac::PWM.ch(self.slice).ctr().write(|w| w.set_ctr(ctr))
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
        1 << self.slice as usize
    }

    /// Splits the PWM driver into separate `PwmOutput` instances for channels A and B.
    #[inline]
    pub fn split(mut self) -> (Option<PwmOutput<'d>>, Option<PwmOutput<'d>>) {
        (
            self.pin_a
                .take()
                .map(|pin| PwmOutput::new(PwmChannelPin::A(pin), self.slice.clone(), true)),
            self.pin_b
                .take()
                .map(|pin| PwmOutput::new(PwmChannelPin::B(pin), self.slice.clone(), true)),
        )
    }
    /// Splits the PWM driver by reference to allow for separate duty cycle control
    /// of each channel (A and B) without taking ownership of the PWM instance.
    #[inline]
    pub fn split_by_ref(&mut self) -> (Option<PwmOutput<'_>>, Option<PwmOutput<'_>>) {
        (
            self.pin_a
                .as_mut()
                .map(|pin| PwmOutput::new(PwmChannelPin::A(pin.reborrow()), self.slice.clone(), false)),
            self.pin_b
                .as_mut()
                .map(|pin| PwmOutput::new(PwmChannelPin::B(pin.reborrow()), self.slice.clone(), false)),
        )
    }
}

enum PwmChannelPin<'d> {
    A(Peri<'d, AnyPin>),
    B(Peri<'d, AnyPin>),
}

/// Single channel of Pwm driver.
pub struct PwmOutput<'d> {
    //pin that can be ether ChannelAPin or ChannelBPin
    channel_pin: PwmChannelPin<'d>,
    slice: usize,
    is_owned: bool,
}

impl<'d> PwmOutput<'d> {
    fn new(channel_pin: PwmChannelPin<'d>, slice: usize, is_owned: bool) -> Self {
        Self {
            channel_pin,
            slice,
            is_owned,
        }
    }
}

impl<'d> Drop for PwmOutput<'d> {
    fn drop(&mut self) {
        if self.is_owned {
            let p = pac::PWM.ch(self.slice);
            match &self.channel_pin {
                PwmChannelPin::A(pin) => {
                    p.cc().modify(|w| {
                        w.set_a(0);
                    });

                    pin.gpio().ctrl().write(|w| w.set_funcsel(31));
                    //Enable pin PULL-DOWN
                    pin.pad_ctrl().modify(|w| {
                        w.set_pde(true);
                    });
                }
                PwmChannelPin::B(pin) => {
                    p.cc().modify(|w| {
                        w.set_b(0);
                    });
                    pin.gpio().ctrl().write(|w| w.set_funcsel(31));
                    //Enable pin PULL-DOWN
                    pin.pad_ctrl().modify(|w| {
                        w.set_pde(true);
                    });
                }
            }
        }
    }
}

impl<'d> ErrorType for PwmOutput<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for PwmOutput<'d> {
    fn max_duty_cycle(&self) -> u16 {
        pac::PWM.ch(self.slice).top().read().top()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let max_duty = self.max_duty_cycle();
        if duty > max_duty {
            return Err(PwmError::InvalidDutyCycle);
        }

        let p = pac::PWM.ch(self.slice);
        match self.channel_pin {
            PwmChannelPin::A(_) => {
                p.cc().modify(|w| {
                    w.set_a(duty);
                });
            }
            PwmChannelPin::B(_) => {
                p.cc().modify(|w| {
                    w.set_b(duty);
                });
            }
        }

        Ok(())
    }
}

/// Batch representation of PWM slices.
pub struct PwmBatch(u32);

impl PwmBatch {
    #[inline]
    /// Enable a PWM slice in this batch.
    pub fn enable(&mut self, pwm: &Pwm<'_>) {
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

impl<'d> Drop for Pwm<'d> {
    fn drop(&mut self) {
        pac::PWM.ch(self.slice).csr().write_clear(|w| w.set_en(false));
        if let Some(pin) = &self.pin_a {
            pin.gpio().ctrl().write(|w| w.set_funcsel(31));
        }
        if let Some(pin) = &self.pin_b {
            pin.gpio().ctrl().write(|w| w.set_funcsel(31));
            #[cfg(feature = "_rp235x")]
            // Disable input mode. Only pin_b can be input, so not needed for pin_a
            pin.pad_ctrl().modify(|w| {
                w.set_ie(false);
            });
        }
    }
}

trait SealedSlice {}

/// PWM Slice.
#[allow(private_bounds)]
pub trait Slice: PeripheralType + SealedSlice + Sized + 'static {
    /// Slice number.
    fn number(&self) -> usize;
}

macro_rules! slice {
    ($name:ident, $num:expr) => {
        impl SealedSlice for peripherals::$name {}
        impl Slice for peripherals::$name {
            fn number(&self) -> usize {
                $num
            }
        }
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

#[cfg(feature = "_rp235x")]
slice!(PWM_SLICE8, 8);
#[cfg(feature = "_rp235x")]
slice!(PWM_SLICE9, 9);
#[cfg(feature = "_rp235x")]
slice!(PWM_SLICE10, 10);
#[cfg(feature = "_rp235x")]
slice!(PWM_SLICE11, 11);

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
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_30, PWM_SLICE7, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_31, PWM_SLICE7, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_32, PWM_SLICE8, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_33, PWM_SLICE8, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_34, PWM_SLICE9, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_35, PWM_SLICE9, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_36, PWM_SLICE10, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_37, PWM_SLICE10, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_38, PWM_SLICE11, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_39, PWM_SLICE11, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_40, PWM_SLICE8, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_41, PWM_SLICE8, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, PWM_SLICE9, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, PWM_SLICE9, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_44, PWM_SLICE10, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_45, PWM_SLICE10, ChannelBPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, PWM_SLICE11, ChannelAPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, PWM_SLICE11, ChannelBPin);
