#![macro_use]

use core::convert::Infallible;
use core::future::{Future, poll_fn};
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;
use maitake_sync::WaitMap;
use mspm0_metapac::iomux::regs::Pincm;

use crate::interrupt::typelevel::Handler;
use crate::pac::gpio::vals::*;
use crate::pac::gpio::{self};
#[cfg(all(feature = "rt", any(gpioa_interrupt, gpiob_interrupt)))]
use crate::pac::interrupt;
use crate::pac::{self};

pub struct Blocking {}
impl crate::sealed::Sealed for Blocking {}
impl GpioMode for Blocking {}

pub struct InterruptHandler<T: Pin> {
    _marker: PhantomData<T>,
}

impl<T: Pin> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // TODO: This code is not optimal for multiple bindings.
        //
        // For example binding PA0 and PA1 would result in 2 reads of MIS and 2 modifications of IMASK.
        let info = T::info();
        let state = T::state();
        let block = info.block();

        let mis = block.cpu_int().mis().read();

        if mis.dio(info.pin()) {
            // Notify the future that an edge event has occurred by masking the interrupt for this pin.
            //
            // This needs to be performed before waking and in a critical section because the thread the
            // waker wake may be higher priority than the interrupt handler.
            critical_section::with(|_cs| {
                block.cpu_int().imask().modify(|w| {
                    w.set_dio(info.pin(), false);
                });
            });

            state.waker.wake();
        }
    }
}

pub struct Async {
    state: &'static State,
}
impl crate::sealed::Sealed for Async {}
impl GpioMode for Async {}

pub trait GpioMode: crate::sealed::Sealed {}

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
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Port {
    /// Port A.
    PortA = 0,

    /// Port B.
    #[cfg(gpio_pb)]
    PortB = 1,

    /// Port C.
    #[cfg(gpio_pc)]
    PortC = 2,
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d, Mode: GpioMode> {
    // pin: Peri<'d, AnyPin>,
    pin: PinBorrow<'d>,
    mode: Mode,
}

impl<'d, Mode: GpioMode> Flex<'d, Mode> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new_blocking(pin: Peri<'d, impl Pin>) -> Flex<'d, Blocking> {
        // Pin will be in disconnected state.
        Flex {
            pin: PinBorrow::new(pin),
            mode: Blocking {},
        }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_pipd(matches!(pull, Pull::Down));
            w.set_pipu(matches!(pull, Pull::Up));
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(false);
            w.set_pc(true);
            w.set_inena(true);
        });

        self.pin.info.block().doeclr31_0().write(|w| {
            w.set_dio(self.pin.info.pin(), true);
        });
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(false);
            w.set_pc(true);
            w.set_inena(false);
        });

        self.pin.info.block().doeset31_0().write(|w| {
            w.set_dio(self.pin.info.pin(), true);
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
    #[inline]
    pub fn set_as_input_output(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(true);
            w.set_pc(true);
            w.set_inena(true);
            w.set_pipd(false);
            w.set_pipu(false);
        });

        // Enable output driver (DOE) - required for open-drain to drive low
        self.pin.info.block().doeset31_0().write(|w| {
            w.set_dio(self.pin.info.pin(), true);
        });
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the internal weak pull-up and pull-down resistors.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.disconnect();
    }

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to both the input and output path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_inv(invert);
        });
    }

    // TODO: drive strength, hysteresis, wakeup enable, wakeup compare

    /// Put the pin into the PF mode, unchecked.
    ///
    /// This puts the pin into the PF mode, with the request number. This is completely unchecked,
    /// it can attach the pin to literally any peripheral, so use with care. In addition the pin
    /// peripheral is connected in the iomux.
    ///
    /// The peripheral attached to the pin depends on the part in use. Consult the datasheet
    /// or technical reference manual for additional details.
    #[inline]
    pub fn set_pf_unchecked(&mut self, pf: u8) {
        // Per SLAU893 and SLAU846B, PF is only 6 bits
        assert_eq!(pf & 0xC0, 0, "PF is out of range");

        let pincm = pac::IOMUX.pincm(self.pin.info.pincm_index());

        pincm.modify(|w| {
            w.set_pf(pf);
            // If the PF is manually set, connect the pin
            w.set_pc(true);
        });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.info.block().din31_0().read().dio(self.pin.info.pin())
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        let info = self.pin.info;
        info.block().doutset31_0().write(|w| {
            w.set_dio(info.pin(), true);
        });
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        let info = self.pin.info;
        info.block().doutclr31_0().write(|w| {
            w.set_dio(info.pin(), true);
        });
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        let info = self.pin.info;
        info.block().douttgl31_0().write(|w| {
            w.set_dio(info.pin(), true);
        });
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Get the current pin output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        let info = self.pin.info;
        info.block().dout31_0().read().dio(info.pin())
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }
}

