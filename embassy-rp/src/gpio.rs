//! GPIO driver.
#![macro_use]
use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::SIO;
use crate::pac::common::{RW, Reg};
use crate::{RegExt, pac, peripherals};

#[cfg(any(feature = "rp2040", feature = "rp235xa"))]
pub(crate) const BANK0_PIN_COUNT: usize = 30;
#[cfg(feature = "rp235xb")]
pub(crate) const BANK0_PIN_COUNT: usize = 48;

/// `FUNCSEL` value that connects a pin to SIO.
#[cfg(feature = "rp2040")]
pub(crate) const SIO_FUNCSEL: u8 = pac::io::vals::Gpio0CtrlFuncsel::Sio0 as _;
/// `FUNCSEL` value that connects a pin to SIO.
#[cfg(feature = "_rp235x")]
pub(crate) const SIO_FUNCSEL: u8 = pac::io::vals::Gpio0CtrlFuncsel::SiobProc0 as _;

/// Represents a digital input or output level.
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
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// Represents a pull setting for an input.
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

/// Drive strength of an output
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Drive {
    /// 2 mA drive.
    _2mA,
    /// 4 mA drive.
    _4mA,
    /// 8 mA drive.
    _8mA,
    /// 1 2mA drive.
    _12mA,
}
/// Slew rate of an output
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlewRate {
    /// Fast slew rate.
    Fast,
    /// Slow slew rate.
    Slow,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Bank {
    /// Bank 0.
    Bank0 = 0,
    /// QSPI.
    #[cfg(feature = "qspi-as-gpio")]
    Qspi = 1,
}

/// Dormant mode config.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DormantWakeConfig {
    /// Wake on edge high.
    pub edge_high: bool,
    /// Wake on edge low.
    pub edge_low: bool,
    /// Wake on level high.
    pub level_high: bool,
    /// Wake on level low.
    pub level_low: bool,
}

/// GPIO input driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Input<'d, M: Mode = Blocking> {
    pin: Flex<'d, M>,
}

impl<'d> Input<'d, Blocking> {
    /// Create a blocking GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        Self::new_inner(Flex::new(pin), pull)
    }
}

impl<'d> Input<'d, Async> {
    /// Create an async GPIO input driver for a [Pin] with the provided [Pull] configuration.
    ///
    /// The pin's interrupt must be bound to [`InterruptHandler`] for this pin.
    #[inline]
    pub fn new_async<T: Pin>(
        pin: Peri<'d, T>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        pull: Pull,
    ) -> Self {
        Self::new_inner(Flex::new_async(pin, irq), pull)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

impl<'d, M: Mode> Input<'d, M> {
    #[inline]
    fn new_inner(mut pin: Flex<'d, M>, pull: Pull) -> Self {
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    /// Set the pin's Schmitt trigger.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        self.pin.set_schmitt(enable)
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

    /// Returns current pin level
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_input_inversion(invert)
    }

    /// Configure dormant wake.
    #[inline]
    pub fn dormant_wake(&mut self, cfg: DormantWakeConfig) -> DormantWake<'_> {
        self.pin.dormant_wake(cfg)
    }

    /// Set the pin's pad isolation
    #[cfg(feature = "_rp235x")]
    #[inline]
    pub fn set_pad_isolation(&mut self, isolate: bool) {
        self.pin.set_pad_isolation(isolate)
    }

    /// Give up the pin, leaving it configured.
    ///
    /// Dropping disconnects the pin, this doesn't. The pin keeps its configuration after the
    /// borrow it came from ends.
    #[inline]
    pub fn persist(self) {
        self.pin.persist()
    }
}

/// Interrupt trigger levels.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptTrigger {
    /// Trigger on pin low.
    LevelLow,
    /// Trigger on pin high.
    LevelHigh,
    /// Trigger on high to low transition.
    EdgeLow,
    /// Trigger on low to high transition.
    EdgeHigh,
    /// Trigger on any transition.
    AnyEdge,
}

