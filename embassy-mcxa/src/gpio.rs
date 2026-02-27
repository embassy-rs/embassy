//! GPIO driver built around a type-erased `Flex` pin, similar to other Embassy HALs.
//! The exported `Output`/`Input` drivers own a `Flex` so they no longer depend on the
//! concrete pin type.

use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::pin;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitMap;

use crate::pac::common::{RW, Reg};
use crate::pac::gpio::vals::{Irqc, Isf, Pdd, Pid, Ptco, Ptso};
use crate::pac::interrupt;
use crate::pac::port::regs::Pcr;
use crate::pac::port::vals::{Dse, Ibe, Inv, Mux, Pe, Ps, Sre};

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

const PORT_COUNT: usize = 5;

static PORT_WAIT_MAPS: [WaitMap<usize, ()>; PORT_COUNT] = [
    WaitMap::new(),
    WaitMap::new(),
    WaitMap::new(),
    WaitMap::new(),
    WaitMap::new(),
];

fn irq_handler(port_index: usize, gpio: crate::pac::gpio::Gpio, perf_wake: fn()) {
    let isfr = gpio.isfr0().read();

    for pin in BitIter(isfr.0) {
        // Clear all pending interrupts
        gpio.isfr0().write(|w| w.0 = 1 << pin);
        gpio.icr(pin).modify(|w| w.set_irqc(Irqc::IRQC0)); // Disable interrupt

        // Wake the corresponding port waker
        if let Some(w) = PORT_WAIT_MAPS.get(port_index) {
            perf_wake();
            w.wake(&pin, ());
        }
    }
}

#[interrupt]
fn GPIO0() {
    crate::perf_counters::incr_interrupt_gpio0();
    irq_handler(0, crate::pac::GPIO0, crate::perf_counters::incr_interrupt_gpio0_wake);
}

#[interrupt]
fn GPIO1() {
    crate::perf_counters::incr_interrupt_gpio1();
    irq_handler(1, crate::pac::GPIO1, crate::perf_counters::incr_interrupt_gpio1_wake);
}

#[interrupt]
fn GPIO2() {
    crate::perf_counters::incr_interrupt_gpio2();
    irq_handler(2, crate::pac::GPIO2, crate::perf_counters::incr_interrupt_gpio2_wake);
}

#[interrupt]
fn GPIO3() {
    crate::perf_counters::incr_interrupt_gpio3();
    irq_handler(3, crate::pac::GPIO3, crate::perf_counters::incr_interrupt_gpio3_wake);
}

#[interrupt]
fn GPIO4() {
    crate::perf_counters::incr_interrupt_gpio4();
    irq_handler(4, crate::pac::GPIO4, crate::perf_counters::incr_interrupt_gpio4_wake);
}

pub(crate) unsafe fn interrupt_init() {
    unsafe {
        use embassy_hal_internal::interrupt::InterruptExt;

        crate::pac::interrupt::GPIO0.enable();
        crate::pac::interrupt::GPIO1.enable();
        crate::pac::interrupt::GPIO2.enable();
        crate::pac::interrupt::GPIO3.enable();
        crate::pac::interrupt::GPIO4.enable();

        cortex_m::interrupt::enable();
    }
}

/// Logical level for GPIO pins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    Low,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Pull {
    Disabled,
    Up,
    Down,
}

