//! General-purpose Input/Output (GPIO).
//!
//! Provides [`Flex`], [`Input`], [`Output`] and [`OutputOpenDrain`] for Series 0 and Series 2.
//!
//! # Waiting for edges
//!
//! With the `rt` feature, [`Input`], [`OutputOpenDrain`] and [`Flex`] can wait for levels and edges
//! (`wait_for_*`, `embedded_hal_async::digital::Wait`) through GPIO external interrupt lines.
//! `GPIO_EVEN` serves even lines and `GPIO_ODD` odd lines. [`crate::init`] enables both IRQs at
//! [`crate::Config::gpio_interrupt_priority`]. The HAL defines their handlers, so do not bind
//! `GPIO_EVEN` or `GPIO_ODD` in `bind_interrupts!`.
//!
//! Line mapping per series:
//! - Series 0: line `n` is pin `n` on the port that `EXTIPSELL` or `EXTIPSELH` selects. PA3 and PB3
//!   cannot wait at the same time, but PA3 and PB4 can.
//! - Series 2: line `n` is pin `4 * (n / 4) + EXTIPINSELn` on the port that `EXTIPSELn` selects
//!   (reference manual, "Interrupt Generation"). Pin `p` takes the first free line in
//!   `4 * (p / 4)..=4 * (p / 4) + 3`.
//!
//! A wait panics if its pin has no free line. A line is free again when its future ends or drops.

#![macro_use]

use core::convert::Infallible;

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};

use crate::pac::GPIO;
// Port registers have the same shape in every GPIO version. Series 0 `DOUTSET`, `DOUTCLR` and
// `DOUTTGL` are real registers, not bus aliases, with Series 2 names (`p_dout_set`) and behavior.
use crate::pac::gpio as gpio_mod;

/// Logic level.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low.
    Low,
    /// High.
    High,
}

impl From<bool> for Level {
    fn from(b: bool) -> Self {
        if b { Level::High } else { Level::Low }
    }
}
impl From<Level> for bool {
    fn from(l: Level) -> Self {
        matches!(l, Level::High)
    }
}

/// Pull setting for an input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull.
    None,
    /// Pull-up.
    Up,
    /// Pull-down.
    Down,
}

/// Output drive characteristics, ignored by the drivers.
///
/// Push-pull mode uses the default drive strength. Series 0 `P[n].CTRL.DRIVEMODE` applies only to
/// `PUSHPULLDRIVE` modes, which the drivers do not use.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    /// Low.
    Low,
    /// Medium.
    Medium,
    /// High.
    High,
}

/// GPIO port.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    #[cfg(silabs_series = "0")]
    E,
    /// Port F.
    #[cfg(silabs_series = "0")]
    F,
}

impl Port {
    fn from_index(i: u8) -> Self {
        match i {
            0 => Port::A,
            1 => Port::B,
            2 => Port::C,
            3 => Port::D,
            #[cfg(silabs_series = "0")]
            4 => Port::E,
            #[cfg(silabs_series = "0")]
            5 => Port::F,
            _ => unreachable!(),
        }
    }
}

use gpio_mod::vals::Mode;

/// Type-erased GPIO pin reference.
pub struct AnyPin {
    pin_port: u8,
}

impl_peripheral!(AnyPin);

impl AnyPin {
    /// Creates an `AnyPin` for `pin_port` without ownership.
    ///
    /// # Safety
    ///
    /// No other `AnyPin` for the same pin may be in scope at the same time.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }

    /// Returns `port * 16 + pin`. EUSART route registers take both parts.
    #[cfg_attr(not(eusart), allow(dead_code))]
    #[inline]
    pub(crate) fn pin_port(&self) -> u8 {
        self.pin_port
    }

    #[inline]
    fn port_index(&self) -> usize {
        (self.pin_port >> 4) as usize
    }

    #[inline]
    fn pin_number(&self) -> u8 {
        self.pin_port & 0xF
    }
}

