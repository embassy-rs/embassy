//! GPIO driver built around a type-erased `Flex` pin, similar to other Embassy HALs.
//! The exported `Output`/`Input` drivers own a `Flex` so they no longer depend on the
//! concrete pin type.

use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::pin;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitMap;
use paste::paste;

use crate::pac::interrupt;
use crate::pac::port0::pcr0::{Dse, Inv, Mux, Pe, Ps, Sre};

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

fn irq_handler(port_index: usize, gpio_base: *const crate::pac::gpio0::RegisterBlock) {
    let gpio = unsafe { &*gpio_base };
    let isfr = gpio.isfr0().read().bits();

    for pin in BitIter(isfr) {
        // Clear all pending interrupts
        gpio.isfr0().write(|w| unsafe { w.bits(1 << pin) });
        gpio.icr(pin).modify(|_, w| w.irqc().irqc0()); // Disable interrupt

        // Wake the corresponding port waker
        if let Some(w) = PORT_WAIT_MAPS.get(port_index) {
            w.wake(&pin, ());
        }
    }
}

#[interrupt]
fn GPIO0() {
    irq_handler(0, crate::pac::Gpio0::ptr());
}

#[interrupt]
fn GPIO1() {
    irq_handler(1, crate::pac::Gpio1::ptr());
}

#[interrupt]
fn GPIO2() {
    irq_handler(2, crate::pac::Gpio2::ptr());
}

#[interrupt]
fn GPIO3() {
    irq_handler(3, crate::pac::Gpio3::ptr());
}

#[interrupt]
fn GPIO4() {
    irq_handler(4, crate::pac::Gpio4::ptr());
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
            Pull::Disabled => (Pe::Pe0, Ps::Ps0),
            Pull::Up => (Pe::Pe1, Ps::Ps1),
            Pull::Down => (Pe::Pe1, Ps::Ps0),
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
            SlewRate::Fast => Sre::Sre0,
            SlewRate::Slow => Sre::Sre1,
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
            DriveStrength::Normal => Dse::Dse0,
            DriveStrength::Double => Dse::Dse1,
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
            Inverter::Disabled => Inv::Inv0,
            Inverter::Enabled => Inv::Inv1,
        }
    }
}

pub type Gpio = crate::peripherals::GPIO0;

/// Type-erased representation of a GPIO pin.
pub struct AnyPin {
    port: usize,
    pin: usize,
    gpio: &'static crate::pac::gpio0::RegisterBlock,
    port_reg: &'static crate::pac::port0::RegisterBlock,
    pcr_reg: &'static crate::pac::port0::Pcr0,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    fn new(
        port: usize,
        pin: usize,
        gpio: &'static crate::pac::gpio0::RegisterBlock,
        port_reg: &'static crate::pac::port0::RegisterBlock,
        pcr_reg: &'static crate::pac::port0::Pcr0,
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
    fn mask(&self) -> u32 {
        1 << self.pin
    }

    #[inline(always)]
    fn gpio(&self) -> &'static crate::pac::gpio0::RegisterBlock {
        self.gpio
    }

    #[inline(always)]
    pub fn port_index(&self) -> usize {
        self.port
    }

    #[inline(always)]
    pub fn pin_index(&self) -> usize {
        self.pin
    }

    #[inline(always)]
    fn port_reg(&self) -> &'static crate::pac::port0::RegisterBlock {
        self.port_reg
    }

    #[inline(always)]
    fn pcr_reg(&self) -> &'static crate::pac::port0::Pcr0 {
        self.pcr_reg
    }
}

embassy_hal_internal::impl_peripheral!(AnyPin);

pub(crate) trait SealedPin {
    fn pin_port(&self) -> usize;

    fn port(&self) -> usize {
        self.pin_port() / 32
    }

    fn pin(&self) -> usize {
        self.pin_port() % 32
    }

    fn gpio(&self) -> &'static crate::pac::gpio0::RegisterBlock;

    fn port_reg(&self) -> &'static crate::pac::port0::RegisterBlock;

    fn pcr_reg(&self) -> &'static crate::pac::port0::Pcr0;

    fn set_function(&self, function: Mux);

    fn set_pull(&self, pull: Pull);

    fn set_drive_strength(&self, strength: Dse);

    fn set_slew_rate(&self, slew_rate: Sre);

