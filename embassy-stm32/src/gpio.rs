//! General-purpose Input/Output (GPIO)

#![macro_use]
use core::convert::Infallible;

use critical_section::CriticalSection;
use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};

use crate::pac::gpio::{self, vals};
use crate::peripherals;

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d> {
    pub(crate) pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    ///
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        // Pin will be in disconnected state.
        Self { pin: pin.into() }
    }

    /// Put the pin into input mode.
    ///
    /// The internal weak pull-up and pull-down resistors will be enabled according to `pull`.
    #[inline(never)]
    pub fn set_as_input(&mut self, pull: Pull) {
        critical_section::with(|_| {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            #[cfg(gpio_v1)]
            {
                let cnf = match pull {
                    Pull::Up => {
                        r.bsrr().write(|w| w.set_bs(n, true));
                        vals::CnfIn::PULL
                    }
                    Pull::Down => {
                        r.bsrr().write(|w| w.set_br(n, true));
                        vals::CnfIn::PULL
                    }
                    Pull::None => vals::CnfIn::FLOATING,
                };

                r.cr(n / 8).modify(|w| {
                    w.set_mode(n % 8, vals::Mode::INPUT);
                    w.set_cnf_in(n % 8, cnf);
                });
            }
            #[cfg(gpio_v2)]
            {
                r.pupdr().modify(|w| w.set_pupdr(n, pull.to_pupdr()));
                r.otyper().modify(|w| w.set_ot(n, vals::Ot::PUSH_PULL));
                r.moder().modify(|w| w.set_moder(n, vals::Moder::INPUT));
            }
        });
    }

    /// Put the pin into push-pull output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    ///
    /// The internal weak pull-up and pull-down resistors will be disabled.
    #[inline(never)]
    pub fn set_as_output(&mut self, speed: Speed) {
        critical_section::with(|_| {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            #[cfg(gpio_v1)]
            {
                r.cr(n / 8).modify(|w| {
                    w.set_mode(n % 8, speed.to_mode());
                    w.set_cnf_out(n % 8, vals::CnfOut::PUSH_PULL);
                });
            }
            #[cfg(gpio_v2)]
            {
                r.pupdr().modify(|w| w.set_pupdr(n, vals::Pupdr::FLOATING));
                r.otyper().modify(|w| w.set_ot(n, vals::Ot::PUSH_PULL));
                r.ospeedr().modify(|w| w.set_ospeedr(n, speed.to_ospeedr()));
                r.moder().modify(|w| w.set_moder(n, vals::Moder::OUTPUT));
            }
        });
    }

    /// Put the pin into input + open-drain output mode.
    ///
    /// The hardware will drive the line low if you set it to low, and will leave it floating if you set
    /// it to high, in which case you can read the input to figure out whether another device
    /// is driving the line low.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    ///
    /// The internal weak pull-up and pull-down resistors will be disabled.
    #[inline(never)]
    pub fn set_as_input_output(&mut self, speed: Speed) {
        #[cfg(gpio_v1)]
        critical_section::with(|_| {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            r.cr(n / 8).modify(|w| w.set_mode(n % 8, speed.to_mode()));
            r.cr(n / 8).modify(|w| w.set_cnf_out(n % 8, vals::CnfOut::OPEN_DRAIN));
        });

        #[cfg(gpio_v2)]
        self.set_as_input_output_pull(speed, Pull::None);
    }

    /// Put the pin into input + open-drain output mode with internal pullup or pulldown.
    ///
    /// This works like [`Self::set_as_input_output()`], but it also allows to enable the internal
    /// weak pull-up or pull-down resistors.
    #[inline(never)]
    #[cfg(gpio_v2)]
    pub fn set_as_input_output_pull(&mut self, speed: Speed, pull: Pull) {
        critical_section::with(|_| {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            r.pupdr().modify(|w| w.set_pupdr(n, pull.to_pupdr()));
            r.otyper().modify(|w| w.set_ot(n, vals::Ot::OPEN_DRAIN));
            r.ospeedr().modify(|w| w.set_ospeedr(n, speed.to_ospeedr()));
            r.moder().modify(|w| w.set_moder(n, vals::Moder::OUTPUT));
        });
    }

    /// Put the pin into analog mode
    ///
    /// This mode is used by ADC and COMP but usually there is no need to set this manually
    /// as the mode change is handled by the driver.
    #[inline]
    pub fn set_as_analog(&mut self) {
        // TODO: does this also need a critical section, like other methods?
        self.pin.set_as_analog();
    }

    /// Put the pin into AF mode, unchecked.
    ///
    /// This puts the pin into the AF mode, with the requested number and AF type. This is
    /// completely unchecked, it can attach the pin to literally any peripheral, so use with care.
    #[inline]
    pub fn set_as_af_unchecked(&mut self, af_num: u8, af_type: AfType) {
        critical_section::with(|_| {
            self.pin.set_as_af(af_num, af_type);
        });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        let state = self.pin.block().idr().read().idr(self.pin.pin() as _);
        state == vals::Idr::LOW
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        let state = self.pin.block().odr().read().odr(self.pin.pin() as _);
        state == vals::Odr::LOW
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.pin.set_low(),
            Level::High => self.pin.set_high(),
        }
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        if self.is_set_low() {
            self.set_high()
        } else {
            self.set_low()
        }
    }
}