/// GPIO interrupt handler for one pin.
///
/// Bind one handler per pin used with an async [`Input`], [`OutputOpenDrain`] or [`Flex`], on that
/// pin's interrupt ([`Pin::Interrupt`]). All pins of a bank share one interrupt, so list one
/// handler per pin:
///
/// ```rust,ignore
/// bind_interrupts!(struct Irqs {
///     IO_IRQ_BANK0 => gpio::InterruptHandler<peripherals::PIN_5>, gpio::InterruptHandler<peripherals::PIN_13>;
/// });
/// ```
///
/// The handler only services its own pin, so the same interrupt can also be bound to your own
/// [`Handler`] for pins used in blocking mode.
///
/// The async constructors enable the interrupt in the NVIC of the core they run on. The HAL doesn't
/// enable it otherwise, so if you only use your own handler, enable it yourself.
pub struct InterruptHandler<T: Pin> {
    _phantom: PhantomData<T>,
}

impl<T: Pin> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let pin = T::PIN_NUMBER;
        let idx = pin.pin() as usize;
        let group = idx % 8;
        // INTS is INTR masked by the INTE of the core running this handler, so this only sees
        // events that a future on this core is waiting for.
        let proc_intx = pin.int_proc();
        let event = (proc_intx.ints(idx / 8).read().0 >> (group * 4)) & 0xf;

        // No more than one event can be awaited per pin at any given time, so clear all of the pin's
        // interrupt enables without checking which one fired. The future sees them cleared and
        // completes.
        if event != 0 {
            proc_intx.inte(idx / 8).write_clear(|w| {
                w.set_edge_high(group, true);
                w.set_edge_low(group, true);
                w.set_level_high(group, true);
                w.set_level_low(group, true);
            });
            T::waker().wake();
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture {
    pin: PinNumber,
    waker: &'static AtomicWaker,
}

impl InputFuture {
    fn new(pin: PinNumber, waker: &'static AtomicWaker, level: InterruptTrigger) -> Self {
        let pin_group = (pin.pin() % 8) as usize;
        // first, clear the INTR register bits. without this INTR will still
        // contain reports of previous edges, causing the IRQ to fire early
        // on stale state. clearing these means that we can only detect edges
        // that occur *after* the clear happened, but since both this and the
        // alternative are fundamentally racy it's probably fine.
        // (the alternative being checking the current level and waiting for
        // its inverse, but that requires reading the current level and thus
        // missing anything that happened before the level was read.)
        pin.io().intr(pin.pin() as usize / 8).write(|w| {
            w.set_edge_high(pin_group, true);
            w.set_edge_low(pin_group, true);
        });

        // Each INTR register is divided into 8 groups, one group for each
        // pin, and each group consists of LEVEL_LOW, LEVEL_HIGH, EDGE_LOW,
        // and EDGE_HIGH.
        pin.int_proc()
            .inte((pin.pin() / 8) as usize)
            .write_set(|w| match level {
                InterruptTrigger::LevelHigh => {
                    w.set_level_high(pin_group, true);
                }
                InterruptTrigger::LevelLow => {
                    w.set_level_low(pin_group, true);
                }
                InterruptTrigger::EdgeHigh => {
                    w.set_edge_high(pin_group, true);
                }
                InterruptTrigger::EdgeLow => {
                    w.set_edge_low(pin_group, true);
                }
                InterruptTrigger::AnyEdge => {
                    w.set_edge_high(pin_group, true);
                    w.set_edge_low(pin_group, true);
                }
            });

        Self { pin, waker }
    }
}

impl Drop for InputFuture {
    fn drop(&mut self) {
        // Disarm, in case the future is dropped before the event happened.
        let pin_group = (self.pin.pin() % 8) as usize;
        self.pin
            .int_proc()
            .inte((self.pin.pin() / 8) as usize)
            .write_clear(|w| {
                w.set_edge_high(pin_group, true);
                w.set_edge_low(pin_group, true);
                w.set_level_high(pin_group, true);
                w.set_level_low(pin_group, true);
            });
    }
}

impl Future for InputFuture {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        self.waker.register(cx.waker());

        // self.int_proc() will get the register offset for the current cpu,
        // then we want to access the interrupt enable register for our
        // pin (there are 4 of these PROC0_INTE0, PROC0_INTE1, PROC0_INTE2, and
        // PROC0_INTE3 per cpu).
        let inte: pac::io::regs::Int = self.pin.int_proc().inte((self.pin.pin() / 8) as usize).read();
        // The register is divided into groups of four, one group for
        // each pin. Each group consists of four trigger levels LEVEL_LOW,
        // LEVEL_HIGH, EDGE_LOW, and EDGE_HIGH for each pin.
        let pin_group = (self.pin.pin() % 8) as usize;

        // since the interrupt handler clears all INTE flags we'll check that
        // all have been cleared and unconditionally return Ready(()) if so.
        // we don't need further handshaking since only a single event wait
        // is possible for any given pin at any given time.
        if !inte.edge_high(pin_group)
            && !inte.edge_low(pin_group)
            && !inte.level_high(pin_group)
            && !inte.level_low(pin_group)
        {
            return Poll::Ready(());
        }
        Poll::Pending
    }
}