    fn set_enable_input_buffer(&self);
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
    fn pin_port(&self) -> usize {
        self.port * 32 + self.pin
    }

    fn gpio(&self) -> &'static crate::pac::gpio0::RegisterBlock {
        self.gpio()
    }

    fn port_reg(&self) -> &'static crate::pac::port0::RegisterBlock {
        self.port_reg()
    }

    fn pcr_reg(&self) -> &'static crate::pac::port0::Pcr0 {
        self.pcr_reg()
    }

    fn set_function(&self, function: Mux) {
        self.pcr_reg().modify(|_, w| w.mux().variant(function));
    }

    fn set_pull(&self, pull: Pull) {
        let (pull_enable, pull_select) = pull.into();
        self.pcr_reg().modify(|_, w| {
            w.pe().variant(pull_enable);
            w.ps().variant(pull_select)
        });
    }

    fn set_drive_strength(&self, strength: Dse) {
        self.pcr_reg().modify(|_, w| w.dse().variant(strength));
    }

    fn set_slew_rate(&self, slew_rate: Sre) {
        self.pcr_reg().modify(|_, w| w.sre().variant(slew_rate));
    }

    fn set_enable_input_buffer(&self) {
        self.pcr_reg().modify(|_, w| w.ibe().ibe1());
    }
}

impl GpioPin for AnyPin {}

macro_rules! impl_pin {
    ($peri:ident, $port:expr, $pin:expr, $block:ident) => {
        impl_pin!(crate::peripherals, $peri, $port, $pin, $block);
    };

    ($perip:path, $peri:ident, $port:expr, $pin:expr, $block:ident) => {
        paste! {
            impl SealedPin for $perip::$peri {
                fn pin_port(&self) -> usize {
                    $port * 32 + $pin
                }

                fn gpio(&self) -> &'static crate::pac::gpio0::RegisterBlock {
                    unsafe { &*crate::pac::$block::ptr() }
                }

                fn port_reg(&self) -> &'static crate::pac::port0::RegisterBlock {
                    unsafe { &*crate::pac::[<Port $port>]::ptr() }
                }

                fn pcr_reg(&self) -> &'static crate::pac::port0::Pcr0 {
                    self.port_reg().[<pcr $pin>]()
                }

                fn set_function(&self, function: Mux) {
                    unsafe {
                        let port_reg = &*crate::pac::[<Port $port>]::ptr();
                        port_reg.[<pcr $pin>]().modify(|_, w| {
                            w.mux().variant(function)
                        });
                    }
                }

                fn set_pull(&self, pull: Pull) {
                    let port_reg = unsafe {&*crate::pac::[<Port $port>]::ptr()};
                    let (pull_enable, pull_select) = pull.into();
                    port_reg.[<pcr $pin>]().modify(|_, w| {
                        w.pe().variant(pull_enable);
                        w.ps().variant(pull_select)
                    });
                }

                fn set_drive_strength(&self, strength: Dse) {
                    let port_reg = unsafe {&*crate::pac::[<Port $port>]::ptr()};
                    port_reg.[<pcr $pin>]().modify(|_, w| w.dse().variant(strength));
                }

                fn set_slew_rate(&self, slew_rate: Sre) {
                    let port_reg = unsafe {&*crate::pac::[<Port $port>]::ptr()};
                    port_reg.[<pcr $pin>]().modify(|_, w| w.sre().variant(slew_rate));
                }

                fn set_enable_input_buffer(&self) {
                    let port_reg = unsafe {&*crate::pac::[<Port $port>]::ptr()};
                    port_reg.[<pcr $pin>]().modify(|_, w| w.ibe().ibe1());
                }
            }

            impl GpioPin for $perip::$peri {}

            impl From<$perip::$peri> for AnyPin {
                fn from(value: $perip::$peri) -> Self {
                    value.degrade()
                }
            }

            impl $perip::$peri {
                /// Convenience helper to obtain a type-erased handle to this pin.
                pub fn degrade(&self) -> AnyPin {
                    AnyPin::new(self.port(), self.pin(), self.gpio(), self.port_reg(), self.pcr_reg())
                }
            }
        }
    };
}

