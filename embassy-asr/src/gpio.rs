//! General-purpose I/O for the ASR6601.
//!
//! The register programming in this driver follows the vendor
//! `tremo_gpio.c` implementation. In particular, an `OER` bit disables the
//! output driver when set, `FR` is treated as write-one-to-clear, and
//! open-drain operation on PD8..PD15 is emulated by switching `OER`.

use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::interrupt::Priority;
use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::{pac, peripherals};

const PORT_COUNT: usize = 4;
const PINS_PER_PORT: usize = 16;
const PIN_COUNT: usize = PORT_COUNT * PINS_PER_PORT;
/// Base address of the GPIO bank array, taken from the PAC so the memory map
/// is not re-encoded here.
///
/// All four ports share the `gpioa::RegisterBlock` layout for every register
/// this driver touches: the PAC aliases port B/C onto it
/// (`pub use gpioa as gpiob/gpioc`), and port D differs only in `AFRH`
/// (3-bit fields for pins 8–15, handled by the AFR width special-case).
/// The PAC `int_cr` per-pin fields were verified against Reference Manual
/// §11 (interleaved `POS@2n`/`NEG@2n+1`); the bank-wide fields below stay
/// mask-addressed, which is inherent to banked GPIO on any HAL.
#[inline]
fn gpio_base() -> usize {
    pac::Gpioa::PTR as usize
}
/// Port stride in the GPIO bank array.
const GPIO_PORT_STRIDE: usize = 0x400;

const INTERRUPT_NONE: u32 = 0;
const INTERRUPT_RISING: u32 = 1;
const INTERRUPT_FALLING: u32 = 2;
const INTERRUPT_ANY_EDGE: u32 = 3;

static PIN_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];

/// A GPIO port.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Port {
    /// GPIO port A.
    A = 0,
    /// GPIO port B.
    B = 1,
    /// GPIO port C.
    C = 2,
    /// GPIO port D.
    D = 3,
}

impl Port {
    #[inline]
    const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            _ => unreachable!(),
        }
    }
}

/// A digital pin level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logical low.
    Low,
    /// Logical high.
    High,
}

impl From<bool> for Level {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

impl From<Level> for bool {
    #[inline]
    fn from(value: Level) -> Self {
        value == Level::High
    }
}

/// Internal pull resistor configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// Disable the internal pull resistor.
    None,
    /// Enable the internal pull-up resistor.
    Up,
    /// Enable the internal pull-down resistor.
    Down,
}

/// Output drive capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Drive {
    /// Maximum output current of 4 mA.
    _4mA,
    /// Maximum output current of 8 mA.
    _8mA,
}

/// GPIO output slew-rate selection.
///
/// The vendor GPIO register block and `tremo_gpio` API expose no selectable
/// slew-rate bit. This one-value type makes that hardware limitation explicit
/// while allowing portable board code to state that the hardware default is
/// required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SlewRate {
    /// Leave the pad at its hardware-defined slew rate.
    HardwareDefault,
}

/// GPIO output driver type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputType {
    /// Drive both high and low levels.
    PushPull,
    /// Drive low and release the pin for a high level.
    OpenDrain,
}

/// Explicit GPIO alternate-function selector.
///
/// Function 0 is the GPIO function. Functions 1 through 7 are deliberately
/// not assigned peripheral signal names because the available vendor sources
/// do not contain a complete pinmux matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AlternateFunction {
    /// GPIO function.
    Function0 = 0,
    /// Alternate function 1.
    Function1 = 1,
    /// Alternate function 2.
    Function2 = 2,
    /// Alternate function 3.
    Function3 = 3,
    /// Alternate function 4.
    Function4 = 4,
    /// Alternate function 5.
    Function5 = 5,
    /// Alternate function 6.
    Function6 = 6,
    /// Alternate function 7.
    Function7 = 7,
}

impl AlternateFunction {
    #[inline]
    const fn from_bits(bits: u32) -> Self {
        match bits & 0x7 {
            0 => Self::Function0,
            1 => Self::Function1,
            2 => Self::Function2,
            3 => Self::Function3,
            4 => Self::Function4,
            5 => Self::Function5,
            6 => Self::Function6,
            7 => Self::Function7,
            _ => unreachable!(),
        }
    }
}

/// Error returned when a pin cannot be a stop-3 wake source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Stop3WakeError {
    /// PD8..PD15 are not stop-3 wake sources.
    UnsupportedPin,
}

