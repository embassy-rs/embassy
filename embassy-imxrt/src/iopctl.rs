//! IO Pad Controller (IOPCTL)
//!
//! Also known as IO Pin Configuration (IOCON)

use crate::pac::{iopctl, Iopctl};

// A generic pin of any type.
//
// The actual pin type used here is arbitrary,
// as all PioM_N types provide the same methods.
//
// Merely need some pin type to cast a raw pointer
// to in order to access the provided methods.
#[allow(non_camel_case_types)]
type PioM_N = iopctl::Pio0_0;

/// Pin function number.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Function {
    /// Function 0
    F0,
    /// Function 1
    F1,
    /// Function 2
    F2,
    /// Function 3
    F3,
    /// Function 4
    F4,
    /// Function 5
    F5,
    /// Function 6
    F6,
    /// Function 7
    F7,
    /// Function 8
    F8,
}

/// Internal pull-up/down resistors on a pin.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull-up or pull-down resistor selected
    None,
    /// Pull-up resistor
    Up,
    /// Pull-down resistor
    Down,
}

/// Pin slew rate.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlewRate {
    /// Standard slew rate
    Standard,
    /// Slow slew rate
    Slow,
}

/// Output drive strength of a pin.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DriveStrength {
    /// Normal
    Normal,
    /// Full
    Full,
}

/// Output drive mode of a pin.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DriveMode {
    /// Push-Pull
    PushPull,
    /// Pseudo Open-Drain
    OpenDrain,
}

/// Input inverter of a pin.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Inverter {
    /// No inverter
    Disabled,
    /// Enable input inverter on the input port. A low signal will be
    /// seen as a high signal by the pin.
    Enabled,
}

trait SealedPin {}
trait ToAnyPin: SealedPin {
    #[inline]
    fn to_raw(port: u8, pin: u8) -> AnyPin {
        // SAFETY: This is safe since this is only called from within the module,
        // where the port and pin numbers have been verified to be correct.
        unsafe { AnyPin::steal(port, pin) }
    }
}

trait ToFC15Pin: SealedPin {
    #[inline]
    fn to_raw(pin: u8) -> FC15Pin {
        // SAFETY: This is safe since this is only called from within the module,
        // where the port and pin numbers have been verified to be correct.
        unsafe { FC15Pin::steal(pin) }
    }
}

/// A pin that can be configured via iopctl.
#[allow(private_bounds)]
pub trait IopctlPin: SealedPin {
    /// Sets the function number of a pin.
    ///
    /// This number corresponds to a specific function that the pin supports.
    ///
    /// Typically, function 0 corresponds to GPIO while other numbers correspond to a special function.
    ///
    /// See Section 7.5.3 in reference manual for list of pins and their supported functions.
    fn set_function(&self, function: Function) -> &Self;

    /// Enables either a pull-up or pull-down resistor on a pin.
    ///
    /// Setting this to [`Pull::None`] will disable the resistor.
    fn set_pull(&self, pull: Pull) -> &Self;

    /// Enables the input buffer of a pin.
    ///
    /// This must be enabled for any pin acting as an input,
    /// and some peripheral pins acting as output may need this enabled as well.
    ///
    /// If there is any doubt, it is best to enable the input buffer.
    ///
    /// See Section 7.4.2.3 of reference manual.
    fn enable_input_buffer(&self) -> &Self;

    /// Disables the input buffer of a pin.
    fn disable_input_buffer(&self) -> &Self;

    /// Sets the slew rate of a pin.
    ///
    /// This controls the speed at which a pin can toggle,
    /// which is voltage and load dependent.
    fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self;

    /// Sets the output drive strength of a pin.
    ///
    /// A drive strength of [`DriveStrength::Full`] has twice the
    /// high and low drive capability of the [`DriveStrength::Normal`] setting.
    fn set_drive_strength(&self, strength: DriveStrength) -> &Self;

    /// Enables the analog multiplexer of a pin.
    ///
    /// This must be called to allow analog functionalities of a pin.
    ///
    /// To protect the analog input, [`IopctlPin::set_function`] should be
    /// called with [`Function::F0`] to disable digital functions.
    ///
    /// Additionally, [`IopctlPin::disable_input_buffer`] and [`IopctlPin::set_pull`]
    /// with [`Pull::None`] should be called.
    fn enable_analog_multiplex(&self) -> &Self;

