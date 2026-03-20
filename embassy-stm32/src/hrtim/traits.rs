use embassy_hal_internal::PeripheralType;

use crate::rcc::RccPeripheral;

pub(crate) trait SealedInstance: RccPeripheral {}

/// HRTIM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

foreach_interrupt! {
    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst { }

        impl Instance for crate::peripherals::$inst {

        }
    };
}