impl core::fmt::Display for Stop3WakeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedPin => f.write_str("pin is not a stop-3 wake source"),
        }
    }
}

impl core::error::Error for Stop3WakeError {}

trait SealedPin {
    fn id(&self) -> u8;
}

/// A GPIO pin singleton that can be used by this driver.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Return the GPIO port.
    #[inline]
    fn port(&self) -> Port {
        Port::from_index(self.id() / PINS_PER_PORT as u8)
    }

    /// Return the pin number within its port.
    #[inline]
    fn pin(&self) -> u8 {
        self.id() % PINS_PER_PORT as u8
    }
}

/// A type-erased GPIO pin.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AnyPin {
    id: u8,
}

impl AnyPin {
    /// Unsafely construct a type-erased pin.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no other token or driver accesses the same
    /// port and pin for the lifetime of the returned value.
    pub unsafe fn steal(port: Port, pin: u8) -> Peri<'static, Self> {
        assert!(pin < PINS_PER_PORT as u8);
        unsafe {
            Peri::new_unchecked(Self {
                id: port as u8 * PINS_PER_PORT as u8 + pin,
            })
        }
    }
}

impl_peripheral!(AnyPin);

impl SealedPin for AnyPin {
    #[inline]
    fn id(&self) -> u8 {
        self.id
    }
}

impl Pin for AnyPin {}

macro_rules! impl_pin {
    ($name:ident, $port:expr, $pin:expr) => {
        impl SealedPin for peripherals::$name {
            #[inline]
            fn id(&self) -> u8 {
                $port as u8 * PINS_PER_PORT as u8 + $pin
            }
        }

        impl Pin for peripherals::$name {}

        impl From<peripherals::$name> for AnyPin {
            #[inline]
            fn from(_pin: peripherals::$name) -> Self {
                Self {
                    id: $port as u8 * PINS_PER_PORT as u8 + $pin,
                }
            }
        }
    };
}

impl_pin!(PA0, Port::A, 0);
impl_pin!(PA1, Port::A, 1);
impl_pin!(PA2, Port::A, 2);
impl_pin!(PA3, Port::A, 3);
impl_pin!(PA4, Port::A, 4);
impl_pin!(PA5, Port::A, 5);
impl_pin!(PA6, Port::A, 6);
impl_pin!(PA7, Port::A, 7);
impl_pin!(PA8, Port::A, 8);
impl_pin!(PA9, Port::A, 9);
impl_pin!(PA10, Port::A, 10);
impl_pin!(PA11, Port::A, 11);
impl_pin!(PA12, Port::A, 12);
impl_pin!(PA13, Port::A, 13);
impl_pin!(PA14, Port::A, 14);
impl_pin!(PA15, Port::A, 15);

impl_pin!(PB0, Port::B, 0);
impl_pin!(PB1, Port::B, 1);
impl_pin!(PB2, Port::B, 2);
impl_pin!(PB3, Port::B, 3);
impl_pin!(PB4, Port::B, 4);
impl_pin!(PB5, Port::B, 5);
impl_pin!(PB6, Port::B, 6);
impl_pin!(PB7, Port::B, 7);
impl_pin!(PB8, Port::B, 8);
impl_pin!(PB9, Port::B, 9);
impl_pin!(PB10, Port::B, 10);
impl_pin!(PB11, Port::B, 11);
impl_pin!(PB12, Port::B, 12);
impl_pin!(PB13, Port::B, 13);
impl_pin!(PB14, Port::B, 14);
impl_pin!(PB15, Port::B, 15);

impl_pin!(PC0, Port::C, 0);
impl_pin!(PC1, Port::C, 1);
impl_pin!(PC2, Port::C, 2);
impl_pin!(PC3, Port::C, 3);
impl_pin!(PC4, Port::C, 4);
impl_pin!(PC5, Port::C, 5);
impl_pin!(PC6, Port::C, 6);
impl_pin!(PC7, Port::C, 7);
impl_pin!(PC8, Port::C, 8);
impl_pin!(PC9, Port::C, 9);
impl_pin!(PC10, Port::C, 10);
impl_pin!(PC11, Port::C, 11);
impl_pin!(PC12, Port::C, 12);
impl_pin!(PC13, Port::C, 13);
impl_pin!(PC14, Port::C, 14);
impl_pin!(PC15, Port::C, 15);