impl_pin!(P0_0, 0, 0, Gpio0);
impl_pin!(P0_1, 0, 1, Gpio0);
impl_pin!(P0_2, 0, 2, Gpio0);
impl_pin!(P0_3, 0, 3, Gpio0);
impl_pin!(P0_4, 0, 4, Gpio0);
impl_pin!(P0_5, 0, 5, Gpio0);
impl_pin!(P0_6, 0, 6, Gpio0);
impl_pin!(P0_7, 0, 7, Gpio0);
impl_pin!(P0_8, 0, 8, Gpio0);
impl_pin!(P0_9, 0, 9, Gpio0);
impl_pin!(P0_10, 0, 10, Gpio0);
impl_pin!(P0_11, 0, 11, Gpio0);
impl_pin!(P0_12, 0, 12, Gpio0);
impl_pin!(P0_13, 0, 13, Gpio0);
impl_pin!(P0_14, 0, 14, Gpio0);
impl_pin!(P0_15, 0, 15, Gpio0);
impl_pin!(P0_16, 0, 16, Gpio0);
impl_pin!(P0_17, 0, 17, Gpio0);
impl_pin!(P0_18, 0, 18, Gpio0);
impl_pin!(P0_19, 0, 19, Gpio0);
impl_pin!(P0_20, 0, 20, Gpio0);
impl_pin!(P0_21, 0, 21, Gpio0);
impl_pin!(P0_22, 0, 22, Gpio0);
impl_pin!(P0_23, 0, 23, Gpio0);
impl_pin!(P0_24, 0, 24, Gpio0);
impl_pin!(P0_25, 0, 25, Gpio0);
impl_pin!(P0_26, 0, 26, Gpio0);
impl_pin!(P0_27, 0, 27, Gpio0);
impl_pin!(P0_28, 0, 28, Gpio0);
impl_pin!(P0_29, 0, 29, Gpio0);
impl_pin!(P0_30, 0, 30, Gpio0);
impl_pin!(P0_31, 0, 31, Gpio0);

impl_pin!(P1_0, 1, 0, Gpio1);
impl_pin!(P1_1, 1, 1, Gpio1);
impl_pin!(P1_2, 1, 2, Gpio1);
impl_pin!(P1_3, 1, 3, Gpio1);
impl_pin!(P1_4, 1, 4, Gpio1);
impl_pin!(P1_5, 1, 5, Gpio1);
impl_pin!(P1_6, 1, 6, Gpio1);
impl_pin!(P1_7, 1, 7, Gpio1);
impl_pin!(P1_8, 1, 8, Gpio1);
impl_pin!(P1_9, 1, 9, Gpio1);
impl_pin!(P1_10, 1, 10, Gpio1);
impl_pin!(P1_11, 1, 11, Gpio1);
impl_pin!(P1_12, 1, 12, Gpio1);
impl_pin!(P1_13, 1, 13, Gpio1);
impl_pin!(P1_14, 1, 14, Gpio1);
impl_pin!(P1_15, 1, 15, Gpio1);
impl_pin!(P1_16, 1, 16, Gpio1);
impl_pin!(P1_17, 1, 17, Gpio1);
impl_pin!(P1_18, 1, 18, Gpio1);
impl_pin!(P1_19, 1, 19, Gpio1);
impl_pin!(P1_20, 1, 20, Gpio1);
impl_pin!(P1_21, 1, 21, Gpio1);
impl_pin!(P1_22, 1, 22, Gpio1);
impl_pin!(P1_23, 1, 23, Gpio1);
impl_pin!(P1_24, 1, 24, Gpio1);
impl_pin!(P1_25, 1, 25, Gpio1);
impl_pin!(P1_26, 1, 26, Gpio1);
impl_pin!(P1_27, 1, 27, Gpio1);
impl_pin!(P1_28, 1, 28, Gpio1);
impl_pin!(P1_29, 1, 29, Gpio1);
impl_pin!(crate::internal_peripherals, P1_30, 1, 30, Gpio1);
impl_pin!(crate::internal_peripherals, P1_31, 1, 31, Gpio1);

