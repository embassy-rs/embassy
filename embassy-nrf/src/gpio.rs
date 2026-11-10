//! General purpose input/output (GPIO) driver.
#![macro_use]

use core::convert::Infallible;
use core::future::Future;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpiote::GpiotePin;
use crate::interrupt::typelevel::{Binding, Handler};
use crate::mode::{Async, Blocking, Mode};
use crate::pac;
use crate::pac::common::{RW, Reg};
use crate::pac::gpio;
use crate::pac::gpio::vals;
#[cfg(not(feature = "_nrf51"))]
use crate::pac::shared::{regs::Psel, vals::Connect};

/// A GPIO port with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0, available on nRF9160 and all nRF52 and nRF51 MCUs.
    Port0,

    /// Port 1, only available on some MCUs.
    #[cfg(feature = "_gpio-p1")]
    Port1,

    /// Port 2, only available on some MCUs.
    #[cfg(feature = "_gpio-p2")]
    Port2,

    /// Port 3, only available on some MCUs.
    #[cfg(feature = "_gpio-p3")]
    Port3,
}

/// Pull setting for an input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// GPIO input driver.
pub struct Input<'d, M: Mode = Blocking> {
    pub(crate) pin: Flex<'d, M>,
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
    /// The pin's GPIOTE interrupt must be bound to [`InterruptHandler`] for this pin.
    #[inline]
    pub fn new_async<T: GpiotePin>(
        pin: Peri<'d, T>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        pull: Pull,
    ) -> Self {
        Self::new_inner(Flex::new_async(pin, irq), pull)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

impl<'d, M: Mode> Input<'d, M> {
    #[inline]
    fn new_inner(mut pin: Flex<'d, M>, pull: Pull) -> Self {
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

    /// Get the pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }
}

impl<M: Mode> Input<'static, M> {
    /// Persist the pin's configuration for the rest of the program's lifetime. This method should
    /// be preferred over [`core::mem::forget()`] because the `'static` bound prevents accidental
    /// reuse of the underlying peripheral.
    pub fn persist(self) {
        self.pin.persist()
    }
}

/// Digital input or output level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Drive strength settings for a given output level.
// These numbers match vals::Drive exactly so hopefully the compiler will unify them.
#[cfg(feature = "_nrf54l")]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LevelDrive {
    /// Disconnect (do not drive the output at all)
    Disconnect = 2,
    /// Standard
    Standard = 0,
    /// High drive
    High = 1,
    /// Extra high drive
    ExtraHigh = 3,
}

/// Drive strength settings for an output pin.
///
/// This is a combination of two drive levels, used when the pin is set
/// low and high respectively.
#[cfg(feature = "_nrf54l")]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutputDrive {
    low: LevelDrive,
    high: LevelDrive,
}

#[cfg(feature = "_nrf54l")]
#[allow(non_upper_case_globals)]
impl OutputDrive {
    /// Standard '0', standard '1'
    pub const Standard: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::Standard,
    };
    /// High drive '0', standard '1'
    pub const HighDrive0Standard1: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::Standard,
    };
    /// Standard '0', high drive '1'
    pub const Standard0HighDrive1: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::High,
    };
    /// High drive '0', high 'drive '1'
    pub const HighDrive: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::High,
    };
    /// Disconnect '0' standard '1' (normally used for wired-or connections)
    pub const Disconnect0Standard1: Self = Self {
        low: LevelDrive::Disconnect,
        high: LevelDrive::Standard,
    };
    /// Disconnect '0', high drive '1' (normally used for wired-or connections)
    pub const Disconnect0HighDrive1: Self = Self {
        low: LevelDrive::Disconnect,
        high: LevelDrive::High,
    };
    /// Standard '0'. disconnect '1' (also known as "open drain", normally used for wired-and connections)
    pub const Standard0Disconnect1: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::Disconnect,
    };
    /// High drive '0', disconnect '1' (also known as "open drain", normally used for wired-and connections)
    pub const HighDrive0Disconnect1: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::Disconnect,
    };
}