/// GPIO output driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        pin.set_as_output();
        Self { pin }
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.set_drive_strength(strength)
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate)
    }

    /// Configure the output logic inversion of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_output_inversion(invert)
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
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
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Set the pin's pad isolation
    #[cfg(feature = "_rp235x")]
    #[inline]
    pub fn set_pad_isolation(&mut self, isolate: bool) {
        self.pin.set_pad_isolation(isolate)
    }

    /// Give up the pin, leaving it configured.
    ///
    /// Dropping disconnects the pin, this doesn't. The pin keeps its configuration after the
    /// borrow it came from ends.
    #[inline]
    pub fn persist(self) {
        self.pin.persist()
    }
}

/// GPIO output open-drain.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutputOpenDrain<'d, M: Mode = Blocking> {
    pin: Flex<'d, M>,
}

impl<'d> OutputOpenDrain<'d, Blocking> {
    /// Create a blocking GPIO output driver for a [Pin] in open drain mode with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        Self::new_inner(Flex::new(pin), initial_output)
    }
}

impl<'d> OutputOpenDrain<'d, Async> {
    /// Create an async GPIO output driver for a [Pin] in open drain mode with the provided [Level].
    ///
    /// The pin's interrupt must be bound to [`InterruptHandler`] for this pin.
    #[inline]
    pub fn new_async<T: Pin>(
        pin: Peri<'d, T>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        initial_output: Level,
    ) -> Self {
        Self::new_inner(Flex::new_async(pin, irq), initial_output)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

impl<'d, M: Mode> OutputOpenDrain<'d, M> {
    #[inline]
    fn new_inner(mut pin: Flex<'d, M>, initial_output: Level) -> Self {
        pin.set_low();
        match initial_output {
            Level::High => pin.set_as_input(),
            Level::Low => pin.set_as_output(),
        }
        Self { pin }
    }

    /// Set the pin's pull-up.
    #[inline]
    pub fn set_pullup(&mut self, enable: bool) {
        if enable {
            self.pin.set_pull(Pull::Up);
        } else {
            self.pin.set_pull(Pull::None);
        }
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.set_drive_strength(strength)
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate)
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        // For Open Drain High, disable the output pin. The input buffer is always on, so only touch
        // the output enable to keep this a single write.
        self.pin.pin.sio_oe().value_clr().write_value(self.pin.bit())
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        // For Open Drain Low, enable the output pin.
        self.pin.pin.sio_oe().value_set().write_value(self.pin.bit())
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_output()
    }

    /// What level output is set to
    #[inline]
    pub fn output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.pin.sio_oe().value_xor().write_value(self.pin.bit())
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

    /// Returns current pin level
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the pin's pad isolation
    #[cfg(feature = "_rp235x")]
    #[inline]
    pub fn set_pad_isolation(&mut self, isolate: bool) {
        self.pin.set_pad_isolation(isolate)
    }

    /// Give up the pin, leaving it configured.
    ///
    /// Dropping disconnects the pin, this doesn't. The pin keeps its configuration after the
    /// borrow it came from ends.
    #[inline]
    pub fn persist(self) {
        self.pin.persist()
    }
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin. The output level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d, M: Mode = Blocking> {
    pin: PinNumber,
    waker: M::IfAsync<&'static AtomicWaker>,
    _phantom: PhantomData<(&'d mut (), M)>,
}

impl<'d, M: Mode> core::fmt::Debug for Flex<'d, M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Flex").field("pin", &self.pin).finish()
    }
}

#[cfg(feature = "defmt")]
impl<'d, M: Mode> defmt::Format for Flex<'d, M> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Flex {{ pin: {} }}", self.pin)
    }
}