/// Flexible GPIO pin: an input, an output, or both.
///
/// [`Flex::new`] does not change the pin, so set its mode with the `set_as_*` functions. Dropping a
/// `Flex` disconnects the pin.
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wraps the pin in a `Flex`. The pin keeps its mode.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self { pin: pin.into() }
    }

    /// Returns the pin's port.
    #[inline]
    pub fn port(&self) -> Port {
        Port::from_index(self.pin.port_index() as u8)
    }

    /// Returns the pin number within its port.
    #[inline]
    pub fn pin(&self) -> u8 {
        self.pin.pin_number()
    }

    /// Sets the pin as an input with pull `pull`.
    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        // In `INPUTPULL` mode, `DOUT` 1 selects pull-up and 0 selects pull-down. In `INPUT` mode,
        // `DOUT` 1 turns on the glitch filter.
        match pull {
            Pull::None => {
                write_dout_bit(&self.pin, false);
                write_mode(&self.pin, Mode::Input);
            }
            Pull::Up => {
                write_dout_bit(&self.pin, true);
                write_mode(&self.pin, Mode::Inputpull);
            }
            Pull::Down => {
                write_dout_bit(&self.pin, false);
                write_mode(&self.pin, Mode::Inputpull);
            }
        }
    }

    /// Sets the pin as a push-pull output. Set the level first, or the pin glitches.
    #[inline]
    pub fn set_as_output(&mut self) {
        write_mode(&self.pin, Mode::Pushpull);
    }

    /// Sets the pin as an open-drain output (`WIREDAND`): low drives the pin, high releases it.
    ///
    /// The input stays on, so [`Flex::is_high`] reads the pin.
    #[inline]
    pub fn set_as_output_open_drain(&mut self) {
        write_mode(&self.pin, Mode::Wiredand);
    }

    /// Sets the pin as an open-source output (`WIREDOR`): high drives the pin, low releases it.
    #[inline]
    pub fn set_as_output_open_source(&mut self) {
        write_mode(&self.pin, Mode::Wiredor);
    }

    /// Disconnects the pin (`DISABLED`, no pull), the reset state of most pins.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        set_as_disabled(&self.pin);
    }

    /// Returns `true` if the pin is an input (`INPUT` to `INPUTPULLFILTER`).
    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(
            read_mode(&self.pin),
            Mode::Input | Mode::Inputpull | Mode::Inputpullfilter
        )
    }

    /// Returns `true` if the pin is an output (push-pull, open-source or open-drain).
    #[inline]
    pub fn is_output(&self) -> bool {
        !self.is_input() && !self.is_disconnected()
    }

    /// Returns `true` if the pin is disconnected (`DISABLED`).
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        read_mode(&self.pin) == Mode::Disabled
    }

    /// Returns `true` if the input level is high (`DIN`).
    #[inline]
    pub fn is_high(&self) -> bool {
        GPIO.p_din(self.pin.port_index()).read().0 & (1 << self.pin.pin_number()) != 0
    }

    /// Returns `true` if the input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Returns the input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
    }

    /// Sets the output level high.
    #[inline]
    pub fn set_high(&mut self) {
        write_dout_bit(&self.pin, true);
    }

    /// Sets the output level low.
    #[inline]
    pub fn set_low(&mut self) {
        write_dout_bit(&self.pin, false);
    }

    /// Sets the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        write_dout_bit(&self.pin, level.into());
    }

    /// Toggles the output level.
    #[inline]
    pub fn toggle(&mut self) {
        GPIO.p_dout_tgl(self.pin.port_index())
            .write_value(gpio_mod::regs::PortDout(1 << self.pin.pin_number()));
    }

    /// Returns `true` if the output level is high (`DOUT`).
    #[inline]
    pub fn is_set_high(&self) -> bool {
        GPIO.p_dout(self.pin.port_index()).read().0 & (1 << self.pin.pin_number()) != 0
    }

    /// Returns `true` if the output level is low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Returns the output level (`DOUT`).
    #[inline]
    pub fn output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Waits until the pin is high. Returns at once when it is high.
    #[cfg(feature = "rt")]
    pub async fn wait_for_high(&mut self) {
        // A pin already at the level needs no line, so read the level before claiming one. Read it
        // again after the arm, so no edge is lost.
        if self.is_high() {
            return;
        }
        let fut = exti::ExtiFuture::new(&self.pin, true, false);
        if self.is_high() {
            return;
        }
        fut.await
    }

    /// Waits until the pin is low. Returns at once when it is low.
    #[cfg(feature = "rt")]
    pub async fn wait_for_low(&mut self) {
        // A pin already at the level needs no line, so read the level before claiming one. Read it
        // again after the arm, so no edge is lost.
        if self.is_low() {
            return;
        }
        let fut = exti::ExtiFuture::new(&self.pin, false, true);
        if self.is_low() {
            return;
        }
        fut.await
    }

    /// Waits for a rising edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_rising_edge(&mut self) {
        exti::ExtiFuture::new(&self.pin, true, false).await
    }

    /// Waits for a falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_falling_edge(&mut self) {
        exti::ExtiFuture::new(&self.pin, false, true).await
    }

    /// Waits for a rising or falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_any_edge(&mut self) {
        exti::ExtiFuture::new(&self.pin, true, true).await
    }
}