impl_pin!(P2_0, 2, 0, Gpio2);
impl_pin!(P2_1, 2, 1, Gpio2);
impl_pin!(P2_2, 2, 2, Gpio2);
impl_pin!(P2_3, 2, 3, Gpio2);
impl_pin!(P2_4, 2, 4, Gpio2);
impl_pin!(P2_5, 2, 5, Gpio2);
impl_pin!(P2_6, 2, 6, Gpio2);
impl_pin!(P2_7, 2, 7, Gpio2);
impl_pin!(P2_8, 2, 8, Gpio2);
impl_pin!(P2_9, 2, 9, Gpio2);
impl_pin!(P2_10, 2, 10, Gpio2);
impl_pin!(P2_11, 2, 11, Gpio2);
impl_pin!(P2_12, 2, 12, Gpio2);
impl_pin!(P2_13, 2, 13, Gpio2);
impl_pin!(P2_14, 2, 14, Gpio2);
impl_pin!(P2_15, 2, 15, Gpio2);
impl_pin!(P2_16, 2, 16, Gpio2);
impl_pin!(P2_17, 2, 17, Gpio2);
impl_pin!(P2_18, 2, 18, Gpio2);
impl_pin!(P2_19, 2, 19, Gpio2);
impl_pin!(P2_20, 2, 20, Gpio2);
impl_pin!(P2_21, 2, 21, Gpio2);
impl_pin!(P2_22, 2, 22, Gpio2);
impl_pin!(P2_23, 2, 23, Gpio2);
impl_pin!(P2_24, 2, 24, Gpio2);
impl_pin!(P2_25, 2, 25, Gpio2);
impl_pin!(P2_26, 2, 26, Gpio2);
impl_pin!(P2_27, 2, 27, Gpio2);
impl_pin!(P2_28, 2, 28, Gpio2);
impl_pin!(P2_29, 2, 29, Gpio2);
impl_pin!(P2_30, 2, 30, Gpio2);
impl_pin!(P2_31, 2, 31, Gpio2);

impl_pin!(P3_0, 3, 0, Gpio3);
impl_pin!(P3_1, 3, 1, Gpio3);
impl_pin!(P3_2, 3, 2, Gpio3);
impl_pin!(P3_3, 3, 3, Gpio3);
impl_pin!(P3_4, 3, 4, Gpio3);
impl_pin!(P3_5, 3, 5, Gpio3);
impl_pin!(P3_6, 3, 6, Gpio3);
impl_pin!(P3_7, 3, 7, Gpio3);
impl_pin!(P3_8, 3, 8, Gpio3);
impl_pin!(P3_9, 3, 9, Gpio3);
impl_pin!(P3_10, 3, 10, Gpio3);
impl_pin!(P3_11, 3, 11, Gpio3);
impl_pin!(P3_12, 3, 12, Gpio3);
impl_pin!(P3_13, 3, 13, Gpio3);
impl_pin!(P3_14, 3, 14, Gpio3);
impl_pin!(P3_15, 3, 15, Gpio3);
impl_pin!(P3_16, 3, 16, Gpio3);
impl_pin!(P3_17, 3, 17, Gpio3);
impl_pin!(P3_18, 3, 18, Gpio3);
impl_pin!(P3_19, 3, 19, Gpio3);
impl_pin!(P3_20, 3, 20, Gpio3);
impl_pin!(P3_21, 3, 21, Gpio3);
impl_pin!(P3_22, 3, 22, Gpio3);
impl_pin!(P3_23, 3, 23, Gpio3);
impl_pin!(P3_24, 3, 24, Gpio3);
impl_pin!(P3_25, 3, 25, Gpio3);
impl_pin!(P3_26, 3, 26, Gpio3);
impl_pin!(P3_27, 3, 27, Gpio3);
impl_pin!(P3_28, 3, 28, Gpio3);
impl_pin!(P3_29, 3, 29, Gpio3);
impl_pin!(P3_30, 3, 30, Gpio3);
impl_pin!(P3_31, 3, 31, Gpio3);

