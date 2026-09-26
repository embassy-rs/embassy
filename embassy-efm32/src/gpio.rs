//! General purpose input/output (GPIO) driver.
//!
//! ## Hardware notes
//!
//! EFM32's `GPIO` peripheral registers are laid out per-port (`PA_MODEL`, `PB_MODEL`, ...)
//! rather than as an array of identical port blocks, so most of the low level helpers below
//! dispatch on [`Port`] with a small macro instead of simple pointer arithmetic.
//!
//! Unlike MCUs with separate direction/pull/type registers, EFM32 encodes a pin's direction,
//! pull and drive configuration into a single 4-bit `MODEn` field per pin (`DISABLED`, `INPUT`,
//! `INPUTPULL`, `PUSHPULL`, `WIREDAND` (open-drain), `WIREDOR` (open-source), ...). For
//! `INPUTPULL`, the pull direction (up or down) is taken from the pin's `DOUT` bit.
//!
//! ## Waiting for edges
//!
//! [`Input`], [`OutputOpenDrain`] and [`Flex`] can wait for levels and edges (`wait_for_*`, and
//! `embedded_hal_async::digital::Wait`), using the GPIO external interrupts. There's one external
//! interrupt line per pin number, shared by all ports, so only one pin per pin number can wait at a
//! time: e.g. PA3 and PB3 can't both wait at once, while PA3 and PB4 can. Waiting on a second pin
//! with the same number panics.
//!
//! Waiting needs the `rt` feature, which defines the `GPIO_EVEN`/`GPIO_ODD` interrupt handlers.
#![macro_use]

use core::convert::Infallible;
#[cfg(feature = "rt")]
use core::future::Future;
#[cfg(feature = "rt")]
use core::pin::Pin as FuturePin;
#[cfg(feature = "rt")]
use core::sync::atomic::{AtomicU16, Ordering};
#[cfg(feature = "rt")]
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
#[cfg(feature = "rt")]
use embassy_sync::waitqueue::AtomicWaker;

use crate::pac;
#[cfg(feature = "rt")]
use crate::{interrupt, interrupt::InterruptExt};

/// GPIO port.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Port {
    /// Port A.
    A,
    /// Port B.
    B,
    /// Port C.
    C,
    /// Port D.
    D,
    /// Port E.
    E,
    /// Port F.
    F,
}

/// Pin's mode, as encoded by the `MODEn` field of the port's `MODEL`/`MODEH` register.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Mode {
    Disabled,
    Input,
    InputPull,
    PushPull,
    WiredAnd,
    WiredOr,
}

/// Pull setting for an input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// Digital input or output level.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logical low.
    Low,
    /// Logical high.
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
        matches!(level, Level::High)
    }
}

#[inline]
fn regs() -> &'static pac::gpio::RegisterBlock {
    unsafe { &*pac::GPIO::ptr() }
}

macro_rules! apply_mode {
    ($w:expr, $accessor:ident, $mode:expr) => {
        match $mode {
            Mode::Disabled => {
                $w.$accessor().disabled();
            }
            Mode::Input => {
                $w.$accessor().input();
            }
            Mode::InputPull => {
                $w.$accessor().inputpull();
            }
            Mode::PushPull => {
                $w.$accessor().pushpull();
            }
            Mode::WiredAnd => {
                $w.$accessor().wiredand();
            }
            Mode::WiredOr => {
                $w.$accessor().wiredor();
            }
        }
    };
}