impl<'d> Drop for Flex<'d> {
    #[inline]
    fn drop(&mut self) {
        set_as_disabled(&self.pin);
    }
}

/// Push-pull GPIO output.
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Creates a push-pull output on `pin` at level `initial`. The drivers ignore `speed`.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level, _speed: Speed) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial);
        pin.set_as_output();
        Self { pin }
    }

    /// Sets the output high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Sets the output low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Sets the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Toggles the output.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Returns `true` if the output level is high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Returns `true` if the output level is low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Returns the output level.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }
}

/// GPIO input.
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Creates an input on `pin` with pull `pull`.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input(pull);
        Self { pin }
    }

    /// Returns `true` if the input is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Returns `true` if the input is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns the input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Waits until the input is high. Returns at once when it is high.
    #[cfg(feature = "rt")]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Waits until the input is low. Returns at once when it is low.
    #[cfg(feature = "rt")]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Waits for a rising edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Waits for a falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Waits for a rising or falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// Open-drain GPIO output: low drives the pin, high releases it.
///
/// The input stays on, so the driver can read the pin and wait for it.
pub struct OutputOpenDrain<'d> {
    pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Creates an open-drain output on `pin` at level `initial`.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial);
        pin.set_as_output_open_drain();
        Self { pin }
    }

    /// Returns `true` if the pin is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Returns `true` if the pin is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns the pin's level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Releases the pin (output high).
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Drives the pin low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Sets the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Toggles the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Returns `true` if the output level is high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Returns `true` if the output level is low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Returns the output level.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }

    /// Waits until the pin is high. Returns at once when it is high.
    #[cfg(feature = "rt")]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Waits until the pin is low. Returns at once when it is low.
    #[cfg(feature = "rt")]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Waits for a rising edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Waits for a falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Waits for a rising or falling edge.
    #[cfg(feature = "rt")]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

