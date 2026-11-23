//! MCXA pin control and type erasure.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use super::{DriveStrength, Flex, Inverter, Mode, OpenDrain, Pull, SlewRate};
use crate::pac::common::{RW, Reg};
pub(super) use crate::pac::port::Mux;
use crate::pac::port::Pcr;

impl From<OpenDrain> for bool {
    fn from(open_drain: OpenDrain) -> bool {
        match open_drain {
            OpenDrain::No => false,
            OpenDrain::Yes => true,
        }
    }
}

impl From<Pull> for (bool, bool) {
    fn from(pull: Pull) -> Self {
        match pull {
            Pull::Disabled => (false, false),
            Pull::Up => (true, true),
            Pull::Down => (true, false),
        }
    }
}

impl From<SlewRate> for bool {
    fn from(slew_rate: SlewRate) -> bool {
        match slew_rate {
            SlewRate::Fast => false,
            SlewRate::Slow => true,
        }
    }
}

impl From<DriveStrength> for bool {
    fn from(strength: DriveStrength) -> bool {
        match strength {
            DriveStrength::Normal => false,
            DriveStrength::Double => true,
        }
    }
}

impl From<Inverter> for bool {
    fn from(strength: Inverter) -> bool {
        match strength {
            Inverter::Disabled => false,
            Inverter::Enabled => true,
        }
    }
}
/// Type-erased representation of a GPIO pin.
pub struct AnyPin {
    port: u8,
    pin: u8,
    gpio: crate::pac::gpio::Gpio,

    port_reg: crate::pac::port::Port,

    pcr_reg: Reg<Pcr, RW>,

    irq_bound: bool,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    pub(crate) fn new(
        port: u8,
        pin: u8,
        gpio: crate::pac::gpio::Gpio,
        port_reg: crate::pac::port::Port,
        pcr_reg: Reg<Pcr, RW>,
        irq_bound: bool,
    ) -> Self {
        Self {
            port,
            pin,
            gpio,

            port_reg,

            pcr_reg,

            irq_bound,
        }
    }

    #[inline(always)]
    pub(super) fn gpio(&self) -> crate::pac::gpio::Gpio {
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

    #[inline(always)]
    pub(super) fn irq_bound(&self) -> bool {
        self.irq_bound
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

    fn set_drive_strength(&self, strength: bool);

    fn set_slew_rate(&self, slew_rate: bool);

    fn set_enable_input_buffer(&self, buffer_enabled: bool);

    fn set_as_disabled(&self);

    fn set_input_enabled(&self, input_enabled: bool);
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait GpioPin: SealedPin + Sized + PeripheralType + Into<AnyPin> + 'static {
    /// Type-erase the pin.
    fn degrade(self) -> AnyPin {
        // SAFETY: This is only called within the GpioPin trait, which is only
        // implemented within this module on valid pin peripherals and thus
        // has been verified to be correct.
        AnyPin::new(
            self.port(),
            self.pin(),
            self.gpio(),
            self.port_reg(),
            self.pcr_reg(),
            false,
        )
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
    fn set_drive_strength(&self, strength: bool) {
        self.pcr_reg().modify(|w| w.set_dse(strength));
    }

    #[inline(always)]
    fn set_slew_rate(&self, slew_rate: bool) {
        self.pcr_reg().modify(|w| w.set_sre(slew_rate));
    }

    #[inline(always)]

    fn set_enable_input_buffer(&self, buffer_enabled: bool) {
        self.pcr_reg().modify(|w| w.set_ibe(buffer_enabled));
    }

    #[inline(always)]

    fn set_as_disabled(&self) {
        // Set GPIO direction as input
        self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, false));
        // Set input buffer as disabled
        self.set_enable_input_buffer(false);
        // Set mode as GPIO (vs other potential functions)
        self.set_function(Mux::Mux0);
        // Set pin as disabled
        self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, true));
    }

    #[inline(always)]

    fn set_input_enabled(&self, input_enabled: bool) {
        // if `input_enabled` is true then we want Pid0 since this means "Configured for general-purpose input."
        // if `input_enabled` is false then we want Pid1 since this means "Disabled for general-purpose input".
        self.gpio()
            .pidr()
            .modify(|w| w.set_pid(self.pin() as usize, !input_enabled))
    }
}

