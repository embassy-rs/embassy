//! GPIO

use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::enable_and_reset;
use crate::iopctl::IopctlPin;
pub use crate::iopctl::{AnyPin, DriveMode, DriveStrength, Function, Inverter, Pull, SlewRate};
use crate::sealed::Sealed;
use crate::{interrupt, peripherals, Peri, PeripheralType};

// This should be unique per IMXRT package
const PORT_COUNT: usize = 8;

/// Digital input or output level.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logic Low
    Low,
    /// Logic High
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

/// Interrupt trigger levels.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptType {
    /// Trigger on level.
    Level,
    /// Trigger on edge.
    Edge,
}

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn GPIO_INTA() {
    irq_handler(&GPIO_WAKERS);
}

#[cfg(feature = "rt")]
struct BitIter(u32);

#[cfg(feature = "rt")]
impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

#[cfg(feature = "rt")]
fn irq_handler(port_wakers: &[Option<&PortWaker>]) {
    let reg = unsafe { crate::pac::Gpio::steal() };

    for (port, port_waker) in port_wakers.iter().enumerate() {
        let Some(port_waker) = port_waker else {
            continue;
        };

        let stat = reg.intstata(port).read().bits();
        for pin in BitIter(stat) {
            // Clear the interrupt from this pin
            reg.intstata(port).write(|w| unsafe { w.status().bits(1 << pin) });
            // Disable interrupt from this pin
            reg.intena(port)
                .modify(|r, w| unsafe { w.int_en().bits(r.int_en().bits() & !(1 << pin)) });

            let Some(waker) = port_waker.get_waker(pin as usize) else {
                continue;
            };

            waker.wake();
        }
    }
}

/// Initialization Logic
/// Note: GPIO port clocks are initialized in the clocks module.
pub(crate) fn init() {
    // Enable GPIO clocks
    enable_and_reset::<peripherals::HSGPIO0>();
    enable_and_reset::<peripherals::HSGPIO1>();
    enable_and_reset::<peripherals::HSGPIO2>();
    enable_and_reset::<peripherals::HSGPIO3>();
    enable_and_reset::<peripherals::HSGPIO4>();
    enable_and_reset::<peripherals::HSGPIO5>();
    enable_and_reset::<peripherals::HSGPIO6>();
    enable_and_reset::<peripherals::HSGPIO7>();

    // Enable INTA
    interrupt::GPIO_INTA.unpend();

    // SAFETY:
    //
    // At this point, all GPIO interrupts are masked. No interrupts
    // will trigger until a pin is configured as Input, which can only
    // happen after initialization of the HAL
    unsafe { interrupt::GPIO_INTA.enable() };
}

/// Input Sense mode.
pub trait Sense: Sealed {}

/// Sense Enabled Flex pin.
///
/// This is a true flex pin as the input buffer is enabled.
/// It can sense its own level when even when configured as an output pin.
pub enum SenseEnabled {}
impl Sealed for SenseEnabled {}
impl Sense for SenseEnabled {}

/// Sense Enabled Flex pin.
///
/// This is **not** a true flex pin as the input buffer is disabled.
/// It cannot be turned into an input and it cannot see its own state, but it consumes less power.
/// Consider using a sense enabled flex pin if you need to read the pin's state or turn this into an input,
/// however note that **power consumption may be increased**.
pub enum SenseDisabled {}
impl Sealed for SenseDisabled {}
impl Sense for SenseDisabled {}

/// Flex pin.
///
/// This pin can be either an input or output pin. The output level register bit will
/// remain set while not in output mode, so the pin's level will be 'remembered' when it is not in
/// output mode.
pub struct Flex<'d, S: Sense> {
    pin: Peri<'d, AnyPin>,
    _sense_mode: PhantomData<S>,
}