mod sealed {
    pub trait SealedPin {
        fn pin_port(&self) -> u8;
    }
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + sealed::SealedPin {}

/// Declares a pin trait for one peripheral signal.
///
/// `pin_trait!(TxPin, Instance)` gives `pub trait TxPin<T: Instance>: Pin {}`. A driver takes a
/// `TxPin<T>`, so a pin that cannot carry the signal does not compile. The macro is unused on a
/// chip without such a driver (MG22).
///
/// The Series 0 form `pin_trait!(TxPin, Instance, locations)` adds `LOCATIONS`: the ROUTE LOCATION
/// values (ascending) where the pin carries the signal.
#[allow(unused_macros)]
macro_rules! pin_trait {
    ($signal:ident, $instance:path) => {
        #[doc = concat!(stringify!($signal), " pin trait")]
        pub trait $signal<T: $instance>: crate::gpio::Pin {}
    };
    ($signal:ident, $instance:path, locations) => {
        #[doc = concat!(stringify!($signal), " pin trait")]
        pub trait $signal<T: $instance>: crate::gpio::Pin {
            #[doc = concat!("The ROUTE LOCATION values at which this pin is ", stringify!($signal), ", in increasing order. Never empty.")]
            const LOCATIONS: &'static [u8];
        }
    };
}

/// Implements a pin trait for one pin.
///
/// `build.rs` emits one call for each pin of the signal in chip metadata (`Peripheral.pins`).
///
/// The Series 0 form adds locations: `pin_trait_impl!(crate::usart::TxPin, USART1, PD0, [1])`.
#[allow(unused_macros)]
macro_rules! pin_trait_impl {
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident) => {
        impl crate::$mod::$trait<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident, [$($location:literal),+]) => {
        impl crate::$mod::$trait<crate::peripherals::$instance> for crate::peripherals::$pin {
            const LOCATIONS: &'static [u8] = &[$($location),+];
        }
    };
}

impl sealed::SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}
impl Pin for AnyPin {}

foreach_pin!(
    ($name:ident, $pin_port:expr) => {
        impl sealed::SealedPin for crate::peripherals::$name {
            #[inline]
            fn pin_port(&self) -> u8 {
                $pin_port
            }
        }
        impl Pin for crate::peripherals::$name {}
        impl From<crate::peripherals::$name> for AnyPin {
            #[inline]
            fn from(_: crate::peripherals::$name) -> Self {
                Self { pin_port: $pin_port }
            }
        }
    };
);

#[inline]
fn write_mode(pin: &AnyPin, mode: Mode) {
    let port = pin.port_index();
    let n = pin.pin_number() as usize;

    // A 4-bit mode field has no SET/CLR form, so read-modify-write it.
    critical_section::with(|_| {
        if n < 8 {
            GPIO.p_model(port).modify(|w| w.set_mode(n, mode));
        } else {
            GPIO.p_modeh(port).modify(|w| w.set_mode(n - 8, mode));
        }
    });
}

#[inline]
fn read_mode(pin: &AnyPin) -> Mode {
    let port = pin.port_index();
    let n = pin.pin_number() as usize;
    if n < 8 {
        GPIO.p_model(port).read().mode(n)
    } else {
        GPIO.p_modeh(port).read().mode(n - 8)
    }
}

#[inline]
fn write_dout_bit(pin: &AnyPin, high: bool) {
    let port = pin.port_index();
    let bit = gpio_mod::regs::PortDout(1 << pin.pin_number());
    // `P_DOUT_SET` and `P_DOUT_CLR` change only the written bits, so no critical section.
    if high {
        GPIO.p_dout_set(port).write_value(bit);
    } else {
        GPIO.p_dout_clr(port).write_value(bit);
    }
}

/// Sets `pin` as a push-pull output for a peripheral, such as EUSART TX.
/// Sets the line idle-high before switching to push-pull so a UART idle
/// level is presented immediately.
#[cfg(any(eusart, all(silabs_series = "0", usart)))]
#[inline]
pub(crate) fn set_as_alternate_output(pin: &AnyPin) {
    write_dout_bit(pin, true);
    write_mode(pin, Mode::Pushpull);
}

/// Sets `pin` as a floating input for a peripheral, such as EUSART RX.
#[cfg(any(eusart, all(silabs_series = "0", usart)))]
#[inline]
pub(crate) fn set_as_alternate_input(pin: &AnyPin) {
    write_dout_bit(pin, false);
    write_mode(pin, Mode::Input);
}