impl<'d> Flex<'d, Blocking> {
    /// Wrap the pin in a blocking `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self::new_inner(pin.pin_number(), ())
    }

    /// Wrap a pin without touching its configuration.
    ///
    /// For drivers that put the pin in a peripheral function themselves. Dropping the `Flex`
    /// disconnects the pin.
    #[inline]
    pub(crate) fn new_unconfigured(pin: Peri<'d, impl Pin>) -> Self {
        Self {
            pin: pin.pin_number(),
            waker: (),
            _phantom: PhantomData,
        }
    }
}

impl<'d> Flex<'d, Async> {
    /// Wrap the pin in an async `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    ///
    /// The pin's interrupt must be bound to [`InterruptHandler`] for this pin. It is enabled in the
    /// NVIC of the core this runs on, so wait on the pin from that core.
    #[inline]
    pub fn new_async<T: Pin>(pin: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd) -> Self {
        let this = Self::new_inner(pin.pin_number(), T::waker());
        unsafe { T::Interrupt::enable() };
        this
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        match self.is_input_inverted() {
            true => InputFuture::new(self.pin, self.waker, InterruptTrigger::LevelLow).await,
            false => InputFuture::new(self.pin, self.waker, InterruptTrigger::LevelHigh).await,
        }
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        match self.is_input_inverted() {
            true => InputFuture::new(self.pin, self.waker, InterruptTrigger::LevelHigh).await,
            false => InputFuture::new(self.pin, self.waker, InterruptTrigger::LevelLow).await,
        }
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        match self.is_input_inverted() {
            true => InputFuture::new(self.pin, self.waker, InterruptTrigger::EdgeLow).await,
            false => InputFuture::new(self.pin, self.waker, InterruptTrigger::EdgeHigh).await,
        }
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        match self.is_input_inverted() {
            true => InputFuture::new(self.pin, self.waker, InterruptTrigger::EdgeHigh).await,
            false => InputFuture::new(self.pin, self.waker, InterruptTrigger::EdgeLow).await,
        }
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(self.pin, self.waker, InterruptTrigger::AnyEdge).await;
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    #[inline]
    fn new_inner(pin: PinNumber, waker: M::IfAsync<&'static AtomicWaker>) -> Self {
        // Input buffer, pulls and output driver off. The output enable may have been left set by
        // a previous owner of the pin, so clear it before connecting the pin to SIO.
        pin.pad_ctrl().write(|_w| {
            #[cfg(feature = "_rp235x")]
            _w.set_iso(false);
        });
        pin.sio_oe().value_clr().write_value(pin.bit());

        pin.gpio().ctrl().write(|w| w.set_funcsel(SIO_FUNCSEL));

        Self {
            pin,
            waker,
            _phantom: PhantomData,
        }
    }

    #[inline]
    fn bit(&self) -> u32 {
        self.pin.bit()
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.pad_ctrl().modify(|w| {
            let (pu, pd) = match pull {
                Pull::Up => (true, false),
                Pull::Down => (false, true),
                Pull::None => (false, false),
            };
            w.set_pue(pu);
            w.set_pde(pd);
        });
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_drive(match strength {
                Drive::_2mA => pac::pads::vals::Drive::_2mA,
                Drive::_4mA => pac::pads::vals::Drive::_4mA,
                Drive::_8mA => pac::pads::vals::Drive::_8mA,
                Drive::_12mA => pac::pads::vals::Drive::_12mA,
            });
        });
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_slewfast(slew_rate == SlewRate::Fast);
        });
    }

    /// Set the pin's Schmitt trigger.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_schmitt(enable);
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        self.enable_input_buffer();
        self.pin.sio_oe().value_clr().write_value(self.bit())
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        self.enable_input_buffer();
        self.pin.sio_oe().value_set().write_value(self.bit())
    }

    /// Put the pin into disconnected mode.
    ///
    /// This disables the output driver, the input buffer and the internal weak pull-up and
    /// pull-down resistors.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.sio_oe().value_clr().write_value(self.bit());
        self.pin.pad_ctrl().write_clear(|w| {
            w.set_ie(true);
            w.set_pue(true);
            w.set_pde(true);
        });
    }

    /// Turn the input buffer on. It is off while the pin is disconnected, and it stays on in
    /// output mode so the pin level can be read back.
    #[inline]
    fn enable_input_buffer(&mut self) {
        self.pin.pad_ctrl().write_set(|w| w.set_ie(true));
    }

    /// Is the pin configured as an input?
    ///
    /// This is true after [`Self::set_as_input()`].
    #[inline]
    pub fn is_input(&self) -> bool {
        self.pin.pad_ctrl().read().ie() && !self.is_output()
    }

    /// Is the pin configured as an output?
    ///
    /// This is true after [`Self::set_as_output()`].
    #[inline]
    pub fn is_output(&self) -> bool {
        (self.pin.sio_oe().value().read() & self.bit()) != 0
    }

    /// Is the pin disconnected?
    ///
    /// This is true when both the output driver and the input buffer are disabled.
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        !self.pin.pad_ctrl().read().ie() && !self.is_output()
    }

    /// Toggle output pin.
    #[inline]
    pub fn toggle_set_as_output(&mut self) {
        self.enable_input_buffer();
        self.pin.sio_oe().value_xor().write_value(self.bit())
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_input_inversion(&mut self, invert: bool) {
        self.pin.gpio().ctrl().modify(|w| {
            w.set_inover(if invert {
                pac::io::vals::Inover::Invert
            } else {
                pac::io::vals::Inover::Normal
            })
        });
    }

    /// Get whether the input logic of this pin is inverted.
    #[inline]
    pub fn is_input_inverted(&self) -> bool {
        self.pin.gpio().ctrl().read().inover() == pac::io::vals::Inover::Invert
    }

    /// Configure the output enable inversion of this pin
    #[inline]
    pub fn set_output_enable_inversion(&mut self, invert: bool) {
        self.pin.gpio().ctrl().modify(|w| {
            w.set_oeover(if invert {
                pac::io::vals::Oeover::Invert
            } else {
                pac::io::vals::Oeover::Normal
            })
        })
    }

    /// Configure the output logic inversion of this pin.
    #[inline]
    pub fn set_output_inversion(&mut self, invert: bool) {
        self.pin.gpio().ctrl().modify(|w| {
            w.set_outover(if invert {
                pac::io::vals::Outover::Invert
            } else {
                pac::io::vals::Outover::Normal
            })
        });
    }

    /// Get whether the output logic of this pin is inverted.
    #[inline]
    pub fn is_output_inverted(&self) -> bool {
        self.pin.gpio().ctrl().read().outover() == pac::io::vals::Outover::Invert
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.sio_in().read() & self.bit() == 0
    }

    /// Returns current pin level
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.sio_out().value_set().write_value(self.bit())
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.sio_out().value_clr().write_value(self.bit())
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        (self.pin.sio_out().value().read() & self.bit()) == 0
    }

    /// What level output is set to
    #[inline]
    pub fn output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.sio_out().value_xor().write_value(self.bit())
    }

    /// Configure dormant wake.
    #[inline]
    pub fn dormant_wake(&mut self, cfg: DormantWakeConfig) -> DormantWake<'_> {
        let idx = self.pin.pin() as usize;
        self.pin.io().intr(idx / 8).write(|w| {
            w.set_edge_high(idx % 8, cfg.edge_high);
            w.set_edge_low(idx % 8, cfg.edge_low);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_set(|w| {
            w.set_edge_high(idx % 8, cfg.edge_high);
            w.set_edge_low(idx % 8, cfg.edge_low);
            w.set_level_high(idx % 8, cfg.level_high);
            w.set_level_low(idx % 8, cfg.level_low);
        });
        DormantWake {
            pin: self.pin,
            cfg,
            _phantom: PhantomData,
        }
    }

    /// Set the pin's pad isolation
    #[cfg(feature = "_rp235x")]
    #[inline]
    pub fn set_pad_isolation(&mut self, isolate: bool) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_iso(isolate);
        });
    }

    /// Give up the pin, leaving it configured.
    ///
    /// Dropping disconnects the pin, this doesn't. The pin keeps its configuration after the
    /// borrow it came from ends.
    #[inline]
    pub fn persist(self) {
        core::mem::forget(self);
    }

    /// The pin's number, for drivers that need it to pick a channel or function.
    #[inline]
    pub(crate) fn pin_number(&self) -> PinNumber {
        self.pin
    }

    /// Connect the pin to a peripheral.
    ///
    /// `funcsel` is the pin's `FUNCSEL` value for the peripheral, from the GPIO function table in
    /// the datasheet. Only the function changes: inversions set with the `set_*_inversion` methods
    /// are kept, so set them first to avoid a glitch. A pin starts with no inversions, and
    /// dropping a `Flex` clears them.
    #[inline]
    pub(crate) fn set_function(&mut self, funcsel: u8) {
        self.pin.gpio().ctrl().modify(|w| w.set_funcsel(funcsel));
    }

    /// Overwrite the pin's pad configuration.
    ///
    /// `f` starts from all-zero: input buffer, pulls and Schmitt trigger off, output enabled, 2 mA
    /// drive, slow slew and, on RP235x, pad not isolated.
    #[inline]
    pub(crate) fn write_pad(&mut self, f: impl FnOnce(&mut pac::pads::regs::GpioCtrl)) {
        self.pin.pad_ctrl().write(f);
    }

    /// Modify the pin's pad configuration.
    #[inline]
    pub(crate) fn modify_pad(&mut self, f: impl FnOnce(&mut pac::pads::regs::GpioCtrl)) {
        self.pin.pad_ctrl().modify(f);
    }
}