impl From<Pull> for (Pe, Ps) {
    fn from(pull: Pull) -> Self {
        match pull {
            Pull::Disabled => (Pe::PE0, Ps::PS0),
            Pull::Up => (Pe::PE1, Ps::PS1),
            Pull::Down => (Pe::PE1, Ps::PS0),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SlewRate {
    Fast,
    Slow,
}

impl From<SlewRate> for Sre {
    fn from(slew_rate: SlewRate) -> Self {
        match slew_rate {
            SlewRate::Fast => Sre::SRE0,
            SlewRate::Slow => Sre::SRE1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DriveStrength {
    Normal,
    Double,
}

impl From<DriveStrength> for Dse {
    fn from(strength: DriveStrength) -> Self {
        match strength {
            DriveStrength::Normal => Dse::DSE0,
            DriveStrength::Double => Dse::DSE1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Inverter {
    Disabled,
    Enabled,
}

impl From<Inverter> for Inv {
    fn from(strength: Inverter) -> Self {
        match strength {
            Inverter::Disabled => Inv::INV0,
            Inverter::Enabled => Inv::INV1,
        }
    }
}

pub type Gpio = crate::peripherals::GPIO0;

/// Type-erased representation of a GPIO pin.
pub struct AnyPin {
    port: u8,
    pin: u8,
    gpio: crate::pac::gpio::Gpio,
    port_reg: crate::pac::port::Port,
    pcr_reg: Reg<Pcr, RW>,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    pub(crate) fn new(
        port: u8,
        pin: u8,
        gpio: crate::pac::gpio::Gpio,
        port_reg: crate::pac::port::Port,
        pcr_reg: Reg<Pcr, RW>,
    ) -> Self {
        Self {
            port,
            pin,
            gpio,
            port_reg,
            pcr_reg,
        }
    }

    #[inline(always)]
    fn gpio(&self) -> crate::pac::gpio::Gpio {
        self.gpio
    }

    #[inline(always)]
    pub fn port_index(&self) -> u8 {
        self.port
    }

    #[inline(always)]
    pub fn pin_index(&self) -> u8 {
        self.pin
    }

    #[inline(always)]
    fn port_reg(&self) -> crate::pac::port::Port {
        self.port_reg
    }

    #[inline(always)]
    fn pcr_reg(&self) -> Reg<Pcr, RW> {
        self.pcr_reg
    }
}

embassy_hal_internal::impl_peripheral!(AnyPin);

pub(crate) trait SealedPin {
    fn port(&self) -> u8;

    fn pin(&self) -> u8;

    fn gpio(&self) -> crate::pac::gpio::Gpio;

    fn port_reg(&self) -> crate::pac::port::Port;

    fn pcr_reg(&self) -> Reg<Pcr, RW>;

    fn set_function(&self, function: Mux);

    fn set_pull(&self, pull: Pull);

    fn set_drive_strength(&self, strength: Dse);

    fn set_slew_rate(&self, slew_rate: Sre);

    fn set_enable_input_buffer(&self, buffer_enabled: bool);

    fn set_as_disabled(&self);
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait GpioPin: SealedPin + Sized + PeripheralType + Into<AnyPin> + 'static {
    /// Type-erase the pin.
    fn degrade(self) -> AnyPin {
        // SAFETY: This is only called within the GpioPin trait, which is only
        // implemented within this module on valid pin peripherals and thus
        // has been verified to be correct.
        AnyPin::new(self.port(), self.pin(), self.gpio(), self.port_reg(), self.pcr_reg())
    }
}

impl SealedPin for AnyPin {
    #[inline(always)]
    fn pin(&self) -> u8 {
        self.pin_index()
    }

    #[inline(always)]
    fn port(&self) -> u8 {
        self.port_index()
    }

    #[inline(always)]
    fn gpio(&self) -> crate::pac::gpio::Gpio {
        self.gpio()
    }

    #[inline(always)]
    fn port_reg(&self) -> crate::pac::port::Port {
        self.port_reg()
    }

    #[inline(always)]
    fn pcr_reg(&self) -> Reg<Pcr, RW> {
        self.pcr_reg()
    }

    #[inline(always)]
    fn set_function(&self, function: Mux) {
        self.pcr_reg().modify(|w| w.set_mux(function));
    }

    #[inline(always)]
    fn set_pull(&self, pull: Pull) {
        let (pull_enable, pull_select) = pull.into();
        self.pcr_reg().modify(|w| {
            w.set_pe(pull_enable);
            w.set_ps(pull_select)
        });
    }

    #[inline(always)]
    fn set_drive_strength(&self, strength: Dse) {
        self.pcr_reg().modify(|w| w.set_dse(strength));
    }

    #[inline(always)]
    fn set_slew_rate(&self, slew_rate: Sre) {
        self.pcr_reg().modify(|w| w.set_sre(slew_rate));
    }

    #[inline(always)]
    fn set_enable_input_buffer(&self, buffer_enabled: bool) {
        self.pcr_reg()
            .modify(|w| w.set_ibe(if buffer_enabled { Ibe::IBE1 } else { Ibe::IBE0 }));
    }

    #[inline(always)]
    fn set_as_disabled(&self) {
        // Set GPIO direction as input
        self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, Pdd::PDD0));
        // Set input buffer as disabled
        self.set_enable_input_buffer(false);
        // Set mode as GPIO (vs other potential functions)
        self.set_function(Mux::MUX0);
        // Set pin as disabled
        self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, Pid::PID1));
    }
}

impl GpioPin for AnyPin {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_pin {
    ($peri:ident, $port:expr, $pin:expr, $block:ident) => {
        ::paste::paste! {
            impl SealedPin for crate::peripherals::$peri {
                #[inline(always)]
                fn port(&self) -> u8 {
                    $port
                }

                #[inline(always)]
                fn pin(&self) -> u8 {
                    $pin
                }

                #[inline(always)]
                fn gpio(&self) -> crate::pac::gpio::Gpio {
                    crate::pac::$block
                }

                #[inline(always)]
                fn port_reg(&self) -> crate::pac::port::Port {
                    crate::pac::[<PORT $port>]
                }

                #[inline(always)]
                fn pcr_reg(&self) -> Reg<Pcr, RW> {
                    self.port_reg().pcr($pin)
                }

                #[inline(always)]
                fn set_function(&self, function: Mux) {
                    self.pcr_reg().modify(|w| w.set_mux(function));
                }

                #[inline(always)]
                fn set_pull(&self, pull: Pull) {
                    let (pull_enable, pull_select) = pull.into();
                    self.pcr_reg().modify(|w| {
                        w.set_pe(pull_enable);
                        w.set_ps(pull_select);
                    });
                }

                #[inline(always)]
                fn set_drive_strength(&self, strength: Dse) {
                    self.pcr_reg().modify(|w| w.set_dse(strength));
                }

                #[inline(always)]
                fn set_slew_rate(&self, slew_rate: Sre) {
                    self.pcr_reg().modify(|w| w.set_sre(slew_rate));
                }

                #[inline(always)]
                fn set_enable_input_buffer(&self, buffer_enabled: bool) {
                    self.pcr_reg().modify(|w| w.set_ibe(if buffer_enabled { Ibe::IBE1 } else { Ibe::IBE0 }));
                }

                #[inline(always)]
                fn set_as_disabled(&self) {
                    // Set GPIO direction as input
                    self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, Pdd::PDD0));
                    // Set input buffer as disabled
                    self.set_enable_input_buffer(false);
                    // Set mode as GPIO (vs other potential functions)
                    self.set_function(Mux::MUX0);
                    // Set pin as disabled
                    self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, Pid::PID1));
                }
            }

            impl GpioPin for crate::peripherals::$peri {}

            impl From<crate::peripherals::$peri> for AnyPin {
                fn from(value: crate::peripherals::$peri) -> Self {
                    value.degrade()
                }
            }

            impl crate::peripherals::$peri {
                /// Convenience helper to obtain a type-erased handle to this pin.
                pub fn degrade(&self) -> AnyPin {
                    AnyPin::new(
                        self.port(),
                        self.pin(),
                        self.gpio(),
                        self.port_reg(),
                        self.pcr_reg(),
                    )
                }
            }
        }
    };
}

/// A flexible pin that can be configured as input or output.
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
    _marker: PhantomData<&'d mut ()>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains unmodified. The initial output level is unspecified, but
    /// can be changed before the pin is put into output mode.
    pub fn new(pin: Peri<'d, impl GpioPin>) -> Self {
        pin.set_function(Mux::MUX0);
        Self {
            pin: pin.into(),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn gpio(&self) -> crate::pac::gpio::Gpio {
        self.pin.gpio()
    }

    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        self.set_enable_input_buffer(true);
        self.gpio()
            .pddr()
            .modify(|w| w.set_pdd(self.pin.pin_index() as usize, Pdd::PDD0));
    }

    /// Put the pin into output mode.
    pub fn set_as_output(&mut self) {
        self.set_pull(Pull::Disabled);
        self.gpio()
            .pddr()
            .modify(|w| w.set_pdd(self.pin.pin_index() as usize, Pdd::PDD1));
    }

    /// Set output level to High.
    #[inline]
    pub fn set_high(&mut self) {
        self.gpio()
            .psor()
            .write(|w| w.set_ptso(self.pin.pin_index() as usize, Ptso::PTSO1));
    }

    /// Set output level to Low.
    #[inline]
    pub fn set_low(&mut self) {
        self.gpio()
            .pcor()
            .write(|w| w.set_ptco(self.pin.pin_index() as usize, Ptco::PTCO1));
    }

    /// Set output level to the given `Level`.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }
    }

    /// Toggle output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.gpio()
            .ptor()
            .write(|w| w.set_ptto(self.pin.pin_index() as usize, true));
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.gpio().pdir().read().pdi(self.pin.pin_index() as usize)
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.gpio().pdor().read().pdo(self.pin.pin_index() as usize)
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Configure the pin pull up/down level.
    pub fn set_pull(&mut self, pull_select: Pull) {
        self.pin.set_pull(pull_select);
    }

    /// Configure the pin drive strength.
    pub fn set_drive_strength(&mut self, strength: DriveStrength) {
        self.pin.set_drive_strength(strength.into());
    }

    /// Configure the pin slew rate.
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate.into());
    }

    /// Enable input buffer for the pin.
    pub fn set_enable_input_buffer(&mut self, buffer_enabled: bool) {
        self.pin.set_enable_input_buffer(buffer_enabled);
    }

    /// Get pin level.
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }
}