impl_pin!(PD0, Port::D, 0);
impl_pin!(PD1, Port::D, 1);
impl_pin!(PD2, Port::D, 2);
impl_pin!(PD3, Port::D, 3);
impl_pin!(PD4, Port::D, 4);
impl_pin!(PD5, Port::D, 5);
impl_pin!(PD6, Port::D, 6);
impl_pin!(PD7, Port::D, 7);
impl_pin!(PD8, Port::D, 8);
impl_pin!(PD9, Port::D, 9);
impl_pin!(PD10, Port::D, 10);
impl_pin!(PD11, Port::D, 11);
impl_pin!(PD12, Port::D, 12);
impl_pin!(PD13, Port::D, 13);
impl_pin!(PD14, Port::D, 14);
impl_pin!(PD15, Port::D, 15);

#[inline]
fn regs(port: Port) -> &'static pac::gpioa::RegisterBlock {
    let address = gpio_base() + port as usize * GPIO_PORT_STRIDE;
    unsafe { &*(address as *const pac::gpioa::RegisterBlock) }
}

#[inline]
fn enable_port(port: Port) {
    critical_section::with(|_| {
        let rcc = unsafe { pac::Rcc::steal() };
        // The reference manual names the GPIO bank clock gates IOM0..IOM3;
        // GPIOA maps to IOM0 and GPIOD to IOM3.
        match port {
            Port::A => rcc.cgr0().modify(|_, w| w.iom0_clk_en().set_bit()),
            Port::B => rcc.cgr0().modify(|_, w| w.iom1_clk_en().set_bit()),
            Port::C => rcc.cgr0().modify(|_, w| w.iom2_clk_en().set_bit()),
            Port::D => rcc.cgr0().modify(|_, w| w.iom3_clk_en().set_bit()),
        };
    });
}

#[inline]
fn pin_mask(pin: u8) -> u32 {
    1 << pin
}

#[inline]
fn interrupt_mask(pin: u8) -> u32 {
    0x3 << (pin as u32 * 2)
}

#[inline]
fn set_interrupt(port: Port, pin: u8, mode: u32) {
    let mask = interrupt_mask(pin);
    let value = mode << (pin as u32 * 2);

    critical_section::with(|_| {
        let gpio = regs(port);
        gpio.int_cr()
            .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | value) });
        unsafe {
            gpio.fr().write_with_zero(|w| w.bits(mask));
        }
    });
}

/// Handler for the single interrupt shared by all four GPIO ports.
///
/// Bind this to `GPIO` with [`crate::bind_interrupts!`] before constructing an
/// asynchronous input or flex pin.
pub struct InterruptHandler;

impl Handler<crate::interrupt::typelevel::GPIO> for InterruptHandler {
    unsafe fn on_interrupt() {
        for port_index in 0..PORT_COUNT as u8 {
            let port = Port::from_index(port_index);
            let gpio = regs(port);
            let flags = gpio.fr().read().bits();

            if flags == 0 {
                continue;
            }

            let configured = gpio.int_cr().read().bits();
            let mut disable_mask = 0;
            let mut wake_mask = 0u16;

            for pin in 0..PINS_PER_PORT as u8 {
                let field = interrupt_mask(pin);
                if flags & field != 0 && configured & field != 0 {
                    disable_mask |= field;
                    wake_mask |= 1 << pin;
                }
            }

            if disable_mask != 0 {
                gpio.int_cr().modify(|r, w| unsafe { w.bits(r.bits() & !disable_mask) });
            }

            // `FR` is write-1-to-clear; write back the latched flags.
            unsafe {
                gpio.fr().write_with_zero(|w| w.bits(flags));
            }

            for pin in 0..PINS_PER_PORT {
                if wake_mask & (1 << pin) != 0 {
                    PIN_WAKERS[port_index as usize * PINS_PER_PORT + pin].wake();
                }
            }
        }
    }
}

#[inline]
fn enable_gpio_interrupt() {
    crate::interrupt::typelevel::GPIO::unpend();
    crate::interrupt::typelevel::GPIO::set_priority(Priority::P3);
    unsafe { crate::interrupt::typelevel::GPIO::enable() };
}

#[must_use = "futures do nothing unless polled or awaited"]
struct InputFuture<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> InputFuture<'d> {
    #[inline]
    fn new(pin: Peri<'d, AnyPin>, mode: u32) -> Self {
        set_interrupt(pin.port(), pin.pin(), mode);
        Self { pin }
    }

    #[inline]
    fn is_high(&self) -> bool {
        regs(self.pin.port()).idr().read().bits() & pin_mask(self.pin.pin()) != 0
    }
}