impl<'d> Drop for Flex<'d> {
    #[inline]
    fn drop(&mut self) {
        critical_section::with(|_| {
            self.pin.set_as_disconnected();
        });
    }
}

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull
    None,
    /// Pull up
    Up,
    /// Pull down
    Down,
}

impl Pull {
    #[cfg(gpio_v2)]
    const fn to_pupdr(self) -> vals::Pupdr {
        match self {
            Pull::None => vals::Pupdr::FLOATING,
            Pull::Up => vals::Pupdr::PULL_UP,
            Pull::Down => vals::Pupdr::PULL_DOWN,
        }
    }
}

/// Speed setting for an output.
///
/// These vary depending on the chip, check the reference manual and datasheet ("I/O port
/// characteristics") for details.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    #[cfg_attr(gpio_v1, doc = "Output speed OUTPUT2MHZ")]
    #[cfg_attr(gpio_v2, doc = "Output speed 00")]
    Low,
    #[cfg_attr(gpio_v1, doc = "Output speed OUTPUT10MHZ")]
    #[cfg_attr(gpio_v2, doc = "Output speed 01")]
    Medium,
    #[cfg_attr(gpio_v2, doc = "Output speed 10")]
    #[cfg(not(any(gpio_v1, syscfg_f0)))]
    High,
    #[cfg_attr(gpio_v1, doc = "Output speed OUTPUT50MHZ")]
    #[cfg_attr(gpio_v2, doc = "Output speed 11")]
    VeryHigh,
}

impl Speed {
    #[cfg(gpio_v1)]
    const fn to_mode(self) -> vals::Mode {
        match self {
            Speed::Low => vals::Mode::OUTPUT2MHZ,
            Speed::Medium => vals::Mode::OUTPUT10MHZ,
            Speed::VeryHigh => vals::Mode::OUTPUT50MHZ,
        }
    }

    #[cfg(gpio_v2)]
    const fn to_ospeedr(self: Speed) -> vals::Ospeedr {
        match self {
            Speed::Low => vals::Ospeedr::LOW_SPEED,
            Speed::Medium => vals::Ospeedr::MEDIUM_SPEED,
            #[cfg(not(syscfg_f0))]
            Speed::High => vals::Ospeedr::HIGH_SPEED,
            Speed::VeryHigh => vals::Ospeedr::VERY_HIGH_SPEED,
        }
    }
}

/// GPIO input driver.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input(pull);
        Self { pin }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }
}