    /// Disables the analog multiplexer of a pin.
    fn disable_analog_multiplex(&self) -> &Self;

    /// Sets the ouput drive mode of a pin.
    ///
    /// A pin configured as [`DriveMode::OpenDrain`] actually operates in
    /// a "pseudo" open-drain mode which is somewhat different than true open-drain.
    ///
    /// See Section 7.4.2.7 of reference manual.
    fn set_drive_mode(&self, mode: DriveMode) -> &Self;

    /// Sets the input inverter of an input pin.
    ///
    /// Setting this to [`Inverter::Enabled`] will invert
    /// the input signal.
    fn set_input_inverter(&self, inverter: Inverter) -> &Self;

    /// Returns a pin to its reset state.
    fn reset(&self) -> &Self;
}

/// Represents a pin peripheral created at run-time from given port and pin numbers.
pub struct AnyPin {
    pin_port: u8,
    reg: &'static PioM_N,
}

impl AnyPin {
    /// Creates a pin from raw port and pin numbers which can then be configured.
    ///
    /// This should ONLY be called when there is no other choice
    /// (e.g. from a type-erased GPIO pin).
    ///
    /// Otherwise, pin peripherals should be configured directly.
    ///
    /// # Safety
    ///
    /// The caller MUST ensure valid port and pin numbers are provided,
    /// and that multiple instances of [`AnyPin`] with the same port
    /// and pin combination are not being used simultaneously.
    ///
    /// Failure to uphold these requirements will result in undefined behavior.
    ///
    /// See Table 297 in reference manual for a list of valid
    /// pin and port number combinations.
    #[must_use]
    pub unsafe fn steal(port: u8, pin: u8) -> Self {
        // Calculates the offset from the beginning of the IOPCTL register block
        // address to the register address representing the pin.
        //
        // See Table 297 in reference manual for how this offset is calculated.
        let offset = ((port as usize) << 7) + ((pin as usize) << 2);

        // SAFETY: This is safe assuming the caller of this function satisfies the safety requirements above.
        let reg = unsafe { &*Iopctl::ptr().byte_offset(offset as isize).cast() };
        Self {
            pin_port: port * 32 + pin,
            reg,
        }
    }

    /// Returns the pin's port and pin combination.
    #[must_use]
    pub fn pin_port(&self) -> usize {
        self.pin_port as usize
    }
}

/// Represents a FC15 pin peripheral created at run-time from given pin number.
pub struct FC15Pin {
    reg: &'static PioM_N,
}

impl FC15Pin {
    /// Creates an FC15 pin from raw pin number which can then be configured.
    ///
    /// This should ONLY be called when there is no other choice
    /// (e.g. from a type-erased GPIO pin).
    ///
    /// Otherwise, pin peripherals should be configured directly.
    ///
    /// # Safety
    ///
    /// The caller MUST ensure valid port and pin numbers are provided,
    /// and that multiple instances of [`AnyPin`] with the same port
    /// and pin combination are not being used simultaneously.
    ///
    /// Failure to uphold these requirements will result in undefined behavior.
    ///
    /// See Table 297 in reference manual for a list of valid
    /// pin and port number combinations.
    #[must_use]
    pub unsafe fn steal(pin: u8) -> Self {
        // Table 297:  FC15_I2C_SCL offset = 0x400, FC15_I2C_SCL offset = 0x404
        let iopctl = unsafe { crate::pac::Iopctl::steal() };

        let reg = if pin == 0 {
            &*iopctl.fc15_i2c_scl().as_ptr().cast()
        } else {
            &*iopctl.fc15_i2c_sda().as_ptr().cast()
        };

        Self { reg }
    }
}

// This allows AnyPin/FC15Pin to be used in HAL constructors that require types
// which impl Peripheral. Used primarily by GPIO HAL to convert type-erased
// GPIO pins back into an Output or Input pin specifically.
embassy_hal_internal::impl_peripheral!(AnyPin);

impl SealedPin for AnyPin {}

embassy_hal_internal::impl_peripheral!(FC15Pin);

impl SealedPin for FC15Pin {}