impl GpioPin for AnyPin {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_gpio_pin {
    ($peri:ident, $port:expr, $pin:expr, $block:ident) => {
        ::paste::paste! {

            impl $crate::gpio::SealedPin for $crate::peripherals::$peri {
                #[inline(always)]
                fn port(&self) -> u8 {
                    $port
                }

                #[inline(always)]
                fn pin(&self) -> u8 {
                    $pin
                }

                #[inline(always)]
                fn gpio(&self) -> $crate::pac::gpio::Gpio {
                    $crate::pac::$block
                }

                #[inline(always)]
                fn port_reg(&self) -> $crate::pac::port::Port {
                    $crate::pac::[<PORT $port>]
                }

                #[inline(always)]
                fn pcr_reg(&self) -> $crate::pac::common::Reg<$crate::pac::port::Pcr, $crate::pac::common::RW> {
                    self.port_reg().pcr($pin)
                }

                #[inline(always)]
                fn set_function(&self, function: $crate::pac::port::Mux) {
                    self.pcr_reg().modify(|w| w.set_mux(function));
                }

                #[inline(always)]
                fn set_pull(&self, pull: $crate::gpio::Pull) {
                    let (pull_enable, pull_select) = pull.into();
                    self.pcr_reg().modify(|w| {
                        w.set_pe(pull_enable);
                        w.set_ps(pull_select);
                    });
                }

                #[inline(always)]
                fn set_drive_strength(&self, strength: bool) {
                    self.pcr_reg().modify(|w| w.set_dse(strength));
                }

                #[inline(always)]
                fn set_slew_rate(&self, slew_rate: bool) {
                    self.pcr_reg().modify(|w| w.set_sre(slew_rate));
                }

                #[inline(always)]
                fn set_enable_input_buffer(&self, buffer_enabled: bool) {
                    self.pcr_reg().modify(|w| w.set_ibe(buffer_enabled));
                }

                #[inline(always)]
                fn set_as_disabled(&self) {
                    // Set GPIO direction as input
                    self.gpio().pddr().modify(|w| w.set_pdd(self.pin() as usize, false));
                    // Set input buffer as disabled
                    self.set_enable_input_buffer(false);
                    // Set mode as GPIO (vs other potential functions)
                    self.set_function($crate::pac::port::Mux::Mux0);
                    // Set pin as disabled
                    self.set_input_enabled(false);
                }

                #[inline(always)]
                fn set_input_enabled(&self, input_enabled: bool) {
                    // if `input_enabled` is true then we want Pid0 since this means "Configured for general-purpose input."
                    // if `input_enabled` is false then we want Pid1 since this means "Disabled for general-purpose input".
                    self.gpio().pidr().modify(|w| w.set_pid(self.pin() as usize, !input_enabled))
                }
            }

            impl $crate::gpio::GpioPin for $crate::peripherals::$peri {}

            impl From<$crate::peripherals::$peri> for $crate::gpio::AnyPin {
                fn from(value: $crate::peripherals::$peri) -> Self {
                    value.degrade()
                }
            }

            impl $crate::peripherals::$peri {
                /// Convenience helper to obtain a type-erased handle to this pin.
                pub fn degrade(&self) -> $crate::gpio::AnyPin {
                    use $crate::gpio::SealedPin;

                    $crate::gpio::AnyPin::new(
                        self.port(),
                        self.pin(),
                        self.gpio(),

                        self.port_reg(),

                        self.pcr_reg(),

                        false,
                    )
                }
            }

            impl $crate::gpio::HasGpioInstance for $crate::peripherals::$peri {
                type Instance = $crate::peripherals::$block;
                fn degrade_async<'p>(
                    this: embassy_hal_internal::Peri<'p, Self>,
                    _irq: impl $crate::interrupt::typelevel::Binding<
                        <Self::Instance as $crate::gpio::Instance>::Interrupt,
                        $crate::gpio::InterruptHandler<Self::Instance>,
                    >,
                ) -> embassy_hal_internal::Peri<'p, $crate::gpio::AnyPin> {
                    use $crate::interrupt::typelevel::Interrupt;
                    use $crate::gpio::SealedPin;
                    unsafe {
                        <<Self as $crate::gpio::HasGpioInstance>::Instance as $crate::gpio::Instance>::Interrupt::enable();
                    }
                    unsafe {
                        embassy_hal_internal::Peri::new_unchecked($crate::gpio::AnyPin::new(
                            this.port(),
                            this.pin(),
                            this.gpio(),
                            this.port_reg(),
                            this.pcr_reg(),
                            true,
                        ))
                    }
                }
            }
        }
    };
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
            _phantom: PhantomData,
        }
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        self.set_enable_input_buffer(true);
        // Clear PIDR to enable pin input. set_as_disabled() sets PID=1
        // which permanently disables input reads (always returns 0).
        self.gpio()
            .pidr()
            .modify(|w| w.set_pid(self.pin.pin_index() as usize, false));
        self.gpio()
            .pddr()
            .modify(|w| w.set_pdd(self.pin.pin_index() as usize, false));
    }

    /// Put the pin into output mode.
    pub fn set_as_output(&mut self) {
        self.set_pull(Pull::Disabled);

        // Clear PIDR in case this pin was previously disabled.
        self.gpio()
            .pidr()
            .modify(|w| w.set_pid(self.pin.pin_index() as usize, false));
        self.gpio()
            .pddr()
            .modify(|w| w.set_pdd(self.pin.pin_index() as usize, true));
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        #[cfg(feature = "_mcxa2xx")]
        let set = self.gpio().pdor().read().pdo(self.pin.pin_index() as usize);
        #[cfg(feature = "_mcxa5xx")]
        let set = (self.gpio().pdor().read().0 & (1 << self.pin.pin_index())) != 0;
        set
    }

    /// Configure open-drain output.
    #[inline]
    pub fn set_open_drain(&mut self, open_drain: OpenDrain) {
        self.pin.pcr_reg().modify(|w| w.set_ode(open_drain.into()));
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_input_inversion(&mut self, invert: Inverter) {
        self.pin.pcr_reg().modify(|w| w.set_inv(invert.into()));
    }
}