/// Drive strength settings for an output pin.
// These numbers match vals::Drive exactly so hopefully the compiler will unify them.
#[cfg(not(feature = "_nrf54l"))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OutputDrive {
    /// Standard '0', standard '1'
    Standard = 0,
    /// High drive '0', standard '1'
    HighDrive0Standard1 = 1,
    /// Standard '0', high drive '1'
    Standard0HighDrive1 = 2,
    /// High drive '0', high 'drive '1'
    HighDrive = 3,
    /// Disconnect '0' standard '1' (normally used for wired-or connections)
    Disconnect0Standard1 = 4,
    /// Disconnect '0', high drive '1' (normally used for wired-or connections)
    Disconnect0HighDrive1 = 5,
    /// Standard '0'. disconnect '1' (also known as "open drain", normally used for wired-and connections)
    Standard0Disconnect1 = 6,
    /// High drive '0', disconnect '1' (also known as "open drain", normally used for wired-and connections)
    HighDrive0Disconnect1 = 7,
}

/// GPIO output driver.
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] and [OutputDrive] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level, drive: OutputDrive) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_output(drive);

        Self { pin }
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

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
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
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }
}

impl Output<'static> {
    /// Persist the pin's configuration for the rest of the program's lifetime. This method should
    /// be preferred over [`core::mem::forget()`] because the `'static` bound prevents accidental
    /// reuse of the underlying peripheral.
    pub fn persist(self) {
        self.pin.persist()
    }
}

/// GPIO output open-drain driver.
///
/// The pin drives the line low when set low and leaves it floating (disconnected) when set
/// high, so the line can be shared with other open-drain devices (wired-and). The input buffer
/// stays connected, so the actual line level can be read back with [`is_high`](Self::is_high) and
/// friends.
pub struct OutputOpenDrain<'d, M: Mode = Blocking> {
    pub(crate) pin: Flex<'d, M>,
}

impl<'d> OutputOpenDrain<'d, Blocking> {
    /// Create a blocking GPIO output open-drain driver for a [Pin] with the provided [Level],
    /// [OutputDrive] and [Pull] configuration.
    ///
    /// `drive` selects the drive strength used when driving the line low. It should be one of the
    /// `*Disconnect1` variants (typically [`OutputDrive::Standard0Disconnect1`]), otherwise the pin
    /// will actively drive the line high as well.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level, drive: OutputDrive, pull: Pull) -> Self {
        Self::new_inner(Flex::new(pin), initial_output, drive, pull)
    }
}