macro_rules! impl_iopctlpin {
    ($pintype:ident) => {
        impl IopctlPin for $pintype {
            fn set_function(&self, function: Function) -> &Self {
                critical_section::with(|_| match function {
                    Function::F0 => {
                        self.reg.modify(|_, w| w.fsel().function_0());
                    }
                    Function::F1 => {
                        self.reg.modify(|_, w| w.fsel().function_1());
                    }
                    Function::F2 => {
                        self.reg.modify(|_, w| w.fsel().function_2());
                    }
                    Function::F3 => {
                        self.reg.modify(|_, w| w.fsel().function_3());
                    }
                    Function::F4 => {
                        self.reg.modify(|_, w| w.fsel().function_4());
                    }
                    Function::F5 => {
                        self.reg.modify(|_, w| w.fsel().function_5());
                    }
                    Function::F6 => {
                        self.reg.modify(|_, w| w.fsel().function_6());
                    }
                    Function::F7 => {
                        self.reg.modify(|_, w| w.fsel().function_7());
                    }
                    Function::F8 => {
                        self.reg.modify(|_, w| w.fsel().function_8());
                    }
                });
                self
            }

            fn set_pull(&self, pull: Pull) -> &Self {
                critical_section::with(|_| {
                    match pull {
                        Pull::None => {
                            self.reg.modify(|_, w| w.pupdena().disabled());
                        }
                        Pull::Up => {
                            self.reg.modify(|_, w| w.pupdena().enabled().pupdsel().pull_up());
                        }
                        Pull::Down => {
                            self.reg
                                .modify(|_, w| w.pupdena().enabled().pupdsel().pull_down());
                        }
                    }
                    self
                })
            }

            fn enable_input_buffer(&self) -> &Self {
                critical_section::with(|_| self.reg.modify(|_, w| w.ibena().enabled()));
                self
            }

            fn disable_input_buffer(&self) -> &Self {
                critical_section::with(|_| self.reg.modify(|_, w| w.ibena().disabled()));
                self
            }

            fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self {
                critical_section::with(|_| match slew_rate {
                    SlewRate::Standard => {
                        self.reg.modify(|_, w| w.slewrate().normal());
                    }
                    SlewRate::Slow => {
                        self.reg.modify(|_, w| w.slewrate().slow());
                    }
                });
                self
            }

            fn set_drive_strength(&self, strength: DriveStrength) -> &Self {
                critical_section::with(|_| match strength {
                    DriveStrength::Normal => {
                        self.reg.modify(|_, w| w.fulldrive().normal_drive());
                    }
                    DriveStrength::Full => {
                        self.reg.modify(|_, w| w.fulldrive().full_drive());
                    }
                });
                self
            }

            fn enable_analog_multiplex(&self) -> &Self {
                critical_section::with(|_| self.reg.modify(|_, w| w.amena().enabled()));
                self
            }

            fn disable_analog_multiplex(&self) -> &Self {
                critical_section::with(|_| self.reg.modify(|_, w| w.amena().disabled()));
                self
            }

            fn set_drive_mode(&self, mode: DriveMode) -> &Self {
                critical_section::with(|_| match mode {
                    DriveMode::PushPull => {
                        self.reg.modify(|_, w| w.odena().disabled());
                    }
                    DriveMode::OpenDrain => {
                        self.reg.modify(|_, w| w.odena().enabled());
                    }
                });
                self
            }

            fn set_input_inverter(&self, inverter: Inverter) -> &Self {
                critical_section::with(|_| match inverter {
                    Inverter::Disabled => {
                        self.reg.modify(|_, w| w.iiena().disabled());
                    }
                    Inverter::Enabled => {
                        self.reg.modify(|_, w| w.iiena().enabled());
                    }
                });
                self
            }

            fn reset(&self) -> &Self {
                self.reg.reset();
                self
            }
        }
    };
}

impl_iopctlpin!(AnyPin);
impl_iopctlpin!(FC15Pin);