impl_pin!(P4_0, 4, 0, Gpio4);
impl_pin!(P4_1, 4, 1, Gpio4);
impl_pin!(P4_2, 4, 2, Gpio4);
impl_pin!(P4_3, 4, 3, Gpio4);
impl_pin!(P4_4, 4, 4, Gpio4);
impl_pin!(P4_5, 4, 5, Gpio4);
impl_pin!(P4_6, 4, 6, Gpio4);
impl_pin!(P4_7, 4, 7, Gpio4);
impl_pin!(P4_8, 4, 8, Gpio4);
impl_pin!(P4_9, 4, 9, Gpio4);
impl_pin!(P4_10, 4, 10, Gpio4);
impl_pin!(P4_11, 4, 11, Gpio4);
impl_pin!(P4_12, 4, 12, Gpio4);
impl_pin!(P4_13, 4, 13, Gpio4);
impl_pin!(P4_14, 4, 14, Gpio4);
impl_pin!(P4_15, 4, 15, Gpio4);
impl_pin!(P4_16, 4, 16, Gpio4);
impl_pin!(P4_17, 4, 17, Gpio4);
impl_pin!(P4_18, 4, 18, Gpio4);
impl_pin!(P4_19, 4, 19, Gpio4);
impl_pin!(P4_20, 4, 20, Gpio4);
impl_pin!(P4_21, 4, 21, Gpio4);
impl_pin!(P4_22, 4, 22, Gpio4);
impl_pin!(P4_23, 4, 23, Gpio4);
impl_pin!(P4_24, 4, 24, Gpio4);
impl_pin!(P4_25, 4, 25, Gpio4);
impl_pin!(P4_26, 4, 26, Gpio4);
impl_pin!(P4_27, 4, 27, Gpio4);
impl_pin!(P4_28, 4, 28, Gpio4);
impl_pin!(P4_29, 4, 29, Gpio4);
impl_pin!(P4_30, 4, 30, Gpio4);
impl_pin!(P4_31, 4, 31, Gpio4);

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
        pin.set_function(Mux::Mux0);
        Self {
            pin: pin.into(),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn gpio(&self) -> &'static crate::pac::gpio0::RegisterBlock {
        self.pin.gpio()
    }

    #[inline]
    fn mask(&self) -> u32 {
        self.pin.mask()
    }

    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        let mask = self.mask();
        let gpio = self.gpio();

        self.set_enable_input_buffer();

        gpio.pddr().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
    }

    /// Put the pin into output mode.
    pub fn set_as_output(&mut self) {
        let mask = self.mask();
        let gpio = self.gpio();

        self.set_pull(Pull::Disabled);

        gpio.pddr().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
    }

    /// Set output level to High.
    #[inline]
    pub fn set_high(&mut self) {
        self.gpio().psor().write(|w| unsafe { w.bits(self.mask()) });
    }

    /// Set output level to Low.
    #[inline]
    pub fn set_low(&mut self) {
        self.gpio().pcor().write(|w| unsafe { w.bits(self.mask()) });
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
        self.gpio().ptor().write(|w| unsafe { w.bits(self.mask()) });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        (self.gpio().pdir().read().bits() & self.mask()) != 0
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.is_high()
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
    pub fn set_enable_input_buffer(&mut self) {
        self.pin.set_enable_input_buffer();
    }

    /// Get pin level.
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }
}

/// Async methods
impl<'d> Flex<'d> {
    /// Helper function that waits for a given interrupt trigger
    async fn wait_for_inner(&mut self, level: crate::pac::gpio0::icr::Irqc) {
        // First, ensure that we have a waker that is ready for this port+pin
        let w = PORT_WAIT_MAPS[self.pin.port].wait(self.pin.pin);
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
        self.pin
            .gpio()
            .isfr0()
            .write(|w| unsafe { w.bits(1 << self.pin.pin()) });
        self.pin.gpio().icr(self.pin.pin()).write(|w| w.isf().isf1());

        // Pin interrupt configuration
        self.pin
            .gpio()
            .icr(self.pin.pin())
            .modify(|_, w| w.irqc().variant(level));

        // Finally, we can await the matching call to `.wake()` from the interrupt.
        //
        // Again, technically, this could return a result, but for the same reasons
        // as above, this can't be an error in our case, so just wait for it to complete
        _ = w.await;
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(crate::pac::gpio0::icr::Irqc::Irqc12)
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(crate::pac::gpio0::icr::Irqc::Irqc8)
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub fn wait_for_rising_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(crate::pac::gpio0::icr::Irqc::Irqc9)
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub fn wait_for_falling_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(crate::pac::gpio0::icr::Irqc::Irqc10)
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub fn wait_for_any_edge(&mut self) -> impl Future<Output = ()> + use<'_, 'd> {
        self.wait_for_inner(crate::pac::gpio0::icr::Irqc::Irqc11)
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
        self.flex.is_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
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