/// Digital input or output level.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low
    Low,
    /// High
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// GPIO output driver.
///
/// Note that pins will **return to their floating state** when `Output` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `Output`, or pass it to [`core::mem::forget`].
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] and [Speed] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level, speed: Speed) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_output(speed);
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }
}

/// GPIO output open-drain driver.
///
/// Note that pins will **return to their floating state** when `OutputOpenDrain` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `OutputOpenDrain`, or pass it to [`core::mem::forget`].
pub struct OutputOpenDrain<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level] and [Speed].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level, speed: Speed) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_input_output(speed);
        Self { pin }
    }

    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level], [Speed]
    /// and [Pull].
    #[inline]
    #[cfg(gpio_v2)]
    pub fn new_pull(pin: Peri<'d, impl Pin>, initial_output: Level, speed: Speed, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_input_output_pull(speed, pull);
        Self { pin }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.pin.is_low()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }
}

/// GPIO output type
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputType {
    /// Drive the pin both high or low.
    PushPull,
    /// Drive the pin low, or don't drive it at all if the output level is high.
    OpenDrain,
}

impl OutputType {
    #[cfg(gpio_v1)]
    const fn to_cnf_out(self) -> vals::CnfOut {
        match self {
            OutputType::PushPull => vals::CnfOut::ALT_PUSH_PULL,
            OutputType::OpenDrain => vals::CnfOut::ALT_OPEN_DRAIN,
        }
    }

    #[cfg(gpio_v2)]
    const fn to_ot(self) -> vals::Ot {
        match self {
            OutputType::PushPull => vals::Ot::PUSH_PULL,
            OutputType::OpenDrain => vals::Ot::OPEN_DRAIN,
        }
    }
}

/// Alternate function type settings.
#[derive(Copy, Clone)]
#[cfg(gpio_v1)]
pub struct AfType {
    mode: vals::Mode,
    cnf: u8,
    pull: Pull,
}

#[cfg(gpio_v1)]
impl AfType {
    /// Input with optional pullup or pulldown.
    pub const fn input(pull: Pull) -> Self {
        let cnf_in = match pull {
            Pull::Up | Pull::Down => vals::CnfIn::PULL,
            Pull::None => vals::CnfIn::FLOATING,
        };
        Self {
            mode: vals::Mode::INPUT,
            cnf: cnf_in.to_bits(),
            pull,
        }
    }

    /// Output with output type and speed and no pull-up or pull-down.
    pub const fn output(output_type: OutputType, speed: Speed) -> Self {
        Self {
            mode: speed.to_mode(),
            cnf: output_type.to_cnf_out().to_bits(),
            pull: Pull::None,
        }
    }
}

#[inline(never)]
#[cfg(gpio_v1)]
fn set_as_af(pin_port: u8, _af_num: u8, af_type: AfType) {
    let pin = unsafe { AnyPin::steal(pin_port) };
    let r = pin.block();
    let n = pin._pin() as usize;

    r.cr(n / 8).modify(|w| {
        w.set_mode(n % 8, af_type.mode);
        // note that we are writing the CNF field, which is exposed as both `cnf_in` and `cnf_out`
        // in the PAC. the choice of `cnf_in` instead of `cnf_out` in this code is arbitrary and
        // does not affect the result.
        w.set_cnf_in(n % 8, vals::CnfIn::from_bits(af_type.cnf));
    });

    match af_type.pull {
        Pull::Up => r.bsrr().write(|w| w.set_bs(n, true)),
        Pull::Down => r.bsrr().write(|w| w.set_br(n, true)),
        Pull::None => {}
    }
}

/// Alternate function type settings.
#[derive(Copy, Clone)]
#[cfg(gpio_v2)]
pub struct AfType {
    pupdr: vals::Pupdr,
    ot: vals::Ot,
    ospeedr: vals::Ospeedr,
}

