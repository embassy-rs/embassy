//! MCXN native PCR and blocking GPIO capabilities.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use super::{DriveStrength, Flex, Inverter, Mode, OpenDrain, Pull, SlewRate};
use crate::_generated::{mcxn_pcr_modify, mcxn_pcr_modify_drive};

// N has native per-pad PCR types, not A's uniform PORT/PCR model. `build.rs`
// generates the `(port, pin)` dispatch to them from the PAC's GPIO pads; the GPIO
// registers and public driver are shared.

type Dse = DriveStrength;

type Sre = SlewRate;

pub(crate) enum Mux {
    Mux0,
}

macro_rules! modify_mcxn_pcr {
    ($pin:expr, |$w:ident| $body:block) => {
        mcxn_pcr_modify!($pin.port(), $pin.pin(), |$w| $body)
    };
}

/// Like `modify_mcxn_pcr!`, for `DSE`/`SRE`. Pads without those fields (PORT5)
/// are left unchanged.
macro_rules! modify_mcxn_pcr_drive {
    ($pin:expr, |$w:ident| $body:block) => {
        mcxn_pcr_modify_drive!($pin.port(), $pin.pin(), |$w| $body)
    };
}

// MCXN PCR encodings. Match the setting explicitly: declaration order of the
// shared public enums is not part of the register-programming contract.
const fn drive_strength_bits(strength: DriveStrength) -> bool {
    match strength {
        DriveStrength::Normal => false,
        DriveStrength::Double => true,
    }
}

const fn slew_rate_bits(slew: SlewRate) -> bool {
    match slew {
        SlewRate::Fast => false,
        SlewRate::Slow => true,
    }
}

const fn open_drain_bits(open_drain: OpenDrain) -> bool {
    match open_drain {
        OpenDrain::No => false,
        OpenDrain::Yes => true,
    }
}

const fn inverter_bits(invert: Inverter) -> bool {
    match invert {
        Inverter::Disabled => false,
        Inverter::Enabled => true,
    }
}

/// Type-erased representation of a GPIO pin.
pub struct AnyPin {
    port: u8,
    pin: u8,
    gpio: crate::pac::gpio::Gpio,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    pub(crate) fn new(port: u8, pin: u8, gpio: crate::pac::gpio::Gpio) -> Self {
        Self { port, pin, gpio }
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
}

embassy_hal_internal::impl_peripheral!(AnyPin);

pub(crate) trait SealedPin {
    fn port(&self) -> u8;

    fn pin(&self) -> u8;

    fn gpio(&self) -> crate::pac::gpio::Gpio;

    fn set_function(&self, _function: Mux) {
        modify_mcxn_pcr!(self, |w| { w.set_mux(0u8.into()) });
    }

    fn set_pull(&self, pull: Pull) {
        modify_mcxn_pcr!(self, |w| {
            w.set_pe(pull != Pull::Disabled);
            w.set_ps(pull == Pull::Up);
        });
    }

    fn set_drive_strength(&self, strength: Dse) {
        modify_mcxn_pcr_drive!(self, |w| { w.set_dse(drive_strength_bits(strength)) });
    }

    fn set_slew_rate(&self, slew: Sre) {
        modify_mcxn_pcr_drive!(self, |w| { w.set_sre(slew_rate_bits(slew)) });
    }

    fn set_enable_input_buffer(&self, enabled: bool) {
        modify_mcxn_pcr!(self, |w| { w.set_ibe(enabled) });
    }

    fn set_as_disabled(&self) {
        self.gpio().pddr().modify(|w| w.set_pdd(self.pin().into(), false));
        self.set_enable_input_buffer(false);
        self.set_function(Mux::Mux0);
        self.set_input_enabled(false);
    }

    fn set_input_enabled(&self, enabled: bool) {
        self.gpio().pidr().modify(|w| {
            w.set_pid(self.pin().into(), !enabled);
        });
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
        AnyPin::new(self.port(), self.pin(), self.gpio())
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
}

impl GpioPin for AnyPin {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_gpio_pin {
    ($peri:ident, $port:expr, $pin:expr, $block:ident) => {
        ::paste::paste! {

            impl $crate::gpio::SealedPin for $crate::peripherals::$peri {
                fn port(&self) -> u8 { $port }
                fn pin(&self) -> u8 { $pin }
                fn gpio(&self) -> $crate::pac::gpio::Gpio { $crate::pac::$block }
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

                    )
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
        Self {
            pin: pin.into(),
            _phantom: PhantomData,
        }
    }
}

impl<'d, M: Mode> Flex<'d, M> {
    /// Put the pin into input mode.
    pub fn set_as_input(&mut self) {
        self.pin.set_function(Mux::Mux0);
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

        self.pin.set_function(Mux::Mux0);
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
        (self.gpio().pdor().read().0 & (1 << self.pin.pin_index())) != 0
    }

    /// Configure open-drain output.
    #[inline]
    pub fn set_open_drain(&mut self, open_drain: OpenDrain) {
        modify_mcxn_pcr!(self.pin, |w| { w.set_ode(open_drain_bits(open_drain)) });
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_input_inversion(&mut self, invert: Inverter) {
        modify_mcxn_pcr!(self.pin, |w| { w.set_inv(inverter_bits(invert)) });
    }
}