/// Puts `pin` back in its reset state (`DISABLED`).
///
/// `DOUT` goes low first, because `DOUT` high in `DISABLED` mode turns on the pull-up.
#[inline]
pub(crate) fn set_as_disabled(pin: &AnyPin) {
    write_dout_bit(pin, false);
    write_mode(pin, Mode::Disabled);
}

/// Sets up GPIO at `init` and enables `GPIO_EVEN` and `GPIO_ODD` at `prio`.
///
/// A bootloader can leave lines armed, flags set, `INSENSE.INT` clear or the configuration locked,
/// so the code does not assume reset state.
pub(crate) fn init(prio: crate::interrupt::Priority) {
    use crate::interrupt::InterruptExt;

    GPIO.lock().write(|w| w.set_lockkey(gpio_mod::vals::Lockkey::Unlock));
    critical_section::with(|_| {
        // IEN bits 0..LINES are the EXT lines. Keep the other bits (Series 2 EM4 wake-up enables).
        GPIO.ien().modify(|w| w.0 &= !exti::LINE_MASK);
        GPIO.extirise().write(|w| w.0 = 0);
        GPIO.extifall().write(|w| w.0 = 0);
    });
    GPIO.if_clr().write(|w| w.0 = exti::LINE_MASK);
    // Series 0: the interrupt synchronizers run only with `INSENSE.INT`.
    #[cfg(silabs_series = "0")]
    GPIO.insense().modify(|w| w.set_int(true));

    for irq in [exti::EVEN_IRQ, exti::ODD_IRQ] {
        irq.unpend();
        irq.set_priority(prio);
        #[cfg(feature = "rt")]
        unsafe {
            irq.enable()
        };
    }
}

/// External interrupt lines.
///
/// The handlers that `build.rs` emits call [`on_interrupt`](exti::on_interrupt). `build.rs` also
/// emits both IRQs and `LINES`, the width of `EXTIRISE`.
pub(crate) mod exti {
    pub(crate) use crate::_generated_exti::{EVEN_IRQ, LINES, ODD_IRQ};

    /// EXT bits in IEN, IF, EXTIRISE and EXTIFALL.
    pub(crate) const LINE_MASK: u32 = (1u32 << LINES) - 1;

    #[cfg(feature = "rt")]
    pub(crate) use waits::*;

    #[cfg(feature = "rt")]
    mod waits {
        use core::future::Future;
        use core::pin::Pin;
        use core::sync::atomic::{AtomicU32, Ordering};
        use core::task::{Context, Poll};

        use embassy_sync::waitqueue::AtomicWaker;

        use super::{LINE_MASK, LINES};
        use crate::gpio::AnyPin;
        use crate::pac::GPIO;

        static WAKERS: [AtomicWaker; LINES] = [const { AtomicWaker::new() }; LINES];
        /// Lines owned by live `ExtiFuture`s, one bit per line. Changed only in critical sections.
        static CLAIMED: AtomicU32 = AtomicU32::new(0);

        /// Disarms each pending line in `lines` and wakes its future.
        ///
        /// A disarmed line tells the future that its edge came. The `GPIO_EVEN` and `GPIO_ODD`
        /// handlers call this with `0x5555` or `0xAAAA`.
        pub(crate) fn on_interrupt(lines: u32) {
            let pending = critical_section::with(|_| {
                let pending = GPIO.if_().read().0 & GPIO.ien().read().0 & lines & LINE_MASK;
                GPIO.ien().modify(|w| w.0 &= !pending);
                GPIO.if_clr().write(|w| w.0 = pending);
                pending
            });
            for (line, waker) in WAKERS.iter().enumerate() {
                if pending & (1 << line) != 0 {
                    waker.wake();
                }
            }
        }

        /// Returns the lines that pin number `pin` can use, in the order to try.
        #[cfg(silabs_series = "0")]
        fn candidate_lines(pin: u8) -> core::ops::Range<u8> {
            pin..pin + 1
        }

