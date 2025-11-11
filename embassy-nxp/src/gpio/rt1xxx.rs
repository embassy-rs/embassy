#![macro_use]

use core::future::Future;
use core::ops::Not;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;
use nxp_pac::gpio::vals::Icr;
use nxp_pac::iomuxc::vals::Pus;

use crate::chip::{mux_address, pad_address};
use crate::pac::common::{RW, Reg};
use crate::pac::gpio::Gpio;
#[cfg(feature = "rt")]
use crate::pac::interrupt;
use crate::pac::iomuxc::regs::{Ctl, MuxCtl};
use crate::pac::{self};

/// The GPIO pin level for pins set on "Digital" mode.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Level {
    /// Logical low. Corresponds to 0V.
    Low,
    /// Logical high. Corresponds to VDD.
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

impl Not for Level {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Level::Low => Level::High,
            Level::High => Level::Low,
        }
    }
}

/// Pull setting for a GPIO input set on "Digital" mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    /// No pull.
    None,

    // TODO: What Does PUE::KEEPER mean here?

    // 22 kOhm pull-up resistor.
    Up22K,

    // 47 kOhm pull-up resistor.
    Up47K,

    // 100 kOhm pull-up resistor.
    Up100K,

    // 100 kOhm pull-down resistor.
    Down100K,
}

/// Drive strength of an output
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Drive {
    Disabled,
    _150R,
    _75R,
    _50R,
    _37R,
    _30R,
    _25R,
    _20R,
}

/// Slew rate of an output
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlewRate {
    Slow,

    Fast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    /// Bank 1
    #[cfg(gpio1)]
    Gpio1,

    /// Bank 2
    #[cfg(gpio2)]
    Gpio2,

    /// Bank 3
    #[cfg(gpio3)]
    Gpio3,

    /// Bank 4
    #[cfg(gpio4)]
    Gpio4,

    /// Bank 5
    #[cfg(gpio5)]
    Gpio5,
}

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
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self { pin: pin.into() }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        let (pke, pue, pus) = match pull {
            Pull::None => (false, true, Pus::PUS_0_100K_OHM_PULL_DOWN),
            Pull::Up22K => (true, true, Pus::PUS_3_22K_OHM_PULL_UP),
            Pull::Up47K => (true, true, Pus::PUS_1_47K_OHM_PULL_UP),
            Pull::Up100K => (true, true, Pus::PUS_2_100K_OHM_PULL_UP),
            Pull::Down100K => (true, true, Pus::PUS_0_100K_OHM_PULL_DOWN),
        };

        self.pin.pad().modify(|w| {
            w.set_pke(pke);
            w.set_pue(pue);
            w.set_pus(pus);
        });
    }

    // Set the pin's slew rate.
    #[inline]
    pub fn set_slewrate(&mut self, rate: SlewRate) {
        self.pin.pad().modify(|w| {
            w.set_sre(match rate {
                SlewRate::Slow => false,
                SlewRate::Fast => true,
            });
        });
    }

    /// Set the pin's Schmitt trigger.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        self.pin.pad().modify(|w| {
            w.set_hys(enable);
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        self.pin.mux().modify(|w| {
            w.set_mux_mode(GPIO_MUX_MODE);
        });

        // Setting direction is RMW
        critical_section::with(|_cs| {
            self.pin.block().gdir().modify(|w| {
                w.set_gdir(self.pin.pin_number() as usize, false);
            });
        })
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        self.pin.mux().modify(|w| {
            w.set_mux_mode(GPIO_MUX_MODE);
        });

        // Setting direction is RMW
        critical_section::with(|_cs| {
            self.pin.block().gdir().modify(|w| {
                w.set_gdir(self.pin.pin_number() as usize, true);
            });
        })
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
        self.pin.pad().modify(|w| {
            w.set_ode(true);
        });
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the pull-up and pull-down resistors.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.pad().modify(|w| {
            w.set_ode(false);
            w.set_pke(false);
            w.set_pue(false);
            w.set_pus(Pus::PUS_0_100K_OHM_PULL_DOWN);
        });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.block().psr().read().psr(self.pin.pin_number() as usize)
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
        self.pin.block().dr_set().write(|w| {
            w.set_dr_set(self.pin.pin_number() as usize, true);
        });
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.block().dr_clear().write(|w| {
            w.set_dr_clear(self.pin.pin_number() as usize, true);
        });
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.block().dr_toggle().write(|w| {
            w.set_dr_toggle(self.pin.pin_number() as usize, true);
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
    ///
    /// If the [`Flex`] is set as an input, then this is equivalent to [`Flex::is_high`].
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.block().dr().read().dr(self.pin.pin_number() as usize)
    }

    /// Is the output level low?
    ///
    /// If the [`Flex`] is set as an input, then this is equivalent to [`Flex::is_low`].
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptConfiguration::High).await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptConfiguration::Low).await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptConfiguration::RisingEdge).await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptConfiguration::FallingEdge).await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptConfiguration::AnyEdge).await
    }
}

