//! GPIO core, SDK `fsl_gpio`: data, direction, and pin interrupt registers of one port.
//!
//! The core never touches the pad: muxing, input buffer, pull and open-drain are the pin
//! controller's, which the HAL drives around the calls into this core. One [`Line`] drives one pin
//! of a port; the interrupt side shares a [`State`] per port.
//!
//! The HAL's port vector calls [`on_interrupt`]. Async waits route
//! every pin to the port's first interrupt line (`ISFR0`).

use core::future::poll_fn;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use crate::pac::gpio::Irqc;
use crate::pac::{self};

/// Number of pins per port.
pub const PINS_PER_PORT: usize = 32;

/// Digital level.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low.
    Low,
    /// High.
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
        level == Level::High
    }
}

/// Pull resistor setting of an input.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull resistor.
    None,
    /// Pull-up resistor.
    Up,
    /// Pull-down resistor.
    Down,
}

/// Condition a wait completes on.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Trigger {
    /// The pin is high.
    High,
    /// The pin is low.
    Low,
    /// Low to high transition.
    RisingEdge,
    /// High to low transition.
    FallingEdge,
    /// Any transition.
    AnyEdge,
}

impl Trigger {
    fn irqc(self) -> Irqc {
        match self {
            Trigger::High => Irqc::Irqc12,
            Trigger::Low => Irqc::Irqc8,
            Trigger::RisingEdge => Irqc::Irqc9,
            Trigger::FallingEdge => Irqc::Irqc10,
            Trigger::AnyEdge => Irqc::Irqc11,
        }
    }
}

/// Read-only data of one GPIO port; the HAL keeps it in a `static`.
pub struct Info {
    /// Register block of the port.
    pub regs: pac::gpio::Gpio,
    /// Port index.
    pub port: u8,
    /// Interrupt line the pin interrupts (`ISFR0`) of the port are routed to.
    pub interrupt: pac::Interrupt,
}

/// Mutable state of one GPIO port; the HAL keeps it in a `static`.
pub struct State {
    wakers: [AtomicWaker; PINS_PER_PORT],
    /// Pins whose armed trigger fired and has not been consumed by a wait yet.
    fired: AtomicU32,
}

impl State {
    /// Create an idle state. All-zero bytes, so a `static` of it lands in `.bss`.
    pub const fn new() -> Self {
        Self {
            wakers: [const { AtomicWaker::new() }; PINS_PER_PORT],
            fired: AtomicU32::new(0),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b as usize)
            }
        }
    }
}

/// Handle the port's pin interrupt: disarm every pin that fired and wake its waiter.
///
/// Returns the pins that were served, as a bit mask.
///
/// # Safety
///
/// Call only from the interrupt handler of the port `info` describes, with the `state` the pins
/// of that port use.
pub unsafe fn on_interrupt(info: &'static Info, state: &'static State) -> u32 {
    let flags = info.regs.isfr(0).read().0;
    for pin in BitIter(flags) {
        // Clear the flag and disarm the pin until the next wait.
        info.regs.icr(pin).write(|w| {
            w.set_isf(true);
            w.set_irqc(Irqc::Irqc0);
        });
        state.fired.fetch_or(1 << pin, Ordering::Release);
        state.wakers[pin].wake();
    }
    flags
}

/// Disarm the pin interrupt of `pin` and clear its flag.
fn disarm(info: &Info, pin: usize) {
    info.regs.icr(pin).write(|w| {
        w.set_isf(true);
        w.set_irqc(Irqc::Irqc0);
    });
}

/// Return `pin` to an input with its port input disabled (the released state).
///
/// For pins that are handed on to a peripheral function without a [`Line`].
pub fn release_pin(info: &Info, pin: u8) {
    let n = pin as usize;
    critical_section::with(|_| {
        info.regs.pddr().modify(|w| w.set_pdd(n, false));
        #[cfg(gpio_has_port_input_control)]
        info.regs.pidr().modify(|w| w.set_pid(n, true));
    });
    disarm(info, n);
}