impl<'d> Flex<'d, Async> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new<T: Pin>(pin: Peri<'d, T>) -> Self {
        // Pin will be in disconnected state.
        Flex {
            pin: PinBorrow::new(pin),
            mode: Async { state: T::state() },
        }
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        if self.is_high() {
            return;
        }

        self.wait_for_rising_edge().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        if self.is_low() {
            return;
        }

        self.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> {
        // Per https://tweedegolf.nl/en/blog/235/debloat-your-async-rust
        //
        // We match the async pass-through suggestion to reduce async bloat.
        self.wait_inner(Polarity::Rise)
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> {
        self.wait_inner(Polarity::Fall)
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> {
        self.wait_inner(Polarity::RiseFall)
    }

    fn wait_inner(&mut self, polarity: Polarity) -> impl Future<Output = ()> {
        let info = self.pin.info;
        let pin = info.pin();
        let state = self.mode.state;
        let block = info.block();

        // Selecting the event to trigger. A RMW operation.
        critical_section::with(|_cs| {
            if pin >= 16 {
                block.polarity31_16().modify(|w| {
                    w.set_dio(pin - 16, polarity);
                });
            } else {
                block.polarity15_0().modify(|w| {
                    w.set_dio(pin, polarity);
                });
            };
        });

        // Clear previous edge events. This is done after setting the event to listen for to avoid a redundant write.
        block.cpu_int().iclr().write(|w| {
            w.set_dio(pin, true);
        });

        // FIXME: Get waker state
        poll_fn(move |cx| {
            // Register waker before checking if event has happened.
            state.waker.register(cx.waker());

            if block.cpu_int().ris().read().dio(pin) {
                return Poll::Ready(());
            }

            // Pin singletons are Send and share the same IMASK register. Therefore enabling the interrupt
            // for this pin must be done in a critical section.
            critical_section::with(|_cs| {
                block.cpu_int().imask().modify(|w| {
                    w.set_dio(pin, true);
                });
            });

            Poll::Pending
        })
    }
}

impl<'d, Mode: GpioMode> Drop for Flex<'d, Mode> {
    #[inline]
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

/// GPIO input driver.
pub struct Input<'d, Mode: GpioMode> {
    pin: Flex<'d, Mode>,
}

impl<'d, Mode: GpioMode> Input<'d, Mode> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new_blocking(pin: Peri<'d, impl Pin>, pull: Pull) -> Input<'d, Blocking> {
        let mut pin = Flex::<Blocking>::new_blocking(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Input { pin }
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

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }
}

impl<'d> Input<'d, Async> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new<T: Pin>(pin: Peri<'d, T>, pull: Pull) -> Input<'d, Async> {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// GPIO output driver.
///
/// Note that pins will **return to their floating state** when `Output` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `Output`, or pass it to [`core::mem::forget`].
pub struct Output<'d> {
    pin: Flex<'d, Blocking>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::<Blocking>::new_blocking(pin);
        pin.set_as_output();
        pin.set_level(initial_output);
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

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }
}

/// GPIO output open-drain driver.
///
/// Note that pins will **return to their floating state** when `OutputOpenDrain` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `OutputOpenDrain`, or pass it to [`core::mem::forget`].
pub struct OutputOpenDrain<'d, Mode: GpioMode> {
    pin: Flex<'d, Mode>,
}

impl<'d, Mode: GpioMode> OutputOpenDrain<'d, Mode> {
    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new_blocking(pin: Peri<'d, impl Pin>, initial_output: Level) -> OutputOpenDrain<'d, Blocking> {
        let mut pin = Flex::<Blocking>::new_blocking(pin);
        pin.set_level(initial_output);
        pin.set_as_input_output();
        OutputOpenDrain { pin }
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

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }
}

impl<'d> OutputOpenDrain<'d, Async> {
    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_input_output();
        Self { pin }
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

// FIXME: Remove AnyPin once done
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

impl Pin for AnyPin {
    // FIXME: Wrong, but remove AnyPin
    type Interrupt = crate::interrupt::typelevel::GROUP0;
}
impl SealedPin for AnyPin {
    fn info() -> &'static Info {
        todo!("Remove AnyPin")
    }

    fn state() -> &'static State {
        todo!("Remove AnyPin")
    }

    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    type Interrupt: crate::interrupt::typelevel::Interrupt;

    /// The index of this pin in PINCM (pin control management) registers.
    #[inline]
    fn pin_cm(&self) -> u8 {
        self._pin_cm()
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::ErrorType for Flex<'d, Mode> {
    type Error = Infallible;
}

impl<'d, Mode: GpioMode> embedded_hal::digital::InputPin for Flex<'d, Mode> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::OutputPin for Flex<'d, Mode> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::StatefulOutputPin for Flex<'d, Mode> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
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

impl<'d, Mode: GpioMode> embedded_hal::digital::ErrorType for Input<'d, Mode> {
    type Error = Infallible;
}

impl<'d, Mode: GpioMode> embedded_hal::digital::InputPin for Input<'d, Mode> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
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

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::ErrorType for OutputOpenDrain<'d, Mode> {
    type Error = Infallible;
}

impl<'d, Mode: GpioMode> embedded_hal::digital::InputPin for OutputOpenDrain<'d, Mode> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::OutputPin for OutputOpenDrain<'d, Mode> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d, Mode: GpioMode> embedded_hal::digital::StatefulOutputPin for OutputOpenDrain<'d, Mode> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
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

#[cfg_attr(mspm0g518x, allow(dead_code))]
#[derive(Copy, Clone)]
pub struct PfType {
    pull: Pull,
    input: bool,
    invert: bool,
}

impl PfType {
    pub const fn input(pull: Pull, invert: bool) -> Self {
        Self {
            pull,
            input: true,
            invert,
        }
    }

    pub const fn output(pull: Pull, invert: bool) -> Self {
        Self {
            pull,
            input: false,
            invert,
        }
    }
}

/// Pin configuration.
///
/// This is a lower level interface to configure a pin.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PinConfig(Pincm);

impl PinConfig {
    #[inline(always)]
    pub const fn default() -> Self {
        Self(Pincm(0))
    }

    #[inline(always)]
    pub const fn pf(mut self, pf: u8) -> Self {
        self.0.set_pf(pf);
        self
    }

    #[inline(always)]
    pub const fn pc(mut self, pc: bool) -> Self {
        self.0.set_pc(pc);
        self
    }

    #[inline(always)]
    pub const fn pipd(mut self, pipd: bool) -> Self {
        self.0.set_pipd(pipd);
        self
    }

    #[inline(always)]
    pub const fn pipu(mut self, pipu: bool) -> Self {
        self.0.set_pipu(pipu);
        self
    }

    #[inline(always)]
    pub const fn inena(mut self, inena: bool) -> Self {
        self.0.set_inena(inena);
        self
    }

    #[inline(always)]
    pub const fn hysten(mut self, hysten: bool) -> Self {
        self.0.set_hysten(hysten);
        self
    }

    #[inline(always)]
    pub const fn drv(mut self, drv: bool) -> Self {
        self.0.set_drv(drv);
        self
    }

    #[inline(always)]
    pub const fn hiz1(mut self, hiz1: bool) -> Self {
        self.0.set_hiz1(hiz1);
        self
    }

    #[inline(always)]
    pub const fn inv(mut self, inv: bool) -> Self {
        self.0.set_inv(inv);
        self
    }
}

/// A borrowed pin.
///
/// This type is intended to be used by drivers which need to state that a pin is being borrowed.
#[must_use = "Dropping this type will disconnect the pin"]
pub struct PinBorrow<'d> {
    info: &'d Info,
}

impl<'d> PinBorrow<'d> {
    #[inline]
    pub fn new<T: Pin>(_p: Peri<'d, T>) -> Self {
        Self { info: T::info() }
    }

    #[inline]
    pub fn with_config<T: Pin>(p: Peri<'d, T>, config: PinConfig) -> Self {
        let this = Self::new(p);
        let pincm_index = this.info.pincm_index();
        pac::IOMUX.pincm(pincm_index).write_value(config.0);
        this
    }

    #[inline]
    pub fn update_config(&mut self, config: PinConfig) {
        let pincm_index = self.info.pincm_index();
        pac::IOMUX.pincm(pincm_index).write_value(config.0);
    }

    /// Get the current pin function set for this pin.
    #[inline]
    pub fn pf(&self) -> u8 {
        let pincm_index = self.info.pincm_index();
        pac::IOMUX.pincm(pincm_index).read().pf()
    }

    /// Set the pin function for this pin without changing any other configuration.
    #[inline]
    pub fn set_pf(&mut self, pf: u8) {
        let pincm_index = self.info.pincm_index();
        pac::IOMUX.pincm(pincm_index).modify(|w| {
            w.set_pf(pf);
        });
    }

    /// Set the pin as "disconnected".
    ///
    /// This will disable peripheral connections, pull-up, pull-down resistors and pin inversion,
    #[inline]
    pub fn disconnect(&mut self) {
        let pincm_index = self.info.pincm_index();
        // This will write PF = 0, PC = 0 and disconnect the output.
        //
        // This will also disable other things like pull ups, open drain, etc.
        pac::IOMUX.pincm(pincm_index).write_value(Pincm::default());
    }
}

/// Verify that PinBorrow is has niche optimizations.
const _: () = core::assert!(
    size_of::<PinBorrow>() == size_of::<Option<PinBorrow>>(),
    "Option<PinBorrow> is not niche optimized"
);

impl<'d> Drop for PinBorrow<'d> {
    #[inline]
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// The pin function to disconnect peripherals from the pin.
///
/// This is also the pin function used to connect to analog peripherals, such as an ADC.
const DISCONNECT_PF: u8 = 0;

/// The pin function for the GPIO peripheral.
///
/// This is fixed to `1` for every part.
const GPIO_PF: u8 = 1;

macro_rules! impl_pin {
    ($name: ident, $port: expr, $pin_num: expr) => {
        impl crate::gpio::Pin for crate::peripherals::$name {
            // FIXME: This is wrong, but for compilation it is needed
            type Interrupt = crate::interrupt::typelevel::GROUP0;
        }
        impl crate::gpio::SealedPin for crate::peripherals::$name {
            #[inline]
            fn info() -> &'static crate::gpio::Info {
                todo!("implement this")
            }

            #[inline]
            fn state() -> &'static crate::gpio::State {
                static STATE: crate::gpio::State = crate::gpio::State::new();
                &STATE
            }

            #[inline]
            fn pin_port(&self) -> u8 {
                ($port as u8) * 32 + $pin_num
            }
        }

        impl From<crate::peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: crate::peripherals::$name) -> Self {
                Self {
                    pin_port: crate::gpio::SealedPin::pin_port(&val),
                }
            }
        }
    };
}