impl<S: Sense> Flex<'_, S> {
    /// Converts pin to output pin
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    pub fn set_as_output(&mut self, mode: DriveMode, strength: DriveStrength, slew_rate: SlewRate) {
        self.pin
            .set_pull(Pull::None)
            .set_drive_mode(mode)
            .set_drive_strength(strength)
            .set_slew_rate(slew_rate);

        self.pin.block().dirset(self.pin.port()).write(|w|
            // SAFETY: Writing a 0 to bits in this register has no effect,
            // however PAC has it marked unsafe due to using the bits() method.
            // There is not currently a "safe" method for setting a single-bit.
            unsafe { w.dirsetp().bits(1 << self.pin.pin()) });
    }

    /// Set high
    pub fn set_high(&mut self) {
        self.pin.block().set(self.pin.port()).write(|w|
            // SAFETY: Writing a 0 to bits in this register has no effect,
            // however PAC has it marked unsafe due to using the bits() method.
            // There is not currently a "safe" method for setting a single-bit.
            unsafe { w.setp().bits(1 << self.pin.pin()) });
    }

    /// Set low
    pub fn set_low(&mut self) {
        self.pin.block().clr(self.pin.port()).write(|w|
            // SAFETY: Writing a 0 to bits in this register has no effect,
            // however PAC has it marked unsafe due to using the bits() method.
            // There is not currently a "safe" method for setting a single-bit.
            unsafe { w.clrp().bits(1 << self.pin.pin()) });
    }

    /// Set level
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }
    }

    /// Is the output level high?
    #[must_use]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[must_use]
    pub fn is_set_low(&self) -> bool {
        (self.pin.block().set(self.pin.port()).read().setp().bits() & (1 << self.pin.pin())) == 0
    }

    /// Toggle
    pub fn toggle(&mut self) {
        self.pin.block().not(self.pin.port()).write(|w|
            // SAFETY: Writing a 0 to bits in this register has no effect,
            // however PAC has it marked unsafe due to using the bits() method.
            // There is not currently a "safe" method for setting a single-bit.
            unsafe { w.notp().bits(1 << self.pin.pin()) });
    }
}

impl<S: Sense> Drop for Flex<'_, S> {
    #[inline]
    fn drop(&mut self) {
        critical_section::with(|_| {
            self.pin.reset();
        });
    }
}

impl<'d> Flex<'d, SenseEnabled> {
    /// New flex pin.
    pub fn new_with_sense(pin: Peri<'d, impl GpioPin>) -> Self {
        pin.set_function(Function::F0)
            .disable_analog_multiplex()
            .enable_input_buffer();

        Self {
            pin: pin.into(),
            _sense_mode: PhantomData::<SenseEnabled>,
        }
    }

    /// Converts pin to input pin
    pub fn set_as_input(&mut self, pull: Pull, inverter: Inverter) {
        self.pin.set_pull(pull).set_input_inverter(inverter);

        self.pin.block().dirclr(self.pin.port()).write(|w|
                    // SAFETY: Writing a 0 to bits in this register has no effect,
                    // however PAC has it marked unsafe due to using the bits() method.
                    // There is not currently a "safe" method for setting a single-bit.
                    unsafe { w.dirclrp().bits(1 << self.pin.pin()) });
    }

    /// Converts pin to special function pin
    /// # Safety
    /// Unsafe to require justifying change from default to a special function
    ///
    pub unsafe fn set_as_special_function(&mut self, func: Function) {
        self.pin.set_function(func);
    }

    /// Is high?
    #[must_use]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Is low?
    #[must_use]
    pub fn is_low(&self) -> bool {
        self.pin.block().b(self.pin.port()).b_(self.pin.pin()).read() == 0
    }

    /// Current level
    #[must_use]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptType::Level, Level::High).await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptType::Level, Level::Low).await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptType::Edge, Level::High).await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptType::Edge, Level::Low).await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        if self.is_high() {
            InputFuture::new(self.pin.reborrow(), InterruptType::Edge, Level::Low).await;
        } else {
            InputFuture::new(self.pin.reborrow(), InterruptType::Edge, Level::High).await;
        }
    }

    /// Return a new Flex pin instance with level sensing disabled.
    ///
    /// Consumes less power than a flex pin with sensing enabled.
    #[must_use]
    pub fn disable_sensing(self) -> Flex<'d, SenseDisabled> {
        // Cloning the pin is ok since we consume self immediately
        let new_pin = unsafe { self.pin.clone_unchecked() };
        drop(self);
        Flex::<SenseDisabled>::new(new_pin)
    }
}