fn set_mode(port: Port, pin: u8, mode: Mode) {
    let r = regs();

    // `MODEL` has `MODE0..MODE7` (for pins 0..=7) and `MODEH` has `MODE8..MODE15` (for pins
    // 8..=15) -- the field names are not reused between the two registers, so each register gets
    // its own macro with only the arms that exist for it (a single shared 0..=15 match wouldn't
    // compile: match arms are type-checked even for pin values the caller never actually passes
    // to that register).
    macro_rules! set_in_low {
        ($reg:ident) => {
            r.$reg.modify(|_, w| {
                match pin {
                    0 => apply_mode!(w, mode0, mode),
                    1 => apply_mode!(w, mode1, mode),
                    2 => apply_mode!(w, mode2, mode),
                    3 => apply_mode!(w, mode3, mode),
                    4 => apply_mode!(w, mode4, mode),
                    5 => apply_mode!(w, mode5, mode),
                    6 => apply_mode!(w, mode6, mode),
                    _ => apply_mode!(w, mode7, mode),
                }
                w
            })
        };
    }
    macro_rules! set_in_high {
        ($reg:ident) => {
            r.$reg.modify(|_, w| {
                match pin {
                    8 => apply_mode!(w, mode8, mode),
                    9 => apply_mode!(w, mode9, mode),
                    10 => apply_mode!(w, mode10, mode),
                    11 => apply_mode!(w, mode11, mode),
                    12 => apply_mode!(w, mode12, mode),
                    13 => apply_mode!(w, mode13, mode),
                    14 => apply_mode!(w, mode14, mode),
                    _ => apply_mode!(w, mode15, mode),
                }
                w
            })
        };
    }

    match (port, pin < 8) {
        (Port::A, true) => set_in_low!(pa_model),
        (Port::A, false) => set_in_high!(pa_modeh),
        (Port::B, true) => set_in_low!(pb_model),
        (Port::B, false) => set_in_high!(pb_modeh),
        (Port::C, true) => set_in_low!(pc_model),
        (Port::C, false) => set_in_high!(pc_modeh),
        (Port::D, true) => set_in_low!(pd_model),
        (Port::D, false) => set_in_high!(pd_modeh),
        (Port::E, true) => set_in_low!(pe_model),
        (Port::E, false) => set_in_high!(pe_modeh),
        (Port::F, true) => set_in_low!(pf_model),
        (Port::F, false) => set_in_high!(pf_modeh),
    }
}

fn dout_set(port: Port, pin: u8) {
    let r = regs();
    let mask: u16 = 1 << pin;
    match port {
        Port::A => r.pa_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
        Port::B => r.pb_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
        Port::C => r.pc_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
        Port::D => r.pd_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
        Port::E => r.pe_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
        Port::F => r.pf_doutset.write(|w| unsafe { w.doutset().bits(mask) }),
    }
}

fn dout_clr(port: Port, pin: u8) {
    let r = regs();
    let mask: u16 = 1 << pin;
    match port {
        Port::A => r.pa_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
        Port::B => r.pb_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
        Port::C => r.pc_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
        Port::D => r.pd_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
        Port::E => r.pe_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
        Port::F => r.pf_doutclr.write(|w| unsafe { w.doutclr().bits(mask) }),
    }
}

fn dout_tgl(port: Port, pin: u8) {
    let r = regs();
    let mask: u16 = 1 << pin;
    match port {
        Port::A => r.pa_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
        Port::B => r.pb_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
        Port::C => r.pc_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
        Port::D => r.pd_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
        Port::E => r.pe_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
        Port::F => r.pf_douttgl.write(|w| unsafe { w.douttgl().bits(mask) }),
    }
}

fn din(port: Port, pin: u8) -> bool {
    let r = regs();
    let mask: u16 = 1 << pin;
    let bits = match port {
        Port::A => r.pa_din.read().din().bits(),
        Port::B => r.pb_din.read().din().bits(),
        Port::C => r.pc_din.read().din().bits(),
        Port::D => r.pd_din.read().din().bits(),
        Port::E => r.pe_din.read().din().bits(),
        Port::F => r.pf_din.read().din().bits(),
    };
    bits & mask != 0
}

fn dout(port: Port, pin: u8) -> bool {
    let r = regs();
    let mask: u16 = 1 << pin;
    let bits = match port {
        Port::A => r.pa_dout.read().dout().bits(),
        Port::B => r.pb_dout.read().dout().bits(),
        Port::C => r.pc_dout.read().dout().bits(),
        Port::D => r.pd_dout.read().dout().bits(),
        Port::E => r.pe_dout.read().dout().bits(),
        Port::F => r.pf_dout.read().dout().bits(),
    };
    bits & mask != 0
}

/// What a pin's `MODEn` field configures it as.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ModeKind {
    Disabled,
    Input,
    /// Push-pull, open-drain (`WIREDAND*`) or open-source (`WIREDOR*`), any drive strength.
    Output,
}

