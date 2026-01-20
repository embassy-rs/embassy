//! SPI pin trait definitions and implementations.

use embassy_hal_internal::PeripheralType;

use super::common::sealed;
use crate::gpio::{GpioPin, SealedPin};

// =============================================================================
// Pin Traits
// =============================================================================

/// SCK pin trait.
pub trait SckPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// MOSI/SDO pin trait.
pub trait MosiPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// MISO/SDI pin trait.
pub trait MisoPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// PCS/CS pin trait.
pub trait CsPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

macro_rules! impl_spi_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        impl $trait<crate::peripherals::$peri> for crate::peripherals::$pin {
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port0::pcr0::Mux::$fn);
                self.set_enable_input_buffer();
            }
        }
    };
}

// LPSPI0 pins on PORT1 (ALT2) - per reference board pin mux
impl_spi_pin!(P1_0, LPSPI0, Mux2, MosiPin); // LPSPI0_SDO (SOUT)
impl_spi_pin!(P1_1, LPSPI0, Mux2, SckPin); // LPSPI0_SCK
impl_spi_pin!(P1_2, LPSPI0, Mux2, MisoPin); // LPSPI0_SDI (SIN)
impl_spi_pin!(P1_3, LPSPI0, Mux2, CsPin); // LPSPI0_PCS0

// LPSPI1 pins on PORT3 (ALT2)
impl_spi_pin!(P3_8, LPSPI1, Mux2, MosiPin); // LPSPI1_SOUT
impl_spi_pin!(P3_9, LPSPI1, Mux2, MisoPin); // LPSPI1_SIN
impl_spi_pin!(P3_10, LPSPI1, Mux2, SckPin); // LPSPI1_SCK
impl_spi_pin!(P3_11, LPSPI1, Mux2, CsPin); // LPSPI1_PCS0