impl Future for InputFuture<'_> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let index = this.pin.id as usize;
        PIN_WAKERS[index].register(cx.waker());

        let mode = regs(this.pin.port()).int_cr().read().bits() & interrupt_mask(this.pin.pin());
        if mode == INTERRUPT_NONE {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for InputFuture<'_> {
    fn drop(&mut self) {
        set_interrupt(self.pin.port(), self.pin.pin(), INTERRUPT_NONE);
    }
}

/// A flexible GPIO pin.
///
/// The pin can be changed between input, output, analog, and alternate
/// function configurations. GPIO port clocks remain enabled after a driver is
/// dropped because each clock is shared by sixteen independently-owned pins.
pub struct Flex<'d, M: Mode = Blocking> {
    pin: Peri<'d, AnyPin>,
    output_type: OutputType,
    _mode: PhantomData<M>,
}

fn make_flex<'d, M: Mode>(pin: Peri<'d, impl Pin>) -> Flex<'d, M> {
    enable_port(pin.port());
    let mut result = Flex {
        pin: pin.into(),
        output_type: OutputType::PushPull,
        _mode: PhantomData,
    };
    result.set_alternate_function(AlternateFunction::Function0);
    result
}

impl<'d> Flex<'d> {
    /// Wrap a pin for blocking GPIO use.
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        make_flex(pin)
    }
}

impl<'d> Flex<'d, Async> {
    /// Wrap a pin for GPIO use with asynchronous edge waits.
    pub fn new_async(
        pin: Peri<'d, impl Pin>,
        _irq: impl Binding<crate::interrupt::typelevel::GPIO, InterruptHandler>,
    ) -> Self {
        enable_gpio_interrupt();
        make_flex(pin)
    }

    /// Wait until the sampled input is high.
    ///
    /// The hardware has edge interrupts but no level-interrupt mode, so this
    /// arms a rising edge and rechecks the level to close the check/arm race.
    pub async fn wait_for_high(&mut self) {
        loop {
            if self.is_high() {
                return;
            }
            let future = InputFuture::new(self.pin.reborrow(), INTERRUPT_RISING);
            if future.is_high() {
                return;
            }
            future.await;
        }
    }

    /// Wait until the sampled input is low.
    ///
    /// The hardware has edge interrupts but no level-interrupt mode, so this
    /// arms a falling edge and rechecks the level to close the check/arm race.
    pub async fn wait_for_low(&mut self) {
        loop {
            if self.is_low() {
                return;
            }
            let future = InputFuture::new(self.pin.reborrow(), INTERRUPT_FALLING);
            if !future.is_high() {
                return;
            }
            future.await;
        }
    }