fn mode_kind(port: Port, pin: u8) -> ModeKind {
    let r = regs();

    macro_rules! kind {
        ($field:expr) => {{
            let f = $field;
            if f.is_disabled() {
                ModeKind::Disabled
            } else if f.is_input() || f.is_inputpull() || f.is_inputpullfilter() {
                ModeKind::Input
            } else {
                ModeKind::Output
            }
        }};
    }
    // See `set_mode` for why these are two macros.
    macro_rules! get_in_low {
        ($reg:ident) => {{
            let v = r.$reg.read();
            match pin {
                0 => kind!(v.mode0()),
                1 => kind!(v.mode1()),
                2 => kind!(v.mode2()),
                3 => kind!(v.mode3()),
                4 => kind!(v.mode4()),
                5 => kind!(v.mode5()),
                6 => kind!(v.mode6()),
                _ => kind!(v.mode7()),
            }
        }};
    }
    macro_rules! get_in_high {
        ($reg:ident) => {{
            let v = r.$reg.read();
            match pin {
                8 => kind!(v.mode8()),
                9 => kind!(v.mode9()),
                10 => kind!(v.mode10()),
                11 => kind!(v.mode11()),
                12 => kind!(v.mode12()),
                13 => kind!(v.mode13()),
                14 => kind!(v.mode14()),
                _ => kind!(v.mode15()),
            }
        }};
    }

    match (port, pin < 8) {
        (Port::A, true) => get_in_low!(pa_model),
        (Port::A, false) => get_in_high!(pa_modeh),
        (Port::B, true) => get_in_low!(pb_model),
        (Port::B, false) => get_in_high!(pb_modeh),
        (Port::C, true) => get_in_low!(pc_model),
        (Port::C, false) => get_in_high!(pc_modeh),
        (Port::D, true) => get_in_low!(pd_model),
        (Port::D, false) => get_in_high!(pd_modeh),
        (Port::E, true) => get_in_low!(pe_model),
        (Port::E, false) => get_in_high!(pe_modeh),
        (Port::F, true) => get_in_low!(pf_model),
        (Port::F, false) => get_in_high!(pf_modeh),
    }
}

/// GPIO output driver. Internally, this is a specialized [Flex] pin.
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create a GPIO output driver for a [Pin] with the provided initial output [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_output();
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
        self.pin.set_level(level);
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.pin.output_is_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn output_is_low(&self) -> bool {
        self.pin.output_is_low()
    }

    /// What level output is set to?
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }
}