impl<'d> Drop for Flex<'d> {
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

/// GPIO input driver.
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
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
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
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
}

/// GPIO output open-drain driver.
///
/// Note that pins will **return to their floating state** when `OutputOpenDrain` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `OutputOpenDrain`, or pass it to [`core::mem::forget`].
pub struct OutputOpenDrain<'d> {
    pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_input_output();
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

#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Returns the pin number within a bank
    #[inline]
    fn pin(&self) -> u8 {
        self.pin_number()
    }

    #[inline]
    fn bank(&self) -> Bank {
        self._bank()
    }
}

/// Type-erased GPIO pin.
pub struct AnyPin {
    pub(crate) pin_number: u8,
    pub(crate) bank: Bank,
}

impl AnyPin {
    /// Unsafely create a new type-erased pin.
    ///
    /// # Safety
    ///
    /// You must ensure that you’re only using one instance of this type at a time.
    pub unsafe fn steal(bank: Bank, pin_number: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_number, bank })
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_number(&self) -> u8 {
        self.pin_number
    }

    #[inline]
    fn _bank(&self) -> Bank {
        self.bank
    }
}

// Impl details

/// Mux mode for GPIO pins. This is constant across all RT1xxx parts.
const GPIO_MUX_MODE: u8 = 0b101;

// FIXME: These don't always need to be 32 entries. GPIO5 on RT1101 contains a single pin and GPIO2 only 14.
#[cfg(gpio1)]
static GPIO1_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio2)]
static GPIO2_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio3)]
static GPIO3_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio4)]
static GPIO4_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio5)]
static GPIO5_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];

/// Sealed trait for pins. This trait is sealed and cannot be implemented outside of this crate.
pub(crate) trait SealedPin: Sized {
    fn pin_number(&self) -> u8;

    fn _bank(&self) -> Bank;

    #[inline]
    fn block(&self) -> Gpio {
        match self._bank() {
            #[cfg(gpio1)]
            Bank::Gpio1 => pac::GPIO1,
            #[cfg(gpio2)]
            Bank::Gpio2 => pac::GPIO2,
            #[cfg(gpio3)]
            Bank::Gpio3 => pac::GPIO3,
            #[cfg(gpio4)]
            Bank::Gpio4 => pac::GPIO4,
            #[cfg(gpio5)]
            Bank::Gpio5 => pac::GPIO5,
        }
    }

    #[inline]
    fn mux(&self) -> Reg<MuxCtl, RW> {
        // SAFETY: The generated mux address table is valid since it is generated from the SVD files.
        let address = unsafe { mux_address(self._bank(), self.pin_number()).unwrap_unchecked() };

        // SAFETY: The register at the address is an instance of MuxCtl.
        unsafe { Reg::from_ptr(address as *mut _) }
    }

    #[inline]
    fn pad(&self) -> Reg<Ctl, RW> {
        // SAFETY: The generated pad address table is valid since it is generated from the SVD files.
        let address = unsafe { pad_address(self._bank(), self.pin_number()).unwrap_unchecked() };

        // SAFETY: The register at the address is an instance of Ctl.
        unsafe { Reg::from_ptr(address as *mut _) }
    }