    /// Wait for a low-to-high transition.
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), INTERRUPT_RISING).await;
    }

    /// Wait for a high-to-low transition.
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), INTERRUPT_FALLING).await;
    }

    /// Wait for either edge.
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), INTERRUPT_ANY_EDGE).await;
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    #[inline]
    fn port(&self) -> Port {
        self.pin.port()
    }

    #[inline]
    fn pin_number(&self) -> u8 {
        self.pin.pin()
    }

    #[inline]
    fn mask(&self) -> u32 {
        pin_mask(self.pin_number())
    }

    #[inline]
    fn emulated_open_drain(&self) -> bool {
        self.output_type == OutputType::OpenDrain && self.port() == Port::D && self.pin_number() > 7
    }

    /// Configure the internal pull resistor.
    pub fn set_pull(&mut self, pull: Pull) {
        let mask = self.mask();
        critical_section::with(|_| {
            let gpio = regs(self.port());
            match pull {
                Pull::None => {
                    gpio.per().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
                }
                Pull::Up => {
                    gpio.psr().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
                    gpio.per().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
                }
                Pull::Down => {
                    gpio.psr().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
                    gpio.per().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
                }
            }
        });
    }

    /// Configure the documented output drive capability.
    pub fn set_drive(&mut self, drive: Drive) {
        let mask = self.mask();
        // The SVD models DSR as write-only, but the vendor driver performs
        // read-modify-write on it. Preserve the other pins on the port.
        critical_section::with(|_| unsafe {
            let register = regs(self.port()).dsr().as_ptr();
            let mut value = core::ptr::read_volatile(register);
            match drive {
                Drive::_4mA => value &= !mask,
                Drive::_8mA => value |= mask,
            }
            core::ptr::write_volatile(register, value);
        });
    }

    /// Select the GPIO slew rate.
    ///
    /// ASR's documented GPIO block has no slew-rate control; the only
    /// representable value therefore leaves the hardware unchanged.
    #[inline]
    pub fn set_slew_rate(&mut self, _slew_rate: SlewRate) {}

    /// Configure push-pull or open-drain output behavior.
    ///
    /// PD8..PD15 do not support `OTYPER` according to the vendor driver. For
    /// those pins, open-drain high is emulated by disabling the output driver,
    /// and open-drain low by enabling it with a low output latch.
    pub fn set_output_type(&mut self, output_type: OutputType) {
        self.output_type = output_type;
        let mask = self.mask();
        let emulated = self.emulated_open_drain();

        critical_section::with(|_| {
            let gpio = regs(self.port());
            if output_type == OutputType::OpenDrain && !emulated {
                gpio.otyper().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            } else {
                gpio.otyper().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            }

            if emulated {
                gpio.oer().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
                gpio.ier().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
                gpio.psr().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
        });

        if emulated {
            unsafe {
                regs(self.port()).brr().write_with_zero(|w| w.br().bits(mask as u16));
            }
        }
    }

    /// Return the configured output type.
    pub fn output_type(&self) -> OutputType {
        self.output_type
    }

    /// Put the pin into input mode.
    ///
    /// The pull selection is left unchanged.
    pub fn set_as_input(&mut self) {
        let mask = self.mask();
        critical_section::with(|_| {
            let gpio = regs(self.port());
            gpio.oer().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            gpio.ier().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        });
    }

    /// Put the pin into output mode.
    ///
    /// The output latch and pull selection are left unchanged. On an emulated
    /// open-drain pin this enables the low driver.
    pub fn set_as_output(&mut self) {
        let mask = self.mask();
        critical_section::with(|_| {
            let gpio = regs(self.port());
            gpio.oer().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            gpio.ier().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            if self.output_type == OutputType::PushPull {
                gpio.otyper().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            } else if !self.emulated_open_drain() {
                gpio.otyper().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
        });
    }

    /// Put the pin into analog mode.
    ///
    /// This disables both digital drivers and the pull resistor, matching
    /// `GPIO_MODE_ANALOG` in the vendor driver.
    pub fn set_as_analog(&mut self) {
        let mask = self.mask();
        critical_section::with(|_| {
            let gpio = regs(self.port());
            gpio.oer().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            gpio.ier().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            gpio.per().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
        });
    }

    /// Put the pin into disconnected mode.
    ///
    /// This chip's disconnected/reset state is analog (both digital buffers
    /// off, pull off — the state [`Self::set_as_analog`] programs and the
    /// state [`Drop`] restores); same call.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.set_as_analog();
    }

    /// Is the pin disconnected (output buffer off and input buffer off)?
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        let gpio = regs(self.port());
        gpio.oer().read().bits() & self.mask() != 0 && gpio.ier().read().bits() & self.mask() == 0
    }

    /// Is the pin configured as an input (input buffer on)?
    #[inline]
    pub fn is_input(&self) -> bool {
        regs(self.port()).ier().read().bits() & self.mask() != 0
    }

    /// Is the pin configured as an output (output buffer on)?
    #[inline]
    pub fn is_output(&self) -> bool {
        regs(self.port()).oer().read().bits() & self.mask() == 0
    }

    /// Select an explicit alternate function from 0 through 7.
    ///
    /// This only changes the mux field. Configure input/output direction,
    /// pulls, drive capability, and output type separately as required by the
    /// selected peripheral.
    pub fn set_alternate_function(&mut self, function: AlternateFunction) {
        let port = self.port();
        let pin = self.pin_number();
        let value = function as u32;

        critical_section::with(|_| {
            let gpio = regs(port);
            if pin < 8 {
                let shift = pin as u32 * 4;
                let mask = 0xf << shift;
                gpio.afrl()
                    .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value << shift)) });
            } else {
                let index = pin as u32 - 8;
                let width = if port == Port::D { 3 } else { 4 };
                let shift = index * width;
                let mask = ((1u32 << width) - 1) << shift;
                gpio.afrh()
                    .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value << shift)) });
            }
        });
    }

    /// Read the currently selected alternate function.
    pub fn alternate_function(&self) -> AlternateFunction {
        let port = self.port();
        let pin = self.pin_number();
        let gpio = regs(port);
        let bits = if pin < 8 {
            gpio.afrl().read().bits() >> (pin as u32 * 4)
        } else {
            let width = if port == Port::D { 3 } else { 4 };
            gpio.afrh().read().bits() >> ((pin as u32 - 8) * width)
        };
        AlternateFunction::from_bits(bits)
    }

    /// Return whether the sampled pin level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        regs(self.port()).idr().read().bits() & self.mask() != 0
    }

    /// Return whether the sampled pin level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output latch high, or release an emulated open-drain pin.
    pub fn set_high(&mut self) {
        let mask = self.mask();
        if self.emulated_open_drain() {
            regs(self.port())
                .oer()
                .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        } else {
            unsafe {
                regs(self.port()).bsrr().write_with_zero(|w| w.bsr().bits(mask as u16));
            }
        }
    }

    /// Set the output latch low, or drive an emulated open-drain pin low.
    pub fn set_low(&mut self) {
        let mask = self.mask();
        if self.emulated_open_drain() {
            let gpio = regs(self.port());
            unsafe {
                gpio.brr().write_with_zero(|w| w.br().bits(mask as u16));
            }
            gpio.oer().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
        } else {
            unsafe {
                regs(self.port()).brr().write_with_zero(|w| w.br().bits(mask as u16));
            }
        }
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Return whether the configured output level is high.
    pub fn is_set_high(&self) -> bool {
        if self.emulated_open_drain() {
            regs(self.port()).oer().read().bits() & self.mask() != 0
        } else {
            regs(self.port()).odr().read().bits() & self.mask() != 0
        }
    }

    /// Return whether the configured output level is low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// What level output is set to.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.output_is_high().into()
    }

    /// Is the output pin set as high?
    ///
    /// Canonical guideline spelling; [`Self::is_set_high`] is the same call.
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.is_set_high()
    }

    /// Is the output pin set as low?
    ///
    /// Canonical guideline spelling; [`Self::is_set_low`] is the same call.
    #[inline]
    pub fn output_is_low(&self) -> bool {
        self.is_set_low()
    }

    /// Toggle the configured output level.
    pub fn toggle(&mut self) {
        if self.is_set_high() {
            self.set_low();
        } else {
            self.set_high();
        }
    }

    /// Enable or disable wakeup from stop modes 0, 1, and 2.
    pub fn configure_wakeup(&mut self, enable: bool, level: Level) {
        let mask = self.mask();
        critical_section::with(|_| {
            let gpio = regs(self.port());
            gpio.wu_en()
                .modify(|r, w| unsafe { w.bits(if enable { r.bits() | mask } else { r.bits() & !mask }) });
            gpio.wu_lvl().modify(|r, w| unsafe {
                w.bits(if level == Level::High {
                    r.bits() | mask
                } else {
                    r.bits() & !mask
                })
            });
        });
    }

    /// Configure this pin as a stop-3 wake source.
    ///
    /// Stop-3 wake selection is shared by groups of four pins and selecting a
    /// pin replaces the previous selection in that group. The unusual PA6/7
    /// and PA12/13 remapping follows `gpio_config_stop3_wakeup`.
    pub fn configure_stop3_wakeup(&mut self, enable: bool, level: Level) -> Result<(), Stop3WakeError> {
        let port = self.port();
        let mut pin = self.pin_number();
        if port == Port::D && pin > 7 {
            return Err(Stop3WakeError::UnsupportedPin);
        }

        if port == Port::A {
            if pin == 6 || pin == 7 {
                pin += 6;
            } else if pin == 12 || pin == 13 {
                pin -= 6;
            }
        }

        let group = pin as u32 / 4;
        let offset = pin as u32 % 4;
        let shift = group * 4;
        let mask = 0xf << shift;
        let value = offset | if level == Level::High { 0x4 } else { 0 } | if enable { 0x8 } else { 0 };

        critical_section::with(|_| {
            regs(port)
                .stop3_wu_cr()
                .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value << shift)) });
        });
        Ok(())
    }

    /// Return the owned type-erased pin without resetting the pin state.
    pub fn into_inner(self) -> Peri<'d, AnyPin> {
        let this = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(&this.pin) }
    }
}