impl<'d> OutputOpenDrain<'d, Async> {
    /// Create an async GPIO output open-drain driver for a [Pin] with the provided [Level],
    /// [OutputDrive] and [Pull] configuration.
    ///
    /// The pin's GPIOTE interrupt must be bound to [`InterruptHandler`] for this pin.
    ///
    /// `drive` selects the drive strength used when driving the line low. It should be one of the
    /// `*Disconnect1` variants (typically [`OutputDrive::Standard0Disconnect1`]), otherwise the pin
    /// will actively drive the line high as well.
    #[inline]
    pub fn new_async<T: GpiotePin>(
        pin: Peri<'d, T>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        initial_output: Level,
        drive: OutputDrive,
        pull: Pull,
    ) -> Self {
        Self::new_inner(Flex::new_async(pin, irq), initial_output, drive, pull)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

impl<'d, M: Mode> OutputOpenDrain<'d, M> {
    #[inline]
    fn new_inner(mut pin: Flex<'d, M>, initial_output: Level, drive: OutputDrive, pull: Pull) -> Self {
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_input_output(pull, drive);

        Self { pin }
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

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
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

    /// Get the pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }
}

impl<M: Mode> OutputOpenDrain<'static, M> {
    /// Persist the pin's configuration for the rest of the program's lifetime. This method should
    /// be preferred over [`core::mem::forget()`] because the `'static` bound prevents accidental
    /// reuse of the underlying peripheral.
    pub fn persist(self) {
        self.pin.persist()
    }
}

pub(crate) fn convert_drive(w: &mut pac::gpio::regs::PinCnf, drive: OutputDrive) {
    #[cfg(not(feature = "_nrf54l"))]
    {
        let drive = match drive {
            OutputDrive::Standard => vals::Drive::S0s1,
            OutputDrive::HighDrive0Standard1 => vals::Drive::H0s1,
            OutputDrive::Standard0HighDrive1 => vals::Drive::S0h1,
            OutputDrive::HighDrive => vals::Drive::H0h1,
            OutputDrive::Disconnect0Standard1 => vals::Drive::D0s1,
            OutputDrive::Disconnect0HighDrive1 => vals::Drive::D0h1,
            OutputDrive::Standard0Disconnect1 => vals::Drive::S0d1,
            OutputDrive::HighDrive0Disconnect1 => vals::Drive::H0d1,
        };
        w.set_drive(drive);
    }

    #[cfg(feature = "_nrf54l")]
    {
        fn convert(d: LevelDrive) -> vals::Drive {
            match d {
                LevelDrive::Disconnect => vals::Drive::D,
                LevelDrive::Standard => vals::Drive::S,
                LevelDrive::High => vals::Drive::H,
                LevelDrive::ExtraHigh => vals::Drive::E,
            }
        }

        w.set_drive0(convert(drive.low));
        w.set_drive1(convert(drive.high));
    }
}

fn convert_pull(pull: Pull) -> vals::Pull {
    match pull {
        Pull::None => vals::Pull::Disabled,
        Pull::Up => vals::Pull::Pullup,
        Pull::Down => vals::Pull::Pulldown,
    }
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d, M: Mode = Blocking> {
    pub(crate) pin: Peri<'d, AnyPin>,
    /// Waker for the pin's PORT event.
    waker: M::IfAsync<&'static AtomicWaker>,
    _mode: PhantomData<M>,
}

impl<'d> Flex<'d, Blocking> {
    /// Wrap the pin in a blocking `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self::new_inner(pin.into(), ())
    }
}

impl<'d> Flex<'d, Async> {
    /// Wrap the pin in an async `Flex`.
    ///
    /// The pin's GPIOTE interrupt must be bound to [`InterruptHandler`] for this pin.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new_async<T: GpiotePin>(
        pin: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        crate::gpiote::enable_port_interrupt::<T>();
        Self::new_inner(pin.into(), T::waker())
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    pub async fn wait_for_high(&mut self) {
        self.wait_for_sense(vals::Sense::High).await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.wait_for_sense(vals::Sense::Low).await
    }

    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.wait_for_low().await;
        self.wait_for_high().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.wait_for_high().await;
        self.wait_for_low().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    pub async fn wait_for_any_edge(&mut self) {
        let sense = match self.is_high() {
            true => vals::Sense::Low,
            false => vals::Sense::High,
        };
        self.wait_for_sense(sense).await
    }

    fn wait_for_sense(&mut self, sense: vals::Sense) -> PortInputFuture<'_> {
        // Discard a stale LATCH bit, so the PORT interrupt handler doesn't complete this wait early.
        // SENSE is disabled here, so the bit's DETECT signal is low and the clear takes effect.
        #[cfg(not(feature = "_nrf51"))]
        {
            let latch = self.pin.block().latch();
            let mask = 1 << self.pin._pin();
            if latch.read().0 & mask != 0 {
                latch.write(|w| w.0 = mask);
            }
        }

        self.pin.conf().modify(|w| w.set_sense(sense));
        PortInputFuture {
            pin: self.pin.reborrow(),
            waker: self.waker,
        }
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    #[inline]
    fn new_inner(pin: Peri<'d, AnyPin>, waker: M::IfAsync<&'static AtomicWaker>) -> Self {
        // Pin will be in disconnected state.
        Self {
            pin,
            waker,
            _mode: PhantomData,
        }
    }

    /// Put the pin into input mode.
    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::Input);
            w.set_input(vals::Input::Connect);
            w.set_pull(convert_pull(pull));
            convert_drive(w, OutputDrive::Standard);
            w.set_sense(vals::Sense::Disabled);
        });
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self, drive: OutputDrive) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::Output);
            w.set_input(vals::Input::Disconnect);
            w.set_pull(vals::Pull::Disabled);
            convert_drive(w, drive);
            w.set_sense(vals::Sense::Disabled);
        });
    }

    /// Put the pin into input + output mode.
    ///
    /// This is commonly used for "open drain" mode. If you set `drive = Standard0Disconnect1`,
    /// the hardware will drive the line low if you set it to low, and will leave it floating if you set
    /// it to high, in which case you can read the input to figure out whether another device
    /// is driving the line low.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_input_output(&mut self, pull: Pull, drive: OutputDrive) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::Output);
            w.set_input(vals::Input::Connect);
            w.set_pull(convert_pull(pull));
            convert_drive(w, drive);
            w.set_sense(vals::Sense::Disabled);
        });
    }

    /// Put the pin into disconnected mode.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.conf().write(|w| {
            w.set_input(vals::Input::Disconnect);
        });
    }

    /// Is the pin configured as an input?
    ///
    /// This is true after [`Self::set_as_input()`] or [`Self::set_as_input_output()`].
    #[inline]
    pub fn is_input(&self) -> bool {
        self.pin.conf().read().input() == vals::Input::Connect
    }

    /// Is the pin configured as an output?
    ///
    /// This is true after [`Self::set_as_output()`] or [`Self::set_as_input_output()`].
    #[inline]
    pub fn is_output(&self) -> bool {
        self.pin.conf().read().dir() == vals::Dir::Output
    }

    /// Is the pin disconnected?
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        let conf = self.pin.conf().read();
        conf.input() == vals::Input::Disconnect && conf.dir() == vals::Dir::Input
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.block().in_().read().pin(self.pin.pin() as _)
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Get the pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
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

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        if self.is_set_low() {
            self.set_high()
        } else {
            self.set_low()
        }
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.pin.set_low(),
            Level::High => self.pin.set_high(),
        }
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.block().out().read().pin(self.pin.pin() as _)
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Get the current output level.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.is_set_high().into()
    }
}