/// Async methods
impl<'d> Flex<'d> {
    /// Helper function that waits for a given interrupt trigger
    async fn wait_for_inner(&mut self, level: crate::pac::gpio::vals::Irqc) {
        // First, ensure that we have a waker that is ready for this port+pin
        let w = PORT_WAIT_MAPS[usize::from(self.pin.port)].wait(self.pin.pin.into());
        let mut w = pin!(w);
        // Wait for the subscription to occur, which requires polling at least once
        //
        // This function returns a result, but can only be an Err if:
        //
        // * We call `.close()` on a WaitMap, which we never do
        // * We have a duplicate key, which can't happen because `wait_for_*` methods
        //   take an &mut ref of their unique port+pin combo
        //
        // So we wait for it to complete, but ignore the result.
        _ = w.as_mut().subscribe().await;

        // Now that our waker is in the map, we can enable the appropriate interrupt
        //
        // Clear any existing pending interrupt on this pin
        self.pin.gpio().isfr0().write(|w| w.0 = 1 << self.pin.pin());
        self.pin
            .gpio()
            .icr(self.pin.pin().into())
            .write(|w| w.set_isf(Isf::ISF1));

        // Pin interrupt configuration
        self.pin.gpio().icr(self.pin.pin().into()).modify(|w| w.set_irqc(level));

        // Finally, we can await the matching call to `.wake()` from the interrupt.
        //
        // Again, technically, this could return a result, but for the same reasons
        // as above, this can't be an error in our case, so just wait for it to complete
        _ = w.await;
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::IRQC12)
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::IRQC8)
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::IRQC9)
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::IRQC10)
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(Irqc::IRQC11)
    }
}