impl<'d> Flex<'d, SenseDisabled> {
    /// New flex pin.
    pub fn new(pin: Peri<'d, impl GpioPin>) -> Self {
        pin.set_function(Function::F0)
            .disable_analog_multiplex()
            .disable_input_buffer();

        Self {
            pin: pin.into(),
            _sense_mode: PhantomData::<SenseDisabled>,
        }
    }

    /// Return a new Flex pin instance with level sensing enabled.
    #[must_use]
    pub fn enable_sensing(self) -> Flex<'d, SenseEnabled> {
        // Cloning the pin is ok since we consume self immediately
        let new_pin = unsafe { self.pin.clone_unchecked() };
        drop(self);
        Flex::new_with_sense(new_pin)
    }
}

/// Input pin
pub struct Input<'d> {
    // When Input is dropped, Flex's drop() will make sure the pin is reset to its default state.
    pin: Flex<'d, SenseEnabled>,
}

impl<'d> Input<'d> {
    /// New input pin
    pub fn new(pin: Peri<'d, impl GpioPin>, pull: Pull, inverter: Inverter) -> Self {
        let mut pin = Flex::new_with_sense(pin);
        pin.set_as_input(pull, inverter);
        Self { pin }
    }

    /// Is high?
    #[must_use]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Is low?
    #[must_use]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Input level
    #[must_use]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
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

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> InputFuture<'d> {
    fn new(pin: Peri<'d, impl GpioPin>, int_type: InterruptType, level: Level) -> Self {
        critical_section::with(|_| {
            // Clear any existing pending interrupt on this pin
            pin.block()
                .intstata(pin.port())
                .write(|w| unsafe { w.status().bits(1 << pin.pin()) });

            /* Pin interrupt configuration */
            pin.block().intedg(pin.port()).modify(|r, w| match int_type {
                InterruptType::Edge => unsafe { w.bits(r.bits() | (1 << pin.pin())) },
                InterruptType::Level => unsafe { w.bits(r.bits() & !(1 << pin.pin())) },
            });

            pin.block().intpol(pin.port()).modify(|r, w| match level {
                Level::High => unsafe { w.bits(r.bits() & !(1 << pin.pin())) },
                Level::Low => unsafe { w.bits(r.bits() | (1 << pin.pin())) },
            });

            // Enable pin interrupt on GPIO INT A
            pin.block()
                .intena(pin.port())
                .modify(|r, w| unsafe { w.int_en().bits(r.int_en().bits() | (1 << pin.pin())) });
        });

        Self { pin: pin.into() }
    }
}

impl Future for InputFuture<'_> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        if self.pin.port() >= GPIO_WAKERS.len() {
            panic!("Invalid GPIO port index {}", self.pin.port());
        }

        let port_waker = GPIO_WAKERS[self.pin.port()];
        if port_waker.is_none() {
            panic!("Waker not present for GPIO port {}", self.pin.port());
        }

        let waker = port_waker.unwrap().get_waker(self.pin.pin());
        if waker.is_none() {
            panic!(
                "Waker not present for GPIO pin {}, port {}",
                self.pin.pin(),
                self.pin.port()
            );
        }
        waker.unwrap().register(cx.waker());

        // Double check that the pin interrut has been disabled by IRQ handler
        if self.pin.block().intena(self.pin.port()).read().bits() & (1 << self.pin.pin()) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Output pin
/// Cannot be set as an input and cannot read its own pin state!
/// Consider using a Flex pin if you want that functionality, at the cost of higher power consumption.
pub struct Output<'d> {
    // When Output is dropped, Flex's drop() will make sure the pin is reset to its default state.
    pin: Flex<'d, SenseDisabled>,
}

impl<'d> Output<'d> {
    /// New output pin
    pub fn new(
        pin: Peri<'d, impl GpioPin>,
        initial_output: Level,
        mode: DriveMode,
        strength: DriveStrength,
        slew_rate: SlewRate,
    ) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_output(mode, strength, slew_rate);

        Self { pin }
    }

    /// Set high
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set low
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Toggle
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Set level
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Is set high?
    #[must_use]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Is set low?
    #[must_use]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }
}