impl<'d, M: Mode> Drop for Flex<'d, M> {
    #[inline]
    fn drop(&mut self) {
        let idx = self.pin.pin() as usize;
        self.pin.pad_ctrl().write(|_| {});
        self.pin.gpio().ctrl().write(|w| {
            w.set_funcsel(pac::io::vals::Gpio0CtrlFuncsel::Null as _);
            w.set_inover(pac::io::vals::Inover::Normal);
            w.set_outover(pac::io::vals::Outover::Normal);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_clear(|w| {
            w.set_edge_high(idx % 8, true);
            w.set_edge_low(idx % 8, true);
            w.set_level_high(idx % 8, true);
            w.set_level_low(idx % 8, true);
        });
    }
}

/// Dormant wake driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DormantWake<'w> {
    pin: PinNumber,
    cfg: DormantWakeConfig,
    _phantom: PhantomData<&'w mut ()>,
}

impl<'w> Drop for DormantWake<'w> {
    fn drop(&mut self) {
        let idx = self.pin.pin() as usize;
        self.pin.io().intr(idx / 8).write(|w| {
            w.set_edge_high(idx % 8, self.cfg.edge_high);
            w.set_edge_low(idx % 8, self.cfg.edge_low);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_clear(|w| {
            w.set_edge_high(idx % 8, true);
            w.set_edge_low(idx % 8, true);
            w.set_level_high(idx % 8, true);
            w.set_level_low(idx % 8, true);
        });
    }
}

/// Bank and number of a pin, packed as `bank * 128 + pin`.
///
/// Plain data, it doesn't own the pin: drivers own pins through [`Flex`]. Holds the register
/// accessors for the pin.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct PinNumber(u8);

impl PinNumber {
    #[inline]
    pub(crate) const fn new(pin_bank: u8) -> Self {
        Self(pin_bank)
    }

    /// Bit of this pin in the SIO registers.
    #[inline]
    pub(crate) fn bit(self) -> u32 {
        1 << (self.pin() % 32)
    }

    #[inline]
    pub(crate) fn pin(self) -> u8 {
        self.0 & 0x7f
    }

    #[inline]
    pub(crate) fn bank(self) -> Bank {
        match self.0 >> 7 {
            #[cfg(feature = "qspi-as-gpio")]
            1 => Bank::Qspi,
            _ => Bank::Bank0,
        }
    }

    #[inline]
    pub(crate) fn io(self) -> pac::io::Io {
        match self.bank() {
            Bank::Bank0 => crate::pac::IO_BANK0,
            #[cfg(feature = "qspi-as-gpio")]
            Bank::Qspi => crate::pac::IO_QSPI,
        }
    }

    #[inline]
    pub(crate) fn gpio(self) -> pac::io::Gpio {
        self.io().gpio(self.pin() as _)
    }

    #[inline]
    pub(crate) fn pad_ctrl(self) -> Reg<pac::pads::regs::GpioCtrl, RW> {
        let block = match self.bank() {
            Bank::Bank0 => crate::pac::PADS_BANK0,
            #[cfg(feature = "qspi-as-gpio")]
            Bank::Qspi => crate::pac::PADS_QSPI,
        };
        block.gpio(self.pin() as _)
    }

    #[inline]
    pub(crate) fn sio_out(self) -> pac::sio::Gpio {
        if cfg!(feature = "rp2040") {
            SIO.gpio_out(self.bank() as _)
        } else {
            SIO.gpio_out((self.pin() / 32) as _)
        }
    }

    #[inline]
    pub(crate) fn sio_oe(self) -> pac::sio::Gpio {
        if cfg!(feature = "rp2040") {
            SIO.gpio_oe(self.bank() as _)
        } else {
            SIO.gpio_oe((self.pin() / 32) as _)
        }
    }

    #[inline]
    pub(crate) fn sio_in(self) -> Reg<u32, RW> {
        if cfg!(feature = "rp2040") {
            SIO.gpio_in(self.bank() as _)
        } else {
            SIO.gpio_in((self.pin() / 32) as _)
        }
    }

    #[inline]
    pub(crate) fn int_proc(self) -> pac::io::Int {
        let proc = SIO.cpuid().read();
        self.io().int_proc(proc as _)
    }
}

pub(crate) trait SealedPin {
    fn pin_number(&self) -> PinNumber;
}

/// Per-pin items that only exist on concrete pin types.
trait SealedPinStatic {
    /// This pin's number.
    const PIN_NUMBER: PinNumber;

    /// Waker of the async driver for this pin.
    ///
    /// Each pin has its own `static`, so wakers only take RAM for pins used in async mode.
    fn waker() -> &'static AtomicWaker;
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + SealedPin + SealedPinStatic + Sized + 'static {
    /// Interrupt for this pin's bank.
    type Interrupt: Interrupt;

    /// Returns the pin number within a bank
    #[inline]
    fn pin(&self) -> u8 {
        self.pin_number().pin()
    }

    /// Returns the bank of this pin
    #[inline]
    fn bank(&self) -> Bank {
        self.pin_number().bank()
    }
}

// ==========================

macro_rules! impl_pin {
    ($name:ident, $bank:expr, $pin_num:expr, $irq:ident) => {
        impl Pin for peripherals::$name {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
        impl SealedPinStatic for peripherals::$name {
            const PIN_NUMBER: PinNumber = PinNumber::new(($bank as u8) * 128 + $pin_num);

            #[inline]
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }
        impl SealedPin for peripherals::$name {
            #[inline]
            fn pin_number(&self) -> PinNumber {
                <Self as SealedPinStatic>::PIN_NUMBER
            }
        }
    };
}

impl_pin!(PIN_0, Bank::Bank0, 0, IO_IRQ_BANK0);
impl_pin!(PIN_1, Bank::Bank0, 1, IO_IRQ_BANK0);
impl_pin!(PIN_2, Bank::Bank0, 2, IO_IRQ_BANK0);
impl_pin!(PIN_3, Bank::Bank0, 3, IO_IRQ_BANK0);
impl_pin!(PIN_4, Bank::Bank0, 4, IO_IRQ_BANK0);
impl_pin!(PIN_5, Bank::Bank0, 5, IO_IRQ_BANK0);
impl_pin!(PIN_6, Bank::Bank0, 6, IO_IRQ_BANK0);
impl_pin!(PIN_7, Bank::Bank0, 7, IO_IRQ_BANK0);
impl_pin!(PIN_8, Bank::Bank0, 8, IO_IRQ_BANK0);
impl_pin!(PIN_9, Bank::Bank0, 9, IO_IRQ_BANK0);
impl_pin!(PIN_10, Bank::Bank0, 10, IO_IRQ_BANK0);
impl_pin!(PIN_11, Bank::Bank0, 11, IO_IRQ_BANK0);
impl_pin!(PIN_12, Bank::Bank0, 12, IO_IRQ_BANK0);
impl_pin!(PIN_13, Bank::Bank0, 13, IO_IRQ_BANK0);
impl_pin!(PIN_14, Bank::Bank0, 14, IO_IRQ_BANK0);
impl_pin!(PIN_15, Bank::Bank0, 15, IO_IRQ_BANK0);
impl_pin!(PIN_16, Bank::Bank0, 16, IO_IRQ_BANK0);
impl_pin!(PIN_17, Bank::Bank0, 17, IO_IRQ_BANK0);
impl_pin!(PIN_18, Bank::Bank0, 18, IO_IRQ_BANK0);
impl_pin!(PIN_19, Bank::Bank0, 19, IO_IRQ_BANK0);
impl_pin!(PIN_20, Bank::Bank0, 20, IO_IRQ_BANK0);
impl_pin!(PIN_21, Bank::Bank0, 21, IO_IRQ_BANK0);
impl_pin!(PIN_22, Bank::Bank0, 22, IO_IRQ_BANK0);
impl_pin!(PIN_23, Bank::Bank0, 23, IO_IRQ_BANK0);
impl_pin!(PIN_24, Bank::Bank0, 24, IO_IRQ_BANK0);
impl_pin!(PIN_25, Bank::Bank0, 25, IO_IRQ_BANK0);
impl_pin!(PIN_26, Bank::Bank0, 26, IO_IRQ_BANK0);
impl_pin!(PIN_27, Bank::Bank0, 27, IO_IRQ_BANK0);
impl_pin!(PIN_28, Bank::Bank0, 28, IO_IRQ_BANK0);
impl_pin!(PIN_29, Bank::Bank0, 29, IO_IRQ_BANK0);

#[cfg(feature = "rp235xb")]
impl_pin!(PIN_30, Bank::Bank0, 30, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_31, Bank::Bank0, 31, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_32, Bank::Bank0, 32, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_33, Bank::Bank0, 33, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_34, Bank::Bank0, 34, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_35, Bank::Bank0, 35, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_36, Bank::Bank0, 36, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_37, Bank::Bank0, 37, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_38, Bank::Bank0, 38, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_39, Bank::Bank0, 39, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_40, Bank::Bank0, 40, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_41, Bank::Bank0, 41, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, Bank::Bank0, 42, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, Bank::Bank0, 43, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_44, Bank::Bank0, 44, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_45, Bank::Bank0, 45, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, Bank::Bank0, 46, IO_IRQ_BANK0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, Bank::Bank0, 47, IO_IRQ_BANK0);

// TODO rp235x bank1 as gpio support
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SCLK, Bank::Qspi, 0, IO_IRQ_QSPI);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SS, Bank::Qspi, 1, IO_IRQ_QSPI);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD0, Bank::Qspi, 2, IO_IRQ_QSPI);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD1, Bank::Qspi, 3, IO_IRQ_QSPI);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD2, Bank::Qspi, 4, IO_IRQ_QSPI);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD3, Bank::Qspi, 5, IO_IRQ_QSPI);