impl<M: Mode> Flex<'static, M> {
    /// Persist the pin's configuration for the rest of the program's lifetime. This method should
    /// be preferred over [`core::mem::forget()`] because the `'static` bound prevents accidental
    /// reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d, M: Mode> Drop for Flex<'d, M> {
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

/// GPIO interrupt handler for one pin.
///
/// Bind one handler per pin used with an async [`Input`], [`OutputOpenDrain`] or [`Flex`], on that
/// pin's GPIOTE interrupt ([`GpiotePin::Interrupt`]). All pins served by a GPIOTE instance share
/// its interrupt, so list one handler per pin:
///
/// ```rust,ignore
/// bind_interrupts!(struct Irqs {
///     GPIOTE => gpio::InterruptHandler<peripherals::P0_11>, gpio::InterruptHandler<peripherals::P0_12>;
/// });
/// ```
///
/// The handler only services its own pin, so the same interrupt can also be bound to
/// [`gpiote::InterruptHandler`](crate::gpiote::InterruptHandler) for GPIOTE channels, and to your
/// own [`Handler`] for pins used in blocking mode. If your handler uses the PORT event, it must
/// clear `EVENTS_PORT` first and then clear only its own pins' bits in `LATCH`. While any `LATCH`
/// bit of a port stays set, no new PORT event is generated for that port.
pub struct InterruptHandler<T: GpiotePin> {
    _phantom: PhantomData<T>,
}

impl<T: GpiotePin> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let pin = AnyPin { pin_port: T::PIN_PORT };

        // Clear the event before any LATCH bit, so the PORT event generated when a LATCH clear
        // leaves other bits set isn't lost.
        crate::gpiote::clear_port_event(T::regs());

        // On nRF51 there's no LATCH, check the level against the SENSE setting instead.
        #[cfg(feature = "_nrf51")]
        let fired = match pin.conf().read().sense() {
            vals::Sense::High => pin.block().in_().read().pin(pin._pin() as _),
            vals::Sense::Low => !pin.block().in_().read().pin(pin._pin() as _),
            _ => false,
        };
        #[cfg(not(feature = "_nrf51"))]
        let fired = pin.block().latch().read().0 & (1 << pin._pin()) != 0;

        if fired {
            // Disable SENSE before clearing LATCH: a LATCH bit doesn't clear while its DETECT signal is high.
            // The future sees SENSE disabled and completes.
            pin.conf().modify(|w| w.set_sense(vals::Sense::Disabled));
            #[cfg(not(feature = "_nrf51"))]
            pin.block().latch().write(|w| w.0 = 1 << pin._pin());
            T::waker().wake();
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct PortInputFuture<'a> {
    pin: Peri<'a, AnyPin>,
    waker: &'static AtomicWaker,
}

impl<'a> Unpin for PortInputFuture<'a> {}

impl<'a> Drop for PortInputFuture<'a> {
    fn drop(&mut self) {
        self.pin.conf().modify(|w| w.set_sense(vals::Sense::Disabled));
    }
}