trait SealedPin: IopctlPin {
    fn pin_port(&self) -> usize;

    fn port(&self) -> usize {
        self.pin_port() / 32
    }

    fn pin(&self) -> usize {
        self.pin_port() % 32
    }

    fn block(&self) -> crate::pac::Gpio {
        // SAFETY: Assuming GPIO pin specific registers are only accessed through this HAL,
        // this is safe because the HAL ensures ownership or exclusive mutable references
        // to pins.
        unsafe { crate::pac::Gpio::steal() }
    }
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait GpioPin: SealedPin + Sized + PeripheralType + Into<AnyPin> + 'static {
    /// Type-erase the pin.
    fn degrade(self) -> AnyPin {
        // SAFETY: This is only called within the GpioPin trait, which is only
        // implemented within this module on valid pin peripherals and thus
        // has been verified to be correct.
        unsafe { AnyPin::steal(self.port() as u8, self.pin() as u8) }
    }
}

impl SealedPin for AnyPin {
    fn pin_port(&self) -> usize {
        self.pin_port()
    }
}
impl GpioPin for AnyPin {}

macro_rules! impl_pin {
    ($pin_periph:ident, $pin_port:expr, $pin_no:expr) => {
        impl SealedPin for crate::peripherals::$pin_periph {
            fn pin_port(&self) -> usize {
                $pin_port * 32 + $pin_no
            }
        }
        impl GpioPin for crate::peripherals::$pin_periph {}
        impl From<crate::peripherals::$pin_periph> for AnyPin {
            fn from(value: crate::peripherals::$pin_periph) -> Self {
                value.degrade()
            }
        }
    };
}

/// Container for pin wakers
struct PortWaker {
    offset: usize,
    wakers: &'static [AtomicWaker],
}

impl PortWaker {
    fn get_waker(&self, pin: usize) -> Option<&AtomicWaker> {
        self.wakers.get(pin - self.offset)
    }
}

macro_rules! define_port_waker {
    ($name:ident, $start:expr, $end:expr) => {
        mod $name {
            static PIN_WAKERS: [super::AtomicWaker; $end - $start + 1] =
                [const { super::AtomicWaker::new() }; $end - $start + 1];
            pub static WAKER: super::PortWaker = super::PortWaker {
                offset: $start,
                wakers: &PIN_WAKERS,
            };
        }
    };
}

// GPIO port 0
define_port_waker!(port0_waker, 0, 31);
impl_pin!(PIO0_0, 0, 0);
impl_pin!(PIO0_1, 0, 1);
impl_pin!(PIO0_2, 0, 2);
impl_pin!(PIO0_3, 0, 3);
impl_pin!(PIO0_4, 0, 4);
impl_pin!(PIO0_5, 0, 5);
impl_pin!(PIO0_6, 0, 6);
impl_pin!(PIO0_7, 0, 7);
impl_pin!(PIO0_8, 0, 8);
impl_pin!(PIO0_9, 0, 9);
impl_pin!(PIO0_10, 0, 10);
impl_pin!(PIO0_11, 0, 11);
impl_pin!(PIO0_12, 0, 12);
impl_pin!(PIO0_13, 0, 13);
impl_pin!(PIO0_14, 0, 14);
impl_pin!(PIO0_15, 0, 15);
impl_pin!(PIO0_16, 0, 16);
impl_pin!(PIO0_17, 0, 17);
impl_pin!(PIO0_18, 0, 18);
impl_pin!(PIO0_19, 0, 19);
impl_pin!(PIO0_20, 0, 20);
impl_pin!(PIO0_21, 0, 21);
impl_pin!(PIO0_22, 0, 22);
impl_pin!(PIO0_23, 0, 23);
impl_pin!(PIO0_24, 0, 24);
impl_pin!(PIO0_25, 0, 25);
impl_pin!(PIO0_26, 0, 26);
impl_pin!(PIO0_27, 0, 27);
impl_pin!(PIO0_28, 0, 28);
impl_pin!(PIO0_29, 0, 29);
impl_pin!(PIO0_30, 0, 30);
impl_pin!(PIO0_31, 0, 31);