// ====================

mod eh02 {
    use super::*;

    impl<'d, M: Mode> embedded_hal_02::digital::v2::InputPin for Input<'d, M> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::OutputPin for Output<'d> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

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

    impl<'d, M: Mode> embedded_hal_02::digital::v2::InputPin for OutputOpenDrain<'d, M> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::OutputPin for OutputOpenDrain<'d, M> {
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

    impl<'d, M: Mode> embedded_hal_02::digital::v2::StatefulOutputPin for OutputOpenDrain<'d, M> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::ToggleableOutputPin for OutputOpenDrain<'d, M> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::InputPin for Flex<'d, M> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::OutputPin for Flex<'d, M> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d, M> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, M: Mode> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d, M> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::ErrorType for Input<'d, M> {
    type Error = Infallible;
}

impl<'d, M: Mode> embedded_hal_1::digital::InputPin for Input<'d, M> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::OutputPin for Output<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::ErrorType for OutputOpenDrain<'d, M> {
    type Error = Infallible;
}

impl<'d, M: Mode> embedded_hal_1::digital::OutputPin for OutputOpenDrain<'d, M> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::StatefulOutputPin for OutputOpenDrain<'d, M> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::InputPin for OutputOpenDrain<'d, M> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::ErrorType for Flex<'d, M> {
    type Error = Infallible;
}

impl<'d, M: Mode> embedded_hal_1::digital::InputPin for Flex<'d, M> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::OutputPin for Flex<'d, M> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::StatefulOutputPin for Flex<'d, M> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Flex<'d, Async> {
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

impl<'d> embedded_hal_async::digital::Wait for Input<'d, Async> {
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

impl<'d> embedded_hal_async::digital::Wait for OutputOpenDrain<'d, Async> {
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