/// GPIO pin info.
pub(crate) struct Info {
    /// The packed pin number and port id.
    pin_port: u8,

    /// The index of the pincm used for this pin.
    ///
    /// Only used if pin number does not match pincm order.
    #[cfg(any(gpio_pb, gpio_pc))]
    pincm_index: u8,
}

impl Info {
    #[inline]
    pub const fn new(port: Port, pin: u8, pincm_index: u8) -> Self {
        core::assert!(pin < 32, "Pin cannot be 32 or greater");
        core::assert!(
            (port as u8) < 8,
            "Port cannot be greater than 8 (implementation assumes 8 GPIO instances at most)"
        );
        let pin_port = (port as u8 * 32) + pin;

        // If GPIOB and GPIOC are not available we can optimize the size of info by
        #[cfg(not(any(gpio_pb, gpio_pc)))]
        {
            core::assert!(
                pin == pincm_index,
                "This part has one GPIO instance but the pin number and pincm registers do not have the same order",
            );
        }

        Self {
            pin_port,
            #[cfg(any(gpio_pb, gpio_pc))]
            pincm_index,
        }
    }

    /// Pin number.
    #[inline]
    pub const fn pin(&self) -> usize {
        (self.pin_port % 32) as usize
    }

    /// Index of the PINCM register for this pin in the [IOMUX](pac::IOMUX).
    #[inline]
    pub const fn pincm_index(&self) -> usize {
        cfg_select! {
            any(gpio_pb, gpio_pc) => self.pincm_index as usize,
            _ => self.info.pin_port,
        }
    }