impl<M: Mode> Drop for Flex<'_, M> {
    fn drop(&mut self) {
        set_interrupt(self.port(), self.pin_number(), INTERRUPT_NONE);
        self.set_as_analog();
        self.set_output_type(OutputType::PushPull);
        self.set_alternate_function(AlternateFunction::Function0);
    }
}

/// A GPIO input.
pub struct Input<'d, M: Mode = Blocking> {
    pin: Flex<'d, M>,
}

impl<'d> Input<'d> {
    /// Create a blocking GPIO input.
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }
}

impl<'d> Input<'d, Async> {
    /// Create a GPIO input with asynchronous edge waits.
    pub fn new_async(
        pin: Peri<'d, impl Pin>,
        irq: impl Binding<crate::interrupt::typelevel::GPIO, InterruptHandler>,
        pull: Pull,
    ) -> Self {
        let mut pin = Flex::new_async(pin, irq);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    /// Wait until the input is high.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the input is low.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for a low-to-high transition.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for a high-to-low transition.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for either edge.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

impl<'d, M: Mode> Input<'d, M> {
    /// Return whether the input is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Return whether the input is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Configure wakeup from stop modes 0, 1, and 2.
    #[inline]
    pub fn configure_wakeup(&mut self, enable: bool, level: Level) {
        self.pin.configure_wakeup(enable, level);
    }

    /// Configure wakeup from stop mode 3.
    #[inline]
    pub fn configure_stop3_wakeup(&mut self, enable: bool, level: Level) -> Result<(), Stop3WakeError> {
        self.pin.configure_stop3_wakeup(enable, level)
    }

    /// Convert the input back into a flexible pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d, M> {
        self.pin
    }
}

/// A push-pull GPIO output.
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create a push-pull output with a glitch-free initial level.
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial);
        pin.set_output_type(OutputType::PushPull);
        pin.set_as_output();
        Self { pin }
    }

    /// Set the documented output drive capability.
    #[inline]
    pub fn set_drive(&mut self, drive: Drive) {
        self.pin.set_drive(drive);
    }

    /// Select the hardware-defined slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate);
    }

    /// Drive the pin high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Drive the pin low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Return whether the output latch is high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.output_is_high()
    }

    /// Return whether the output latch is low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.output_is_low()
    }

    /// Is the output pin set as high?
    ///
    /// Canonical guideline spelling; [`Self::is_set_high`] is the same call.
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.pin.output_is_high()
    }

    /// Is the output pin set as low?
    ///
    /// Canonical guideline spelling; [`Self::is_set_low`] is the same call.
    #[inline]
    pub fn output_is_low(&self) -> bool {
        self.pin.output_is_low()
    }

    /// What level output is set to.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }

    /// Toggle the output latch.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Convert the output back into a flexible pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d> {
        self.pin
    }
}