    fn waker(&self) -> &AtomicWaker {
        match self._bank() {
            #[cfg(gpio1)]
            Bank::Gpio1 => &GPIO1_WAKERS[self.pin_number() as usize],
            #[cfg(gpio2)]
            Bank::Gpio2 => &GPIO2_WAKERS[self.pin_number() as usize],
            #[cfg(gpio3)]
            Bank::Gpio3 => &GPIO3_WAKERS[self.pin_number() as usize],
            #[cfg(gpio4)]
            Bank::Gpio4 => &GPIO4_WAKERS[self.pin_number() as usize],
            #[cfg(gpio5)]
            Bank::Gpio5 => &GPIO5_WAKERS[self.pin_number() as usize],
        }
    }
}

/// This enum matches the layout of Icr.
enum InterruptConfiguration {
    Low,
    High,
    RisingEdge,
    FallingEdge,
    AnyEdge,
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> InputFuture<'d> {
    fn new(pin: Peri<'d, AnyPin>, config: InterruptConfiguration) -> Self {
        let block = pin.block();

        let (icr, edge_sel) = match config {
            InterruptConfiguration::Low => (Icr::LOW_LEVEL, false),
            InterruptConfiguration::High => (Icr::HIGH_LEVEL, false),
            InterruptConfiguration::RisingEdge => (Icr::RISING_EDGE, false),
            InterruptConfiguration::FallingEdge => (Icr::FALLING_EDGE, false),
            InterruptConfiguration::AnyEdge => (Icr::FALLING_EDGE, true),
        };

        let index = if pin.pin_number() > 15 { 1 } else { 0 };

        // Interrupt configuration performs RMW
        critical_section::with(|_cs| {
            // Disable interrupt so a level/edge detection change does not cause ISR to be set.
            block.imr().modify(|w| {
                w.set_imr(pin.pin_number() as usize, false);
            });

            block.icr(index).modify(|w| {
                w.set_pin(pin.pin_number() as usize, icr);
            });

            block.edge_sel().modify(|w| {
                w.set_edge_sel(pin.pin_number() as usize, edge_sel);
            });

            // Clear the previous interrupt.
            block.isr().modify(|w| {
                // "Status flags are cleared by writing a 1 to the corresponding bit position."
                w.set_isr(pin.pin_number() as usize, true);
            });
        });

        Self { pin }
    }
}