/// GPIO output open-drain driver (`WIREDAND` mode). Internally, this is a specialized [Flex] pin.
///
/// The pin's input buffer stays enabled, so the actual level of the line can be read, and waited
/// for, too.
pub struct OutputOpenDrain<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a GPIO open-drain output driver for a [Pin] with the provided initial output
    /// [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_output_open_drain();
        Self { pin }
    }

    /// Set the output as high (released, pulled up externally).
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low (driven low).
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.pin.output_is_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn output_is_low(&self) -> bool {
        self.pin.output_is_low()
    }

    /// What level output is set to?
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
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
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Wait until the pin is high. Returns immediately if already high.
    ///
    /// See the [module docs](self) for the limits on waiting on several pins at once.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. Returns immediately if already low.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// GPIO input driver. Internally, this is a specialized [Flex] pin.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a [Pin] with the provided [Pull].
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
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Wait until the pin is high. Returns immediately if already high.
    ///
    /// See the [module docs](self) for the limits on waiting on several pins at once.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. Returns immediately if already low.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition.
    #[cfg(feature = "rt")]
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// GPIO flexible pin driver: a pin whose mode can be reconfigured at runtime.
///
/// Under the hood, this is a reference to a type-erased pin called [`AnyPin`].
pub struct Flex<'d> {
    pub(crate) pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin keeps its current mode, which is disconnected unless it was configured before.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self { pin: pin.into() }
    }

    /// The [`Port`] this pin belongs to.
    #[inline]
    pub fn port(&self) -> Port {
        self.pin.pin_port()
    }

    /// The pin number within its port (0..=15).
    #[inline]
    pub fn pin(&self) -> u8 {
        self.pin.pin_number()
    }

    /// Put the pin into disconnected mode (the high-impedance `DISABLED` mode, with the input
    /// buffer off).
    ///
    /// This also sets the output level (`DOUT`) low, since a high `DOUT` would enable the pull-up
    /// in this mode.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.set_as_disconnected();
    }

    /// Put the pin into input mode with the given [Pull] configuration.
    ///
    /// On EFM32 the pull direction is selected with the output level (`DOUT`), so this overwrites
    /// it: high for [`Pull::Up`], low otherwise.
    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        self.pin.set_as_input(pull);
    }

    /// Put the pin into push-pull output mode.
    #[inline]
    pub fn set_as_output(&mut self) {
        self.pin.set_as_output();
    }

    /// Put the pin into open-drain output mode (`WIREDAND`).
    #[inline]
    pub fn set_as_output_open_drain(&mut self) {
        set_mode(self.port(), self.pin(), Mode::WiredAnd);
    }

    /// Put the pin into open-source output mode (`WIREDOR`).
    #[inline]
    pub fn set_as_output_open_source(&mut self) {
        set_mode(self.port(), self.pin(), Mode::WiredOr);
    }

    /// Is the pin configured as an input?
    #[inline]
    pub fn is_input(&self) -> bool {
        mode_kind(self.port(), self.pin()) == ModeKind::Input
    }

    /// Is the pin configured as an output (push-pull, open-drain or open-source)?
    #[inline]
    pub fn is_output(&self) -> bool {
        mode_kind(self.port(), self.pin()) == ModeKind::Output
    }

    /// Is the pin disconnected?
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        mode_kind(self.port(), self.pin()) == ModeKind::Disabled
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
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        dout_tgl(self.port(), self.pin());
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        din(self.port(), self.pin())
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn output_is_high(&self) -> bool {
        dout(self.port(), self.pin())
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn output_is_low(&self) -> bool {
        !self.output_is_high()
    }

    /// What level output is set to?
    #[inline]
    pub fn output_level(&self) -> Level {
        self.output_is_high().into()
    }

    /// Wait until the pin is high. Returns immediately if already high.
    ///
    /// The pin must be in an input mode (or an output mode, whose input buffer is also enabled).
    /// See the [module docs](self) for the limits on waiting on several pins at once.
    #[cfg(feature = "rt")]
    pub async fn wait_for_high(&mut self) {
        // Arm before checking the level, so a rising edge in between isn't missed.
        let fut = ExtiFuture::new(self.port(), self.pin(), true, false);
        if self.is_high() {
            return;
        }
        fut.await
    }

    /// Wait until the pin is low. Returns immediately if already low.
    #[cfg(feature = "rt")]
    pub async fn wait_for_low(&mut self) {
        let fut = ExtiFuture::new(self.port(), self.pin(), false, true);
        if self.is_low() {
            return;
        }
        fut.await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[cfg(feature = "rt")]
    pub async fn wait_for_rising_edge(&mut self) {
        ExtiFuture::new(self.port(), self.pin(), true, false).await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[cfg(feature = "rt")]
    pub async fn wait_for_falling_edge(&mut self) {
        ExtiFuture::new(self.port(), self.pin(), false, true).await
    }

    /// Wait for the pin to undergo any transition.
    #[cfg(feature = "rt")]
    pub async fn wait_for_any_edge(&mut self) {
        ExtiFuture::new(self.port(), self.pin(), true, true).await
    }
}

impl<'d> Drop for Flex<'d> {
    /// Returns the pin to its disconnected/reset state.
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

// External (edge) interrupts.
//
// The GPIO peripheral has 16 external interrupt lines, one per pin number: line `n` can be routed
// (`EXTIPSELL`/`EXTIPSELH`) to pin `n` of any one port. `GPIO_EVEN` handles the even lines,
// `GPIO_ODD` the odd ones. A waiting future claims its line (`EXTI_CLAIMED`) and arms it (`IEN`);
// the interrupt handler masks the line again and wakes the future, which takes the masked line as
// its signal that the edge happened. The line stays claimed until the future is dropped, so no
// other pin can re-arm it in between (which would hide the edge from the first future).

#[cfg(feature = "rt")]
const EXTI_LINES: usize = 16;

#[cfg(feature = "rt")]
static EXTI_WAKERS: [AtomicWaker; EXTI_LINES] = [const { AtomicWaker::new() }; EXTI_LINES];

/// Lines owned by a live `ExtiFuture`, one bit per line. Only accessed in a critical section.
#[cfg(feature = "rt")]
static EXTI_CLAIMED: AtomicU16 = AtomicU16::new(0);

/// Reset the external interrupt state, and enable the GPIO interrupts with the given priority.
pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    let r = regs();
    // Don't assume reset state: a bootloader may have left lines armed.
    r.ien.write(|w| unsafe { w.ext().bits(0) });
    r.extirise.write(|w| unsafe { w.extirise().bits(0) });
    r.extifall.write(|w| unsafe { w.extifall().bits(0) });
    r.ifc.write(|w| unsafe { w.ext().bits(0xFFFF) });
    r.insense.modify(|_, w| w.int().set_bit());

    // Hand the SWD pins (PF0 = SWCLK, PF1 = SWDIO) over to the GPIO, disconnected.
    #[cfg(feature = "swd-as-gpio")]
    {
        r.route.modify(|_, w| w.swclkpen().clear_bit().swdiopen().clear_bit());
        for pin_number in [0, 1] {
            AnyPin {
                pin_port: Port::F,
                pin_number,
            }
            .set_as_disconnected();
        }
    }
    // The LFXO crystal pins (PB7, PB8): disconnected, like the other pins at reset. `cmu` has
    // stopped the LFXO, which would otherwise keep driving them.
    #[cfg(feature = "lfxo-as-gpio")]
    for pin_number in [7, 8] {
        AnyPin {
            pin_port: Port::B,
            pin_number,
        }
        .set_as_disconnected();
    }

    #[cfg(feature = "rt")]
    {
        interrupt::GPIO_EVEN.set_priority(irq_prio);
        interrupt::GPIO_ODD.set_priority(irq_prio);
        unsafe {
            interrupt::GPIO_EVEN.enable();
            interrupt::GPIO_ODD.enable();
        }
    }
    #[cfg(not(feature = "rt"))]
    let _ = irq_prio;
}

#[cfg(feature = "rt")]
fn on_interrupt(lines: u16) {
    let r = regs();
    // In a critical section: `ExtiFuture` may modify `IEN` from a higher priority context.
    let pending = critical_section::with(|_| {
        let pending = r.if_.read().ext().bits() & r.ien.read().ext().bits() & lines;
        // Masking the line tells the future its edge happened (see `ExtiFuture::poll`).
        r.ien.modify(|r, w| unsafe { w.ext().bits(r.ext().bits() & !pending) });
        r.ifc.write(|w| unsafe { w.ext().bits(pending) });
        pending
    });
    for (line, waker) in EXTI_WAKERS.iter().enumerate() {
        if pending & (1 << line) != 0 {
            waker.wake();
        }
    }
}

#[cfg(feature = "rt")]
#[interrupt]
fn GPIO_EVEN() {
    on_interrupt(0x5555);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GPIO_ODD() {
    on_interrupt(0xAAAA);
}

/// Route external interrupt line `line` to the pin with that number on `port`.
#[cfg(feature = "rt")]
fn set_exti_port(line: u8, port: Port) {
    let r = regs();
    macro_rules! select {
        ($field:expr) => {
            match port {
                Port::A => $field.porta(),
                Port::B => $field.portb(),
                Port::C => $field.portc(),
                Port::D => $field.portd(),
                Port::E => $field.porte(),
                Port::F => $field.portf(),
            }
        };
    }
    match line {
        0 => r.extipsell.modify(|_, w| select!(w.extipsel0())),
        1 => r.extipsell.modify(|_, w| select!(w.extipsel1())),
        2 => r.extipsell.modify(|_, w| select!(w.extipsel2())),
        3 => r.extipsell.modify(|_, w| select!(w.extipsel3())),
        4 => r.extipsell.modify(|_, w| select!(w.extipsel4())),
        5 => r.extipsell.modify(|_, w| select!(w.extipsel5())),
        6 => r.extipsell.modify(|_, w| select!(w.extipsel6())),
        7 => r.extipsell.modify(|_, w| select!(w.extipsel7())),
        8 => r.extipselh.modify(|_, w| select!(w.extipsel8())),
        9 => r.extipselh.modify(|_, w| select!(w.extipsel9())),
        10 => r.extipselh.modify(|_, w| select!(w.extipsel10())),
        11 => r.extipselh.modify(|_, w| select!(w.extipsel11())),
        12 => r.extipselh.modify(|_, w| select!(w.extipsel12())),
        13 => r.extipselh.modify(|_, w| select!(w.extipsel13())),
        14 => r.extipselh.modify(|_, w| select!(w.extipsel14())),
        _ => r.extipselh.modify(|_, w| select!(w.extipsel15())),
    }
}

/// Future completing on an edge of one pin, through its external interrupt line.
#[cfg(feature = "rt")]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct ExtiFuture {
    line: u8,
}

#[cfg(feature = "rt")]
impl ExtiFuture {
    fn new(port: Port, pin: u8, rising: bool, falling: bool) -> Self {
        let r = regs();
        let bit = 1u16 << pin;

        critical_section::with(|_| {
            // Line `pin` can only serve one pin at a time.
            let claimed = EXTI_CLAIMED.load(Ordering::Relaxed);
            if claimed & bit != 0 {
                panic!("another pin with number {} is already waiting for an edge", pin);
            }
            EXTI_CLAIMED.store(claimed | bit, Ordering::Relaxed);

            set_exti_port(pin, port);
            let set_bit = |bits: u16, on: bool| if on { bits | bit } else { bits & !bit };
            r.extirise
                .modify(|r, w| unsafe { w.extirise().bits(set_bit(r.extirise().bits(), rising)) });
            r.extifall
                .modify(|r, w| unsafe { w.extifall().bits(set_bit(r.extifall().bits(), falling)) });
            // Drop any edge from before we were armed.
            r.ifc.write(|w| unsafe { w.ext().bits(bit) });
            r.ien.modify(|r, w| unsafe { w.ext().bits(r.ext().bits() | bit) });
        });

        Self { line: pin }
    }
}

#[cfg(feature = "rt")]
impl Drop for ExtiFuture {
    fn drop(&mut self) {
        let r = regs();
        let bit = 1u16 << self.line;
        critical_section::with(|_| {
            r.ien.modify(|r, w| unsafe { w.ext().bits(r.ext().bits() & !bit) });
            r.extirise
                .modify(|r, w| unsafe { w.extirise().bits(r.extirise().bits() & !bit) });
            r.extifall
                .modify(|r, w| unsafe { w.extifall().bits(r.extifall().bits() & !bit) });
            let claimed = EXTI_CLAIMED.load(Ordering::Relaxed);
            EXTI_CLAIMED.store(claimed & !bit, Ordering::Relaxed);
        });
    }
}

#[cfg(feature = "rt")]
impl Future for ExtiFuture {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        EXTI_WAKERS[self.line as usize].register(cx.waker());
        if regs().ien.read().ext().bits() & (1 << self.line) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

// embedded-hal trait implementations. These forward to the inherent methods.

macro_rules! impl_error_type {
    ($ty:ident) => {
        impl<'d> embedded_hal::digital::ErrorType for $ty<'d> {
            type Error = Infallible;
        }
    };
}

macro_rules! impl_input_traits {
    ($ty:ident) => {
        impl<'d> embedded_hal_02::digital::v2::InputPin for $ty<'d> {
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

        impl<'d> embedded_hal::digital::InputPin for $ty<'d> {
            #[inline]
            fn is_high(&mut self) -> Result<bool, Self::Error> {
                Ok((*self).is_high())
            }

            #[inline]
            fn is_low(&mut self) -> Result<bool, Self::Error> {
                Ok((*self).is_low())
            }
        }

        #[cfg(feature = "rt")]
        impl<'d> embedded_hal_async::digital::Wait for $ty<'d> {
            async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
                self.wait_for_high().await;
                Ok(())
            }

            async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
                self.wait_for_low().await;
                Ok(())
            }

            async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
                self.wait_for_rising_edge().await;
                Ok(())
            }

            async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
                self.wait_for_falling_edge().await;
                Ok(())
            }

            async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
                self.wait_for_any_edge().await;
                Ok(())
            }
        }
    };
}