    #[inline]
    pub const fn port(&self) -> Port {
        match self.pin_port / 32 {
            0 => Port::PortA,
            #[cfg(gpio_pb)]
            1 => Port::PortB,
            #[cfg(gpio_pc)]
            2 => Port::PortC,
            // SAFETY: Info::new and the device configuration ensure the port is valid.
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    pub const fn block(&self) -> gpio::Gpio {
        match self.port() {
            Port::PortA => pac::GPIOA,
            #[cfg(gpio_pb)]
            Port::PortB => pac::GPIOB,
            #[cfg(gpio_pc)]
            Port::PortC => pac::GPIOC,
        }
    }
}

pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

/// Wait map for GPIO wakers
///
/// This map must **never** be closed because gpio wakers may be used forever.
static GPIO_WAIT_MAP: WaitMap<u8, ()> = WaitMap::new();

pub(crate) trait SealedPin {
    fn info() -> &'static Info;

    fn state() -> &'static State;

    fn pin_port(&self) -> u8;

    fn _pin_cm(&self) -> u8 {
        // Some parts like the MSPM0L222x have pincm mappings all over the place.
        crate::gpio_pincm(self.pin_port())
    }

    fn bit_index(&self) -> usize {
        (self.pin_port() % 32) as usize
    }

    #[inline]
    fn set_as_analog(&self) {
        let pincm = pac::IOMUX.pincm(self._pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(DISCONNECT_PF);
            w.set_pipu(false);
            w.set_pipd(false);
        });
    }

    #[cfg_attr(mspm0g518x, allow(dead_code))]
    fn update_pf(&self, ty: PfType) {
        let pincm = pac::IOMUX.pincm(self._pin_cm() as usize);
        let pf = pincm.read().pf();

        set_pf(self._pin_cm() as usize, pf, ty);
    }

    #[cfg_attr(mspm0g518x, allow(dead_code))]
    fn set_as_pf(&self, pf: u8, ty: PfType) {
        set_pf(self._pin_cm() as usize, pf, ty)
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the internal weak pull-up and pull-down resistors.
    #[inline]
    #[cfg_attr(mspm0g518x, allow(dead_code))]
    fn set_as_disconnected(&self) {
        self.set_as_analog();
    }

    #[inline]
    fn block(&self) -> gpio::Gpio {
        match self.pin_port() / 32 {
            0 => pac::GPIOA,
            #[cfg(gpio_pb)]
            1 => pac::GPIOB,
            #[cfg(gpio_pc)]
            2 => pac::GPIOC,
            _ => unreachable!(),
        }
    }
}

#[inline(never)]
fn set_pf(pincm: usize, pf: u8, ty: PfType) {
    pac::IOMUX.pincm(pincm).modify(|w| {
        w.set_pf(pf);
        w.set_pc(true);
        w.set_pipu(ty.pull == Pull::Up);
        w.set_pipd(ty.pull == Pull::Down);
        w.set_inena(ty.input);
        w.set_inv(ty.invert);
    });
}

pub(crate) fn init(gpio: gpio::Gpio) {
    gpio.gprcm().rstctl().write(|w| {
        w.set_resetstkyclr(true);
        w.set_resetassert(true);
        w.set_key(ResetKey::Key);
    });

    gpio.gprcm().pwren().write(|w| {
        w.set_enable(true);
        w.set_key(PwrenKey::Key);
    });

    gpio.evt_mode().modify(|w| {
        // The CPU will clear it's own interrupts
        w.set_cpu_cfg(EvtCfg::Software);
    });
}

#[cfg(feature = "rt")]
fn irq_handler(gpio: gpio::Gpio, port: Port) {
    use crate::BitIter;
    // Only consider pins which have interrupts unmasked.

    let bits = gpio.cpu_int().mis().read().0;

    for i in BitIter(bits) {
        let id = ((port as u8) * 32) + i as u8;
        let _ = GPIO_WAIT_MAP.wake(&id, ());

        // Notify the future that an edge event has occurred by masking the interrupt for this pin.
        gpio.cpu_int().imask().modify(|w| {
            w.set_dio(i as usize, false);
        });
    }
}

#[cfg(all(gpioa_interrupt, gpioa_group))]
compile_error!("gpioa_interrupt and gpioa_group are mutually exclusive cfgs");
#[cfg(all(gpiob_interrupt, gpiob_group))]
compile_error!("gpiob_interrupt and gpiob_group are mutually exclusive cfgs");

// C110x and L110x have a dedicated interrupts just for GPIOA.
//
// These chips do not have a GROUP1 interrupt.
#[cfg(all(feature = "rt", gpioa_interrupt))]
#[interrupt]
fn GPIOA() {
    irq_handler(pac::GPIOA, Port::PortA);
}

#[cfg(all(feature = "rt", gpiob_interrupt))]
#[interrupt]
fn GPIOB() {
    irq_handler(pac::GPIOB, Port::PortB);
}

// These symbols are weakly defined as DefaultHandler and are called by the interrupt group implementation.
//
// Defining these as no_mangle is required so that the linker will pick these over the default handler.

#[cfg(all(feature = "rt", gpioa_group))]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn GPIOA() {
    irq_handler(pac::GPIOA, Port::PortA);
}

#[cfg(all(feature = "rt", gpiob_group))]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn GPIOB() {
    irq_handler(pac::GPIOB, Port::PortB);
}

#[cfg(all(feature = "rt", gpioc_group))]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn GPIOC() {
    irq_handler(pac::GPIOC, Port::PortC);
}