/// An open-drain GPIO output.
pub struct OutputOpenDrain<'d, M: Mode = Blocking> {
    pin: Flex<'d, M>,
}

fn configure_open_drain<M: Mode>(mut pin: Flex<'_, M>, initial: Level) -> Flex<'_, M> {
    pin.set_output_type(OutputType::OpenDrain);
    if pin.emulated_open_drain() {
        pin.set_level(initial);
    } else {
        pin.set_level(initial);
        pin.set_as_output();
    }
    pin
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a blocking open-drain output.
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level) -> Self {
        Self {
            pin: configure_open_drain(Flex::new(pin), initial),
        }
    }
}

impl<'d> OutputOpenDrain<'d, Async> {
    /// Create an open-drain output with asynchronous input edge waits.
    pub fn new_async(
        pin: Peri<'d, impl Pin>,
        irq: impl Binding<crate::interrupt::typelevel::GPIO, InterruptHandler>,
        initial: Level,
    ) -> Self {
        Self {
            pin: configure_open_drain(Flex::new_async(pin, irq), initial),
        }
    }

    /// Wait until the sampled pin is high.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the sampled pin is low.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for a low-to-high transition.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for a high-to-low transition.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for either edge.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

impl<'d, M: Mode> OutputOpenDrain<'d, M> {
    /// Configure the internal pull resistor.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.set_pull(pull);
    }

    /// Set the documented output drive capability.
    #[inline]
    pub fn set_drive(&mut self, drive: Drive) {
        self.pin.set_drive(drive);
    }