impl<'a> Future for PortInputFuture<'a> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.waker.register(cx.waker());

        if self.pin.conf().read().sense() == vals::Sense::Disabled {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub(crate) trait SealedPin {
    fn pin_port(&self) -> u8;

    #[inline]
    fn _pin(&self) -> u8 {
        cfg_select! {
            feature = "_gpio-p1" => self.pin_port() % 32,
            _ => self.pin_port(),
        }
    }

    #[inline]
    fn block(&self) -> gpio::Gpio {
        match self.pin_port() / 32 {
            #[cfg(feature = "_nrf51")]
            0 => pac::GPIO,
            #[cfg(not(feature = "_nrf51"))]
            0 => pac::P0,
            #[cfg(feature = "_gpio-p1")]
            1 => pac::P1,
            #[cfg(feature = "_gpio-p2")]
            2 => pac::P2,
            #[cfg(feature = "_gpio-p3")]
            3 => pac::P3,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    fn conf(&self) -> Reg<gpio::regs::PinCnf, RW> {
        self.block().pin_cnf(self._pin() as usize)
    }

    /// Set the output as high.
    #[inline]
    fn set_high(&self) {
        self.block().outset().write(|w| w.set_pin(self._pin() as _, true))
    }

    /// Set the output as low.
    #[inline]
    fn set_low(&self) {
        self.block().outclr().write(|w| w.set_pin(self._pin() as _, true))
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Number of the pin within the port (0..31)
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Port of the pin
    #[inline]
    fn port(&self) -> Port {
        match self.pin_port() / 32 {
            0 => Port::Port0,
            #[cfg(feature = "_gpio-p1")]
            1 => Port::Port1,
            #[cfg(feature = "_gpio-p2")]
            2 => Port::Port2,
            #[cfg(feature = "_gpio-p3")]
            3 => Port::Port3,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// Peripheral port register value
    #[inline]
    #[cfg(not(feature = "_nrf51"))]
    fn psel_bits(&self) -> pac::shared::regs::Psel {
        pac::shared::regs::Psel(self.pin_port() as u32)
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pub(crate) pin_port: u8,
}

impl AnyPin {
    /// Create an [AnyPin] for a specific pin.
    ///
    /// # Safety
    /// - `pin_port` should not in use by another driver.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }
}

impl_peripheral!(AnyPin);
impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

// ====================

#[cfg(not(feature = "_nrf51"))]
pub(crate) trait PselBits {
    fn psel_bits(&self) -> pac::shared::regs::Psel;
}

#[cfg(not(feature = "_nrf51"))]
impl<'a, P: Pin> PselBits for Option<Peri<'a, P>> {
    #[inline]
    fn psel_bits(&self) -> pac::shared::regs::Psel {
        match self {
            Some(pin) => pin.psel_bits(),
            None => DISCONNECTED,
        }
    }
}

#[cfg(not(feature = "_nrf51"))]
pub(crate) const DISCONNECTED: Psel = Psel(1 << 31);

#[cfg(not(feature = "_nrf51"))]
#[allow(dead_code)]
pub(crate) fn deconfigure_pin(psel: Psel) {
    if psel.connect() == Connect::Disconnected {
        return;
    }
    unsafe { AnyPin::steal(psel.0 as _) }.conf().write(|w| {
        w.set_input(vals::Input::Disconnect);
    })
}

// ====================

macro_rules! impl_pin {
    ($type:ident, $port_num:expr, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$type {}
        impl crate::gpio::SealedPin for peripherals::$type {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }

        impl From<peripherals::$type> for crate::gpio::AnyPin {
            fn from(_val: peripherals::$type) -> Self {
                Self {
                    pin_port: $port_num * 32 + $pin_num,
                }
            }
        }

        // On nRF54L only some ports have a GPIOTE, the chip file lists them with `impl_gpiote_pin!`.
        #[cfg(not(feature = "_nrf54l"))]
        impl_gpiote_pin!($type, $port_num, $pin_num);
    };
}

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

    /// Implement [`embedded_hal_02::digital::v2::InputPin`] for [`Flex`];
    ///
    /// If the pin is not in input mode the result is unspecified.
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

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }

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

impl<'d, M: Mode> embedded_hal_1::digital::InputPin for OutputOpenDrain<'d, M> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
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

impl<'d, M: Mode> embedded_hal_1::digital::ErrorType for Flex<'d, M> {
    type Error = Infallible;
}

/// Implement [embedded_hal_1::digital::InputPin] for [`Flex`];
///
/// If the pin is not in input mode the result is unspecified.
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

impl<'d> embedded_hal_async::digital::Wait for Input<'d, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_high().await)
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_low().await)
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_rising_edge().await)
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_falling_edge().await)
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_any_edge().await)
    }
}

impl<'d> embedded_hal_async::digital::Wait for OutputOpenDrain<'d, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_high().await)
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_low().await)
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_rising_edge().await)
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_falling_edge().await)
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_any_edge().await)
    }
}

impl<'d> embedded_hal_async::digital::Wait for Flex<'d, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_high().await)
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_low().await)
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_rising_edge().await)
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_falling_edge().await)
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_any_edge().await)
    }
}