macro_rules! impl_FC15pin {
    ($pin_periph:ident, $pin_no:expr) => {
        impl SealedPin for crate::peripherals::$pin_periph {}
        impl ToFC15Pin for crate::peripherals::$pin_periph {}
        impl IopctlPin for crate::peripherals::$pin_periph {
            #[inline]
            fn set_function(&self, _function: Function) -> &Self {
                //No function configuration for FC15 pin
                self
            }

            #[inline]
            fn set_pull(&self, pull: Pull) -> &Self {
                Self::to_raw($pin_no).set_pull(pull);
                self
            }

            #[inline]
            fn enable_input_buffer(&self) -> &Self {
                Self::to_raw($pin_no).enable_input_buffer();
                self
            }

            #[inline]
            fn disable_input_buffer(&self) -> &Self {
                Self::to_raw($pin_no).disable_input_buffer();
                self
            }

            #[inline]
            fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self {
                Self::to_raw($pin_no).set_slew_rate(slew_rate);
                self
            }

            #[inline]
            fn set_drive_strength(&self, strength: DriveStrength) -> &Self {
                Self::to_raw($pin_no).set_drive_strength(strength);
                self
            }

            #[inline]
            fn enable_analog_multiplex(&self) -> &Self {
                Self::to_raw($pin_no).enable_analog_multiplex();
                self
            }

            #[inline]
            fn disable_analog_multiplex(&self) -> &Self {
                Self::to_raw($pin_no).disable_analog_multiplex();
                self
            }

            #[inline]
            fn set_drive_mode(&self, mode: DriveMode) -> &Self {
                Self::to_raw($pin_no).set_drive_mode(mode);
                self
            }

            #[inline]
            fn set_input_inverter(&self, inverter: Inverter) -> &Self {
                Self::to_raw($pin_no).set_input_inverter(inverter);
                self
            }

            #[inline]
            fn reset(&self) -> &Self {
                Self::to_raw($pin_no).reset();
                self
            }
        }
    };
}

macro_rules! impl_pin {
    ($pin_periph:ident, $pin_port:expr, $pin_no:expr) => {
        impl SealedPin for crate::peripherals::$pin_periph {}
        impl ToAnyPin for crate::peripherals::$pin_periph {}
        impl IopctlPin for crate::peripherals::$pin_periph {
            #[inline]
            fn set_function(&self, function: Function) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_function(function);
                self
            }

            #[inline]
            fn set_pull(&self, pull: Pull) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_pull(pull);
                self
            }

            #[inline]
            fn enable_input_buffer(&self) -> &Self {
                Self::to_raw($pin_port, $pin_no).enable_input_buffer();
                self
            }

            #[inline]
            fn disable_input_buffer(&self) -> &Self {
                Self::to_raw($pin_port, $pin_no).disable_input_buffer();
                self
            }

            #[inline]
            fn set_slew_rate(&self, slew_rate: SlewRate) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_slew_rate(slew_rate);
                self
            }

            #[inline]
            fn set_drive_strength(&self, strength: DriveStrength) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_drive_strength(strength);
                self
            }

            #[inline]
            fn enable_analog_multiplex(&self) -> &Self {
                Self::to_raw($pin_port, $pin_no).enable_analog_multiplex();
                self
            }

            #[inline]
            fn disable_analog_multiplex(&self) -> &Self {
                Self::to_raw($pin_port, $pin_no).disable_analog_multiplex();
                self
            }

            #[inline]
            fn set_drive_mode(&self, mode: DriveMode) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_drive_mode(mode);
                self
            }

            #[inline]
            fn set_input_inverter(&self, inverter: Inverter) -> &Self {
                Self::to_raw($pin_port, $pin_no).set_input_inverter(inverter);
                self
            }

            #[inline]
            fn reset(&self) -> &Self {
                Self::to_raw($pin_port, $pin_no).reset();
                self
            }
        }
    };
}

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

// Note: These have have reset values of 0x41 to support SWD by default
impl_pin!(PIO2_25, 2, 25);
impl_pin!(PIO2_26, 2, 26);

impl_pin!(PIO2_27, 2, 27);
impl_pin!(PIO2_28, 2, 28);
impl_pin!(PIO2_29, 2, 29);
impl_pin!(PIO2_30, 2, 30);
impl_pin!(PIO2_31, 2, 31);
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
impl_pin!(PIO7_24, 7, 24);
impl_pin!(PIO7_25, 7, 25);
impl_pin!(PIO7_26, 7, 26);
impl_pin!(PIO7_27, 7, 27);
impl_pin!(PIO7_28, 7, 28);
impl_pin!(PIO7_29, 7, 29);
impl_pin!(PIO7_30, 7, 30);
impl_pin!(PIO7_31, 7, 31);

// FC15 pins
impl_FC15pin!(PIOFC15_SCL, 0);
impl_FC15pin!(PIOFC15_SDA, 1);