macro_rules! impl_output_traits {
    ($ty:ident) => {
        impl<'d> embedded_hal_02::digital::v2::OutputPin for $ty<'d> {
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

        impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for $ty<'d> {
            #[inline]
            fn is_set_high(&self) -> Result<bool, Self::Error> {
                Ok(self.output_is_high())
            }

            #[inline]
            fn is_set_low(&self) -> Result<bool, Self::Error> {
                Ok(self.output_is_low())
            }
        }

        impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for $ty<'d> {
            type Error = Infallible;

            #[inline]
            fn toggle(&mut self) -> Result<(), Self::Error> {
                self.toggle();
                Ok(())
            }
        }

        impl<'d> embedded_hal::digital::OutputPin for $ty<'d> {
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

        impl<'d> embedded_hal::digital::StatefulOutputPin for $ty<'d> {
            #[inline]
            fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                Ok(self.output_is_high())
            }

            #[inline]
            fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                Ok(self.output_is_low())
            }

            #[inline]
            fn toggle(&mut self) -> Result<(), Self::Error> {
                self.toggle();
                Ok(())
            }
        }
    };
}

impl_error_type!(Input);
impl_input_traits!(Input);

impl_error_type!(Output);
impl_output_traits!(Output);

impl_error_type!(OutputOpenDrain);
impl_input_traits!(OutputOpenDrain);
impl_output_traits!(OutputOpenDrain);