        /// Returns the lines that pin number `pin` can use, in the order to try.
        #[cfg(silabs_series = "2")]
        fn candidate_lines(pin: u8) -> core::ops::Range<u8> {
            let base = pin & !3;
            base..(base + 4).min(LINES as u8)
        }

        /// Routes line `line` to `port`, and on Series 2 to pin number `pin`.
        ///
        /// `EXTIPSELL`/`EXTIPSELH` and `EXTIPINSELL`/`EXTIPINSELH` have one 4-bit slot per line:
        /// lines 0..7 in the low register, 8.. in the high one. The field is 3 bits wide on Series
        /// 0 and 2 bits on Series 2. Upper slot bits are reserved, with reset value 0.
        fn route(line: u8, port: u8, pin: u8) {
            let shift = 4 * (line as u32 % 8);
            let set = |v: u32, value: u32| (v & !(0xF << shift)) | (value << shift);
            if line < 8 {
                GPIO.extipsell().modify(|w| w.0 = set(w.0, port as u32));
            } else {
                GPIO.extipselh().modify(|w| w.0 = set(w.0, port as u32));
            }
            #[cfg(silabs_series = "2")]
            {
                // Offset from the line's base pin `4 * (line / 4)`.
                let offset = (pin & 3) as u32;
                if line < 8 {
                    GPIO.extipinsell().modify(|w| w.0 = set(w.0, offset));
                } else {
                    GPIO.extipinselh().modify(|w| w.0 = set(w.0, offset));
                }
            }
            #[cfg(silabs_series = "0")]
            let _ = pin;
        }

        /// Future that completes at a pin edge, through an external interrupt line.
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        pub(crate) struct ExtiFuture {
            line: u8,
        }

        impl ExtiFuture {
            pub(crate) fn new(pin: &AnyPin, rising: bool, falling: bool) -> Self {
                let port = pin.port_index() as u8;
                let n = pin.pin_number();
                let line = critical_section::with(|_| {
                    let claimed = CLAIMED.load(Ordering::Relaxed);
                    let Some(line) = candidate_lines(n).find(|l| claimed & (1 << l) == 0) else {
                        panic!(
                            "gpio: no free EXTI line for P{}{}: the lines {:?} wait for other pins",
                            (b'A' + port) as char,
                            n,
                            candidate_lines(n)
                        );
                    };
                    let bit = 1u32 << line;
                    CLAIMED.store(claimed | bit, Ordering::Relaxed);
                    route(line, port, n);
                    let edge = |v: u32, on: bool| if on { v | bit } else { v & !bit };
                    GPIO.extirise().modify(|w| w.0 = edge(w.0, rising));
                    GPIO.extifall().modify(|w| w.0 = edge(w.0, falling));
                    // Drop an edge from before the line was armed.
                    GPIO.if_clr().write(|w| w.0 = bit);
                    GPIO.ien().modify(|w| w.0 |= bit);
                    line
                });
                Self { line }
            }
        }

        impl Drop for ExtiFuture {
            fn drop(&mut self) {
                let bit = 1u32 << self.line;
                critical_section::with(|_| {
                    GPIO.ien().modify(|w| w.0 &= !bit);
                    GPIO.extirise().modify(|w| w.0 &= !bit);
                    GPIO.extifall().modify(|w| w.0 &= !bit);
                    CLAIMED.fetch_and(!bit, Ordering::Relaxed);
                });
            }
        }