#[cfg(gpio_v2)]
impl AfType {
    /// Input with optional pullup or pulldown.
    pub const fn input(pull: Pull) -> Self {
        Self {
            pupdr: pull.to_pupdr(),
            ot: vals::Ot::PUSH_PULL,
            ospeedr: vals::Ospeedr::LOW_SPEED,
        }
    }

    /// Output with output type and speed and no pull-up or pull-down.
    pub const fn output(output_type: OutputType, speed: Speed) -> Self {
        Self::output_pull(output_type, speed, Pull::None)
    }

    /// Output with output type, speed and pull-up or pull-down;
    pub const fn output_pull(output_type: OutputType, speed: Speed, pull: Pull) -> Self {
        Self {
            pupdr: pull.to_pupdr(),
            ot: output_type.to_ot(),
            ospeedr: speed.to_ospeedr(),
        }
    }
}

#[inline(never)]
#[cfg(gpio_v2)]
fn set_as_af(pin_port: u8, af_num: u8, af_type: AfType) {
    let pin = unsafe { AnyPin::steal(pin_port) };
    let r = pin.block();
    let n = pin._pin() as usize;

    r.afr(n / 8).modify(|w| w.set_afr(n % 8, af_num));
    r.pupdr().modify(|w| w.set_pupdr(n, af_type.pupdr));
    r.otyper().modify(|w| w.set_ot(n, af_type.ot));
    r.ospeedr().modify(|w| w.set_ospeedr(n, af_type.ospeedr));
    r.moder().modify(|w| w.set_moder(n, vals::Moder::ALTERNATE));
}

#[inline(never)]
#[cfg(gpio_v2)]
fn set_speed(pin_port: u8, speed: Speed) {
    let pin = unsafe { AnyPin::steal(pin_port) };
    let r = pin.block();
    let n = pin._pin() as usize;

    r.ospeedr().modify(|w| w.set_ospeedr(n, speed.to_ospeedr()));
}

#[inline(never)]
fn set_as_analog(pin_port: u8) {
    let pin = unsafe { AnyPin::steal(pin_port) };
    let r = pin.block();
    let n = pin._pin() as usize;

    #[cfg(gpio_v1)]
    r.cr(n / 8).modify(|w| {
        w.set_mode(n % 8, vals::Mode::INPUT);
        w.set_cnf_in(n % 8, vals::CnfIn::ANALOG);
    });

    #[cfg(gpio_v2)]
    r.moder().modify(|w| w.set_moder(n, vals::Moder::ANALOG));
}

#[inline(never)]
fn get_pull(pin_port: u8) -> Pull {
    let pin = unsafe { AnyPin::steal(pin_port) };
    let r = pin.block();
    let n = pin._pin() as usize;

    #[cfg(gpio_v1)]
    return match r.cr(n / 8).read().mode(n % 8) {
        vals::Mode::INPUT => match r.cr(n / 8).read().cnf_in(n % 8) {
            vals::CnfIn::PULL => match r.odr().read().odr(n) {
                vals::Odr::LOW => Pull::Down,
                vals::Odr::HIGH => Pull::Up,
            },
            _ => Pull::None,
        },
        _ => Pull::None,
    };

    #[cfg(gpio_v2)]
    return match r.pupdr().read().pupdr(n) {
        vals::Pupdr::FLOATING => Pull::None,
        vals::Pupdr::PULL_DOWN => Pull::Down,
        vals::Pupdr::PULL_UP => Pull::Up,
        vals::Pupdr::_RESERVED_3 => Pull::None,
    };
}

pub(crate) trait SealedPin {
    fn pin_port(&self) -> u8;

    #[inline]
    fn _pin(&self) -> u8 {
        self.pin_port() % 16
    }

    #[inline]
    fn _port(&self) -> u8 {
        self.pin_port() / 16
    }

    #[inline]
    fn block(&self) -> gpio::Gpio {
        crate::_generated::gpio_block(self._port() as _)
    }