/// Enable or disable the port input (`PIDR`) of `pin`, independent of its function.
///
/// [`release_pin`] disables the input, which also blocks a peripheral function muxed onto the
/// pin later; such a peripheral re-enables it here.
#[cfg(gpio_has_port_input_control)]
pub fn set_input_enabled(info: &Info, pin: u8, enabled: bool) {
    let n = pin as usize;
    critical_section::with(|_| {
        info.regs.pidr().modify(|w| w.set_pid(n, !enabled));
    });
}

/// One GPIO pin of a port.
pub struct Line {
    info: &'static Info,
    state: &'static State,
    pin: u8,
}

impl Line {
    /// Create the driver core for `pin` of the port `info` describes.
    ///
    /// # Safety
    ///
    /// 1. The caller exclusively owns the pin for the life of the `Line` (it holds its `Peri`),
    ///    the port's clock runs and the pad is configured for GPIO.
    /// 2. `state` belongs to that port.
    /// 3. For async waits, the port's handler calls [`on_interrupt`] with the same `info` and
    ///    `state`, and the line is unmasked.
    pub unsafe fn new(info: &'static Info, state: &'static State, pin: u8) -> Self {
        assert!((pin as usize) < PINS_PER_PORT);
        Self { info, state, pin }
    }

    #[inline]
    fn n(&self) -> usize {
        self.pin as usize
    }

    /// Set the pin direction; an output drives the current output register value.
    pub fn set_direction(&mut self, output: bool) {
        let n = self.n();
        critical_section::with(|_| {
            // Clear PIDR in case this pin was previously released (PID=1 reads always return 0).
            #[cfg(gpio_has_port_input_control)]
            self.info.regs.pidr().modify(|w| w.set_pid(n, false));
            self.info.regs.pddr().modify(|w| w.set_pdd(n, output));
        });
    }

    /// Set the output register high.
    #[inline]
    pub fn set_high(&mut self) {
        let n = self.n();
        self.info.regs.psor().write(|w| w.set_ptso(n, true));
    }

    /// Set the output register low.
    #[inline]
    pub fn set_low(&mut self) {
        let n = self.n();
        self.info.regs.pcor().write(|w| w.set_ptco(n, true));
    }

    /// Toggle the output register.
    #[inline]
    pub fn toggle(&mut self) {
        let n = self.n();
        self.info.regs.ptor().write(|w| w.set_ptto(n, true));
    }

    /// Whether the pin input reads high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.info.regs.pdir().read().pdi(self.n())
    }

    /// Whether the output register is set high.
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.info.regs.pdor().read().pdo(self.n())
    }

    /// Wait for `trigger`. Level triggers complete at once when the level already holds.
    ///
    /// Dropping the future disarms the pin.
    pub async fn wait_for(&mut self, trigger: Trigger) {
        let n = self.n();
        let bit = 1u32 << n;

        /// Disarms the pin when the wait ends or is cancelled.
        struct Disarm<'a>(&'a Info, usize);
        impl Drop for Disarm<'_> {
            fn drop(&mut self) {
                disarm(self.0, self.1);
            }
        }

        self.state.fired.fetch_and(!bit, Ordering::AcqRel);
        // Disarm and clear any stale flag before reconfiguring the pin, as the reference manual
        // requires, then arm the trigger on interrupt line 0.
        disarm(self.info, n);
        let _guard = Disarm(self.info, n);
        self.info.regs.icr(n).write(|w| {
            #[cfg(gpio_has_interrupt_channel_select)]
            w.set_irqs(false);
            w.set_irqc(trigger.irqc());
        });

        // Register the waker, then re-read `fired`: the interrupt may have run in between.
        poll_fn(|cx| {
            self.state.wakers[n].register(cx.waker());
            if self.state.fired.load(Ordering::Acquire) & bit != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        self.state.fired.fetch_and(!bit, Ordering::AcqRel);
    }

    /// Stop any armed trigger and return the pin to an input.
    pub fn release(&mut self) {
        release_pin(self.info, self.pin);
        self.state.fired.fetch_and(!(1 << self.pin), Ordering::AcqRel);
    }
}
