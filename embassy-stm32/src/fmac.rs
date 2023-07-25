use embassy_hal_common::{into_ref, PeripheralRef};

use crate::pac::FMAC;
use crate::Peripheral;

pub struct FMAC<'d, T: Instance, Tx, Rx> {
    _peri: PeripheralRef<'d, T>,
    txdma: PeripheralRef<'d, Tx>,
    rxdma: PeripheralRef<'d, Rx>,
}

impl<'d, T: Instance, Tx, Rx> FMAC<'d, T, Tx, Rx> {
    pub fn new(peri: impl Peripheral<P = T> + 'd) -> Self {
        unimplemented!()
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }
}
pub trait Instance: Peripheral<P = Self> + sealed::Instance {}

foreach_peripheral!(
    (fmac, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);