// GPIO port 1
define_port_waker!(port1_waker, 0, 31);
impl_pin!(PIO1_0, 1, 0);
impl_pin!(PIO1_1, 1, 1);
impl_pin!(PIO1_2, 1, 2);
impl_pin!(PIO1_3, 1, 3);
impl_pin!(PIO1_4, 1, 4);
impl_pin!(PIO1_5, 1, 5);
impl_pin!(PIO1_6, 1, 6);
impl_pin!(PIO1_7, 1, 7);
impl_pin!(PIO1_8, 1, 8);
impl_pin!(PIO1_9, 1, 9);
impl_pin!(PIO1_10, 1, 10);
impl_pin!(PIO1_11, 1, 11);
impl_pin!(PIO1_12, 1, 12);
impl_pin!(PIO1_13, 1, 13);
impl_pin!(PIO1_14, 1, 14);
impl_pin!(PIO1_15, 1, 15);
impl_pin!(PIO1_16, 1, 16);
impl_pin!(PIO1_17, 1, 17);
impl_pin!(PIO1_18, 1, 18);
impl_pin!(PIO1_19, 1, 19);
impl_pin!(PIO1_20, 1, 20);
impl_pin!(PIO1_21, 1, 21);
impl_pin!(PIO1_22, 1, 22);
impl_pin!(PIO1_23, 1, 23);
impl_pin!(PIO1_24, 1, 24);
impl_pin!(PIO1_25, 1, 25);
impl_pin!(PIO1_26, 1, 26);
impl_pin!(PIO1_27, 1, 27);
impl_pin!(PIO1_28, 1, 28);
impl_pin!(PIO1_29, 1, 29);
impl_pin!(PIO1_30, 1, 30);
impl_pin!(PIO1_31, 1, 31);

// GPIO port 2
define_port_waker!(port2_waker, 0, 31);
impl_pin!(PIO2_0, 2, 0);
impl_pin!(PIO2_1, 2, 1);
impl_pin!(PIO2_2, 2, 2);
impl_pin!(PIO2_3, 2, 3);
impl_pin!(PIO2_4, 2, 4);
impl_pin!(PIO2_5, 2, 5);
impl_pin!(PIO2_6, 2, 6);
impl_pin!(PIO2_7, 2, 7);
impl_pin!(PIO2_8, 2, 8);
impl_pin!(PIO2_9, 2, 9);
impl_pin!(PIO2_10, 2, 10);
impl_pin!(PIO2_11, 2, 11);
impl_pin!(PIO2_12, 2, 12);
impl_pin!(PIO2_13, 2, 13);
impl_pin!(PIO2_14, 2, 14);
impl_pin!(PIO2_15, 2, 15);
impl_pin!(PIO2_16, 2, 16);
impl_pin!(PIO2_17, 2, 17);
impl_pin!(PIO2_18, 2, 18);
impl_pin!(PIO2_19, 2, 19);
impl_pin!(PIO2_20, 2, 20);
impl_pin!(PIO2_21, 2, 21);
impl_pin!(PIO2_22, 2, 22);
impl_pin!(PIO2_23, 2, 23);
impl_pin!(PIO2_24, 2, 24);
impl_pin!(PIO2_25, 2, 25);
impl_pin!(PIO2_26, 2, 26);
impl_pin!(PIO2_27, 2, 27);
impl_pin!(PIO2_28, 2, 28);
impl_pin!(PIO2_29, 2, 29);
impl_pin!(PIO2_30, 2, 30);
impl_pin!(PIO2_31, 2, 31);

