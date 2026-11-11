//! QMI CS1 peripheral for RP235x
//!
//! This module provides access to the QMI CS1 functionality for use with external memory devices
//! such as PSRAM. The QMI (Quad SPI) controller supports CS1 as a second chip select signal.
//!
//! This peripheral is only available on RP235x chips.

#![cfg(feature = "_rp235x")]

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::{Flex, Pin as GpioPin};
use crate::{pac, peripherals};

/// QMI CS1 driver.
pub struct QmiCs1<'d> {
    _inner: Peri<'d, peripherals::QMI_CS1>,
}

impl<'d> QmiCs1<'d> {
    /// Create a new QMI CS1 instance.
    pub fn new(qmi_cs1: Peri<'d, peripherals::QMI_CS1>, cs1: Peri<'d, impl QmiCs1Pin>) -> Self {
        let mut cs1 = Flex::new_unconfigured(cs1);

        // Configure CS1 pin for QMI function (funcsel = 9)
        cs1.set_function(9);

        // Configure pad settings for high-speed operation
        cs1.write_pad(|w| {
            w.set_ie(true);
            w.set_drive(pac::pads::vals::Drive::_12mA);
            w.set_slewfast(true);
        });

        // The pin stays configured when the driver is dropped.
        core::mem::forget(cs1);

        Self { _inner: qmi_cs1 }
    }
}

trait SealedInstance {}

/// QMI CS1 instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

impl SealedInstance for peripherals::QMI_CS1 {}

impl Instance for peripherals::QMI_CS1 {}

/// CS1 pin trait for QMI.
pub trait QmiCs1Pin: GpioPin {}

// Implement pin traits for CS1-capable GPIO pins
impl QmiCs1Pin for peripherals::PIN_0 {}
impl QmiCs1Pin for peripherals::PIN_8 {}
impl QmiCs1Pin for peripherals::PIN_19 {}
#[cfg(feature = "rp235xb")]
impl QmiCs1Pin for peripherals::PIN_47 {}