    /// Select the hardware-defined slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate);
    }

    /// Release the pin.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Drive the pin low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the logical output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Return whether the output is released.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.output_is_high()
    }

    /// Return whether the output is driving low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.output_is_low()
    }

    /// Is the output pin set as high?
    ///
    /// Canonical guideline spelling; [`Self::is_set_high`] is the same call.
    #[inline]
    pub fn output_is_high(&self) -> bool {
        self.pin.output_is_high()
    }

    /// Is the output pin set as low?
    ///
    /// Canonical guideline spelling; [`Self::is_set_low`] is the same call.
    #[inline]
    pub fn output_is_low(&self) -> bool {
        self.pin.output_is_low()
    }

    /// What level output is set to.
    #[inline]
    pub fn output_level(&self) -> Level {
        self.pin.output_level()
    }

    /// Toggle between released and low.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Return whether the sampled pin is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Return whether the sampled pin is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn level(&self) -> Level {
        self.pin.level()
    }

    /// Convert the output back into a flexible pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d, M> {
        self.pin
    }
}

impl<M: Mode> embedded_hal::digital::ErrorType for Flex<'_, M> {
    type Error = Infallible;
}

impl<M: Mode> embedded_hal::digital::InputPin for Flex<'_, M> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Flex::is_high(self))
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Flex::is_low(self))
    }
}

impl<M: Mode> embedded_hal::digital::OutputPin for Flex<'_, M> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Flex::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Flex::set_low(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal::digital::StatefulOutputPin for Flex<'_, M> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Flex::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Flex::is_set_low(self))
    }
}

impl<M: Mode> embedded_hal::digital::ErrorType for Input<'_, M> {
    type Error = Infallible;
}

impl<M: Mode> embedded_hal::digital::InputPin for Input<'_, M> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_high(self))
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_low(self))
    }
}

impl embedded_hal::digital::ErrorType for Output<'_> {
    type Error = Infallible;
}

impl embedded_hal::digital::OutputPin for Output<'_> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Output::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Output::set_low(self);
        Ok(())
    }
}

impl embedded_hal::digital::StatefulOutputPin for Output<'_> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_low(self))
    }
}

impl<M: Mode> embedded_hal::digital::ErrorType for OutputOpenDrain<'_, M> {
    type Error = Infallible;
}

impl<M: Mode> embedded_hal::digital::InputPin for OutputOpenDrain<'_, M> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_high(self))
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_low(self))
    }
}

impl<M: Mode> embedded_hal::digital::OutputPin for OutputOpenDrain<'_, M> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::set_low(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal::digital::StatefulOutputPin for OutputOpenDrain<'_, M> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_set_low(self))
    }
}

impl embedded_hal_async::digital::Wait for Flex<'_, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Flex::wait_for_high(self).await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Flex::wait_for_low(self).await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Flex::wait_for_rising_edge(self).await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Flex::wait_for_falling_edge(self).await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Flex::wait_for_any_edge(self).await;
        Ok(())
    }
}

impl embedded_hal_async::digital::Wait for Input<'_, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Input::wait_for_high(self).await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Input::wait_for_low(self).await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Input::wait_for_rising_edge(self).await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Input::wait_for_falling_edge(self).await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Input::wait_for_any_edge(self).await;
        Ok(())
    }
}

impl embedded_hal_async::digital::Wait for OutputOpenDrain<'_, Async> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::wait_for_high(self).await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::wait_for_low(self).await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::wait_for_rising_edge(self).await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::wait_for_falling_edge(self).await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::wait_for_any_edge(self).await;
        Ok(())
    }
}

// --- embedded-hal 0.2 blocking traits ----------------------------------------

impl<M: Mode> embedded_hal_02::digital::v2::InputPin for Flex<'_, M> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(Flex::is_high(self))
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(Flex::is_low(self))
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::OutputPin for Flex<'_, M> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Flex::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Flex::set_low(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'_, M> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(Flex::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(Flex::is_set_low(self))
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'_, M> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Flex::toggle(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::InputPin for Input<'_, M> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(Input::is_high(self))
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(Input::is_low(self))
    }
}

impl embedded_hal_02::digital::v2::OutputPin for Output<'_> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Output::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Output::set_low(self);
        Ok(())
    }
}

impl embedded_hal_02::digital::v2::StatefulOutputPin for Output<'_> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_low(self))
    }
}

impl embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'_> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Output::toggle(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::InputPin for OutputOpenDrain<'_, M> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_high(self))
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_low(self))
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::OutputPin for OutputOpenDrain<'_, M> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::set_high(self);
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::set_low(self);
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::StatefulOutputPin for OutputOpenDrain<'_, M> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_set_high(self))
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(OutputOpenDrain::is_set_low(self))
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::ToggleableOutputPin for OutputOpenDrain<'_, M> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        OutputOpenDrain::toggle(self);
        Ok(())
    }
}