// GPIO port 3
define_port_waker!(port3_waker, 0, 31);
impl_pin!(PIO3_0, 3, 0);
impl_pin!(PIO3_1, 3, 1);
impl_pin!(PIO3_2, 3, 2);
impl_pin!(PIO3_3, 3, 3);
impl_pin!(PIO3_4, 3, 4);
impl_pin!(PIO3_5, 3, 5);
impl_pin!(PIO3_6, 3, 6);
impl_pin!(PIO3_7, 3, 7);
impl_pin!(PIO3_8, 3, 8);
impl_pin!(PIO3_9, 3, 9);
impl_pin!(PIO3_10, 3, 10);
impl_pin!(PIO3_11, 3, 11);
impl_pin!(PIO3_12, 3, 12);
impl_pin!(PIO3_13, 3, 13);
impl_pin!(PIO3_14, 3, 14);
impl_pin!(PIO3_15, 3, 15);
impl_pin!(PIO3_16, 3, 16);
impl_pin!(PIO3_17, 3, 17);
impl_pin!(PIO3_18, 3, 18);
impl_pin!(PIO3_19, 3, 19);
impl_pin!(PIO3_20, 3, 20);
impl_pin!(PIO3_21, 3, 21);
impl_pin!(PIO3_22, 3, 22);
impl_pin!(PIO3_23, 3, 23);
impl_pin!(PIO3_24, 3, 24);
impl_pin!(PIO3_25, 3, 25);
impl_pin!(PIO3_26, 3, 26);
impl_pin!(PIO3_27, 3, 27);
impl_pin!(PIO3_28, 3, 28);
impl_pin!(PIO3_29, 3, 29);
impl_pin!(PIO3_30, 3, 30);
impl_pin!(PIO3_31, 3, 31);

// GPIO port 4
define_port_waker!(port4_waker, 0, 10);
impl_pin!(PIO4_0, 4, 0);
impl_pin!(PIO4_1, 4, 1);
impl_pin!(PIO4_2, 4, 2);
impl_pin!(PIO4_3, 4, 3);
impl_pin!(PIO4_4, 4, 4);
impl_pin!(PIO4_5, 4, 5);
impl_pin!(PIO4_6, 4, 6);
impl_pin!(PIO4_7, 4, 7);
impl_pin!(PIO4_8, 4, 8);
impl_pin!(PIO4_9, 4, 9);
impl_pin!(PIO4_10, 4, 10);

// GPIO port 7
define_port_waker!(port7_waker, 24, 31);
impl_pin!(PIO7_24, 7, 24);
impl_pin!(PIO7_25, 7, 25);
impl_pin!(PIO7_26, 7, 26);
impl_pin!(PIO7_27, 7, 27);
impl_pin!(PIO7_28, 7, 28);
impl_pin!(PIO7_29, 7, 29);
impl_pin!(PIO7_30, 7, 30);
impl_pin!(PIO7_31, 7, 31);

static GPIO_WAKERS: [Option<&PortWaker>; PORT_COUNT] = [
    Some(&port0_waker::WAKER),
    Some(&port1_waker::WAKER),
    Some(&port2_waker::WAKER),
    Some(&port3_waker::WAKER),
    Some(&port4_waker::WAKER),
    None,
    None,
    Some(&port7_waker::WAKER),
];

impl embedded_hal_02::digital::v2::InputPin for Flex<'_, SenseEnabled> {
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

impl<S: Sense> embedded_hal_02::digital::v2::OutputPin for Flex<'_, S> {
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

impl embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'_, SenseEnabled> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<S: Sense> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'_, S> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl embedded_hal_02::digital::v2::InputPin for Input<'_> {
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

impl embedded_hal_02::digital::v2::OutputPin for Output<'_> {
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

impl embedded_hal_02::digital::v2::StatefulOutputPin for Output<'_> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'_> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<S: Sense> embedded_hal_1::digital::ErrorType for Flex<'_, S> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::InputPin for Flex<'_, SenseEnabled> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        // Dereference of self is used here and a few other places to
        // access the correct method (since different types/traits
        // share method names)
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<S: Sense> embedded_hal_1::digital::OutputPin for Flex<'_, S> {
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

impl embedded_hal_1::digital::StatefulOutputPin for Flex<'_, SenseEnabled> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Flex<'d, SenseEnabled> {
    #[inline]
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

impl embedded_hal_1::digital::ErrorType for Input<'_> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::InputPin for Input<'_> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Input<'d> {
    #[inline]
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    #[inline]
    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

impl embedded_hal_1::digital::ErrorType for Output<'_> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::OutputPin for Output<'_> {
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

impl embedded_hal_1::digital::StatefulOutputPin for Output<'_> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}