        impl Future for ExtiFuture {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                WAKERS[self.line as usize].register(cx.waker());
                if GPIO.ien().read().0 & (1 << self.line) == 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

// embedded-hal 1.0 traits.

macro_rules! impl_eh1 {
    ($ty:ident, input) => {
        impl<'d> embedded_hal::digital::InputPin for $ty<'d> {
            #[inline]
            fn is_high(&mut self) -> Result<bool, Self::Error> {
                Ok($ty::is_high(self))
            }
            #[inline]
            fn is_low(&mut self) -> Result<bool, Self::Error> {
                Ok($ty::is_low(self))
            }
        }
    };
    ($ty:ident, output) => {
        impl<'d> embedded_hal::digital::OutputPin for $ty<'d> {
            #[inline]
            fn set_low(&mut self) -> Result<(), Self::Error> {
                $ty::set_low(self);
                Ok(())
            }
            #[inline]
            fn set_high(&mut self) -> Result<(), Self::Error> {
                $ty::set_high(self);
                Ok(())
            }
        }
        impl<'d> embedded_hal::digital::StatefulOutputPin for $ty<'d> {
            #[inline]
            fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                Ok($ty::is_set_high(self))
            }
            #[inline]
            fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                Ok($ty::is_set_low(self))
            }
            #[inline]
            fn toggle(&mut self) -> Result<(), Self::Error> {
                $ty::toggle(self);
                Ok(())
            }
        }
    };
}

// embedded-hal 0.2 traits.

macro_rules! impl_eh02 {
    ($ty:ident, input) => {
        impl<'d> embedded_hal_02::digital::v2::InputPin for $ty<'d> {
            type Error = Infallible;
            #[inline]
            fn is_high(&self) -> Result<bool, Self::Error> {
                Ok($ty::is_high(self))
            }
            #[inline]
            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok($ty::is_low(self))
            }
        }
    };
    ($ty:ident, output) => {
        impl<'d> embedded_hal_02::digital::v2::OutputPin for $ty<'d> {
            type Error = Infallible;
            #[inline]
            fn set_low(&mut self) -> Result<(), Self::Error> {
                $ty::set_low(self);
                Ok(())
            }
            #[inline]
            fn set_high(&mut self) -> Result<(), Self::Error> {
                $ty::set_high(self);
                Ok(())
            }
        }
        impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for $ty<'d> {
            #[inline]
            fn is_set_high(&self) -> Result<bool, Self::Error> {
                Ok($ty::is_set_high(self))
            }
            #[inline]
            fn is_set_low(&self) -> Result<bool, Self::Error> {
                Ok($ty::is_set_low(self))
            }
        }
        impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for $ty<'d> {
            type Error = Infallible;
            #[inline]
            fn toggle(&mut self) -> Result<(), Self::Error> {
                $ty::toggle(self);
                Ok(())
            }
        }
    };
}

// embedded-hal-async `Wait`.

#[cfg(feature = "rt")]
macro_rules! impl_wait {
    ($ty:ident) => {
        impl<'d> embedded_hal_async::digital::Wait for $ty<'d> {
            async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
                $ty::wait_for_high(self).await;
                Ok(())
            }
            async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
                $ty::wait_for_low(self).await;
                Ok(())
            }
            async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
                $ty::wait_for_rising_edge(self).await;
                Ok(())
            }
            async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
                $ty::wait_for_falling_edge(self).await;
                Ok(())
            }
            async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
                $ty::wait_for_any_edge(self).await;
                Ok(())
            }
        }
    };
}

macro_rules! impl_error_type {
    ($($ty:ident),*) => {
        $(
            impl<'d> embedded_hal::digital::ErrorType for $ty<'d> {
                type Error = Infallible;
            }
        )*
    };
}

impl_error_type!(Flex, Input, Output, OutputOpenDrain);

impl_eh1!(Flex, input);
impl_eh1!(Flex, output);
impl_eh1!(Input, input);
impl_eh1!(Output, output);
impl_eh1!(OutputOpenDrain, input);
impl_eh1!(OutputOpenDrain, output);

impl_eh02!(Flex, input);
impl_eh02!(Flex, output);
impl_eh02!(Input, input);
impl_eh02!(Output, output);
impl_eh02!(OutputOpenDrain, input);
impl_eh02!(OutputOpenDrain, output);

#[cfg(feature = "rt")]
impl_wait!(Flex);
#[cfg(feature = "rt")]
impl_wait!(Input);
#[cfg(feature = "rt")]
impl_wait!(OutputOpenDrain);