    /// Set the output as high.
    #[inline]
    fn set_high(&self) {
        let n = self._pin() as _;
        self.block().bsrr().write(|w| w.set_bs(n, true));
    }

    /// Set the output as low.
    #[inline]
    fn set_low(&self) {
        let n = self._pin() as _;
        self.block().bsrr().write(|w| w.set_br(n, true));
    }

    #[inline]
    fn set_as_af(&self, af_num: u8, af_type: AfType) {
        set_as_af(self.pin_port(), af_num, af_type)
    }

    #[inline]
    #[cfg(gpio_v2)]
    fn set_speed(&self, speed: Speed) {
        set_speed(self.pin_port(), speed)
    }

    #[inline]
    fn set_as_analog(&self) {
        set_as_analog(self.pin_port());
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the internal weak pull-up and pull-down resistors.
    #[inline]
    fn set_as_disconnected(&self) {
        self.set_as_analog();
    }

    /// Get the pull-up configuration.
    #[inline]
    fn pull(&self) -> Pull {
        critical_section::with(|_| get_pull(self.pin_port()))
    }
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// EXTI channel assigned to this pin.
    ///
    /// For example, PC4 uses EXTI4.
    #[cfg(feature = "exti")]
    type ExtiChannel: crate::exti::Channel;

    /// Number of the pin within the port (0..31)
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Port of the pin
    #[inline]
    fn port(&self) -> u8 {
        self._port()
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pin_port: u8,
}

impl AnyPin {
    /// Unsafely create an `AnyPin` from a pin+port number.
    ///
    /// `pin_port` is `port_num * 16 + pin_num`, where `port_num` is 0 for port `A`, 1 for port `B`, etc...
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }

    #[inline]
    fn _port(&self) -> u8 {
        self.pin_port / 16
    }

    /// Get the GPIO register block for this pin.
    #[cfg(feature = "unstable-pac")]
    #[inline]
    pub fn block(&self) -> gpio::Gpio {
        crate::_generated::gpio_block(self._port() as _)
    }
}

impl_peripheral!(AnyPin);
impl Pin for AnyPin {
    #[cfg(feature = "exti")]
    type ExtiChannel = crate::exti::AnyChannel;
}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

// ====================

foreach_pin!(
    ($pin_name:ident, $port_name:ident, $port_num:expr, $pin_num:expr, $exti_ch:ident) => {
        impl Pin for peripherals::$pin_name {
            #[cfg(feature = "exti")]
            type ExtiChannel = peripherals::$exti_ch;
        }
        impl SealedPin for peripherals::$pin_name {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 16 + $pin_num
            }
        }

        impl From<peripherals::$pin_name> for AnyPin {
            fn from(val: peripherals::$pin_name) -> Self {
                Self {
                    pin_port: val.pin_port(),
                }
            }
        }
    };
);

pub(crate) unsafe fn init(_cs: CriticalSection) {
    #[cfg(afio)]
    crate::rcc::enable_and_reset_with_cs::<crate::peripherals::AFIO>(_cs);

    crate::_generated::init_gpio();
}

impl<'d> embedded_hal_02::digital::v2::InputPin for Input<'d> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::OutputPin for Output<'d> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'d> {
    type Error = Infallible;
    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<'d> embedded_hal_02::digital::v2::InputPin for OutputOpenDrain<'d> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::OutputPin for OutputOpenDrain<'d> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for OutputOpenDrain<'d> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for OutputOpenDrain<'d> {
    type Error = Infallible;
    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<'d> embedded_hal_02::digital::v2::InputPin for Flex<'d> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::OutputPin for Flex<'d> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d> {
    type Error = Infallible;
    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::InputPin for Input<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::OutputPin for Output<'d> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Output<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for OutputOpenDrain<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::InputPin for OutputOpenDrain<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::OutputPin for OutputOpenDrain<'d> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for OutputOpenDrain<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_1::digital::InputPin for Flex<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::OutputPin for Flex<'d> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Flex<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Flex<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}