// If the pin is disconnected (see `Flex::set_as_disconnected`), the input level is unspecified.
impl_error_type!(Flex);
impl_input_traits!(Flex);
impl_output_traits!(Flex);

pub(crate) trait SealedPin: Sized {
    fn pin_port(&self) -> Port;
    fn pin_number(&self) -> u8;
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an
/// [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Returns the pin number within its port (0..=15).
    #[inline]
    fn pin(&self) -> u8 {
        self.pin_number()
    }

    /// Returns the port of this pin.
    #[inline]
    fn port(&self) -> Port {
        self.pin_port()
    }
}

/// Type-erased GPIO pin.
pub struct AnyPin {
    pub(crate) pin_port: Port,
    pub(crate) pin_number: u8,
}

impl AnyPin {
    /// Unsafely create a new type-erased pin.
    ///
    /// Panics if `pin_number` isn't in `0..=15`.
    ///
    /// # Safety
    ///
    /// You must ensure that you're only using one instance of this type at a time.
    ///
    /// This bypasses the pins reserved by default: stealing PF0/PF1 (SWD) without the
    /// `swd-as-gpio` feature, or PB7/PB8 (LFXO) while the crystal is in use, and reconfiguring
    /// them, breaks the debug connection or the low-frequency clock. It also allows pins that
    /// the chip's package doesn't bond out.
    pub unsafe fn steal(pin_port: Port, pin_number: u8) -> Peri<'static, Self> {
        assert!(pin_number < 16, "invalid pin number {}", pin_number);
        Peri::new_unchecked(Self { pin_port, pin_number })
    }

    /// Put the pin into disconnected mode.
    ///
    /// This clears `DOUT`, which would otherwise enable the pull-up in `DISABLED` mode.
    #[inline]
    pub(crate) fn set_as_disconnected(&self) {
        set_mode(self.pin_port, self.pin_number, Mode::Disabled);
        self.set_low();
    }

    /// Put the pin into input mode with the given [Pull] configuration.
    ///
    /// `DOUT` is set as the input modes need it (it selects the pull direction in `INPUTPULL`,
    /// and enables the glitch filter in `INPUT`). It's written after the mode, so that a pin
    /// switching over from an output mode doesn't briefly drive the new `DOUT` level.
    pub(crate) fn set_as_input(&self, pull: Pull) {
        match pull {
            Pull::None => {
                set_mode(self.pin_port, self.pin_number, Mode::Input);
                self.set_low();
            }
            Pull::Up => {
                set_mode(self.pin_port, self.pin_number, Mode::InputPull);
                self.set_high();
            }
            Pull::Down => {
                set_mode(self.pin_port, self.pin_number, Mode::InputPull);
                self.set_low();
            }
        }
    }

    /// Put the pin into push-pull output mode.
    #[inline]
    pub(crate) fn set_as_output(&self) {
        set_mode(self.pin_port, self.pin_number, Mode::PushPull);
    }

    /// Set the output (`DOUT`) as high.
    #[inline]
    pub(crate) fn set_high(&self) {
        dout_set(self.pin_port, self.pin_number);
    }

    /// Set the output (`DOUT`) as low.
    #[inline]
    pub(crate) fn set_low(&self) {
        dout_clr(self.pin_port, self.pin_number);
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> Port {
        self.pin_port
    }

    #[inline]
    fn pin_number(&self) -> u8 {
        self.pin_number
    }
}

/// Implement [`Pin`] for a peripheral singleton type, given its [`Port`] and pin number.
///
/// Used by each chip's `chips/*.rs` module to wire up its pin singletons.
macro_rules! impl_pin {
    ($name:ident, $port:ident, $pin_num:expr) => {
        impl $crate::gpio::Pin for peripherals::$name {}
        impl $crate::gpio::SealedPin for peripherals::$name {
            #[inline]
            fn pin_port(&self) -> $crate::gpio::Port {
                $crate::gpio::Port::$port
            }

            #[inline]
            fn pin_number(&self) -> u8 {
                $pin_num
            }
        }

        impl From<peripherals::$name> for $crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                use $crate::gpio::SealedPin;

                Self {
                    pin_port: val.pin_port(),
                    pin_number: val.pin_number(),
                }
            }
        }
    };
}