impl<'d> Future for InputFuture<'d> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        let waker = self.pin.waker();
        waker.register(cx.waker());

        // Enabling interrupt is RMW
        critical_section::with(|_cs| {
            self.pin.block().imr().modify(|w| {
                w.set_imr(self.pin.pin_number() as usize, true);
            });
        });

        let isr = self.pin.block().isr().read();

        if isr.isr(self.pin.pin_number() as usize) {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

/// A macro to generate all GPIO pins.
///
/// This generates a lookup table for IOMUX register addresses.
macro_rules! impl_gpio {
    (
        $($name: ident($bank: ident, $pin_number: expr);)*
    ) => {
        #[inline]
        pub(crate) const fn pad_address(bank: crate::gpio::Bank, pin: u8) -> Option<u32> {
            match (bank, pin) {
                $(
                    (crate::gpio::Bank::$bank, $pin_number) => Some(crate::chip::_generated::iomuxc::pads::$name),
                )*
                _ => None
            }
        }

        #[inline]
        pub(crate) const fn mux_address(bank: crate::gpio::Bank, pin: u8) -> Option<u32> {
            match (bank, pin) {
                $(
                    (crate::gpio::Bank::$bank, $pin_number) => Some(crate::chip::_generated::iomuxc::muxes::$name),
                )*
                _ => None
            }
        }

        $(
            impl_pin!($name, $bank, $pin_number);
        )*
    };
}

macro_rules! impl_pin {
    ($name: ident, $bank: ident, $pin_num: expr) => {
        impl crate::gpio::Pin for crate::peripherals::$name {}
        impl crate::gpio::SealedPin for crate::peripherals::$name {
            #[inline]
            fn pin_number(&self) -> u8 {
                $pin_num
            }

            #[inline]
            fn _bank(&self) -> crate::gpio::Bank {
                crate::gpio::Bank::$bank
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                use crate::gpio::SealedPin;

                Self {
                    pin_number: val.pin_number(),
                    bank: val._bank(),
                }
            }
        }
    };
}

pub(crate) fn init() {
    #[cfg(feature = "rt")]
    unsafe {
        use embassy_hal_internal::interrupt::InterruptExt;

        pac::Interrupt::GPIO1_COMBINED_0_15.enable();
        pac::Interrupt::GPIO1_COMBINED_16_31.enable();
        pac::Interrupt::GPIO2_COMBINED_0_15.enable();
        pac::Interrupt::GPIO5_COMBINED_0_15.enable();
    }
}

/// IRQ handler for GPIO pins.
///
/// If `high_bits` is false, then the interrupt is for pins 0 through 15. If true, then the interrupt
/// is for pins 16 through 31
#[cfg(feature = "rt")]
fn irq_handler(block: Gpio, wakers: &[AtomicWaker; 32], high_bits: bool) {
    use crate::BitIter;

    let isr = block.isr().read().0;
    let imr = block.imr().read().0;
    let mask = if high_bits { 0xFFFF_0000 } else { 0x0000_FFFF };
    let bits = isr & imr & mask;

    for bit in BitIter(bits) {
        wakers[bit as usize].wake();

        // Disable further interrupts for this pin. The input future will check ISR (which is kept
        // until reset).
        block.imr().modify(|w| {
            w.set_imr(bit as usize, false);
        });
    }
}

#[cfg(all(any(feature = "mimxrt1011", feature = "mimxrt1062"), feature = "rt"))]
#[interrupt]
fn GPIO1_COMBINED_0_15() {
    irq_handler(pac::GPIO1, &GPIO1_WAKERS, false);
}

#[cfg(all(any(feature = "mimxrt1011", feature = "mimxrt1062"), feature = "rt"))]
#[interrupt]
fn GPIO1_COMBINED_16_31() {
    irq_handler(pac::GPIO1, &GPIO1_WAKERS, true);
}

#[cfg(all(any(feature = "mimxrt1011", feature = "mimxrt1062"), feature = "rt"))]
#[interrupt]
fn GPIO2_COMBINED_0_15() {
    irq_handler(pac::GPIO2, &GPIO2_WAKERS, false);
}

#[cfg(all(feature = "mimxrt1062", feature = "rt"))]
#[interrupt]
fn GPIO2_COMBINED_16_31() {
    irq_handler(pac::GPIO2, &GPIO2_WAKERS, true);
}

#[cfg(all(feature = "mimxrt1062", feature = "rt"))]
#[interrupt]
fn GPIO3_COMBINED_0_15() {
    irq_handler(pac::GPIO3, &GPIO3_WAKERS, false);
}

#[cfg(all(feature = "mimxrt1062", feature = "rt"))]
#[interrupt]
fn GPIO3_COMBINED_16_31() {
    irq_handler(pac::GPIO3, &GPIO3_WAKERS, true);
}

#[cfg(all(feature = "mimxrt1062", feature = "rt"))]
#[interrupt]
fn GPIO4_COMBINED_0_15() {
    irq_handler(pac::GPIO4, &GPIO4_WAKERS, false);
}

#[cfg(all(feature = "mimxrt1062", feature = "rt"))]
#[interrupt]
fn GPIO4_COMBINED_16_31() {
    irq_handler(pac::GPIO4, &GPIO4_WAKERS, true);
}

#[cfg(all(any(feature = "mimxrt1011", feature = "mimxrt1062"), feature = "rt"))]
#[interrupt]
fn GPIO5_COMBINED_0_15() {
    irq_handler(pac::GPIO5, &GPIO5_WAKERS, false);
}