impl<'d> Drop for Flex<'d> {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_as_disabled();
    }
}

/// GPIO output driver that owns a `Flex` pin.
pub struct Output<'d> {
    flex: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create a GPIO output driver for a [GpioPin] with the provided [Level].
    pub fn new(pin: Peri<'d, impl GpioPin>, initial: Level, strength: DriveStrength, slew_rate: SlewRate) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_level(initial);
        flex.set_as_output();
        flex.set_drive_strength(strength);
        flex.set_slew_rate(slew_rate);
        Self { flex }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.flex.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.flex.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.flex.set_level(level);
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.flex.toggle();
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.flex.is_set_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.flex.is_set_low()
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d> {
        self.flex
    }
}

/// GPIO input driver that owns a `Flex` pin.
pub struct Input<'d> {
    flex: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a [GpioPin].
    ///
    pub fn new(pin: Peri<'d, impl GpioPin>, pull_select: Pull) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input();
        flex.set_pull(pull_select);
        Self { flex }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.flex.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.is_low()
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    ///
    /// Since Drive Strength and Slew Rate are not set when creating the Input
    /// pin, they need to be set when converting
    #[inline]
    pub fn into_flex(mut self, strength: DriveStrength, slew_rate: SlewRate) -> Flex<'d> {
        self.flex.set_drive_strength(strength);
        self.flex.set_slew_rate(slew_rate);
        self.flex
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.flex.get_level()
    }
}

/// Async methods
impl<'d> Input<'d> {
    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_high()
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_low()
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_rising_edge()
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_falling_edge()
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.flex.wait_for_any_edge()
    }
}

impl embedded_hal_async::digital::Wait for Input<'_> {
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

impl embedded_hal_async::digital::Wait for Flex<'_> {
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

// Both embedded_hal 0.2 and 1.0 must be supported by embassy HALs.
impl embedded_hal_02::digital::v2::InputPin for Flex<'_> {
    // GPIO operations on this block cannot fail, therefor we set the error type
    // to Infallible to guarantee that we can only produce Ok variants.
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

impl embedded_hal_02::digital::v2::OutputPin for Flex<'_> {
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

impl embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'_> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'_> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl embedded_hal_1::digital::ErrorType for Flex<'_> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::ErrorType for Input<'_> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::ErrorType for Output<'_> {
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

impl embedded_hal_1::digital::OutputPin for Flex<'_> {
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

impl embedded_hal_1::digital::StatefulOutputPin for Flex<'_> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}
