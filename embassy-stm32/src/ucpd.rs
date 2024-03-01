//! USB Type-C/USB Power Delivery Interface (UCPD)

use crate::rcc::RccPeripheral;

/// UCPD instance trait.
pub trait Instance: sealed::Instance + RccPeripheral {}

pub(crate) mod sealed {
    pub trait Instance {
        const REGS: crate::pac::ucpd::Ucpd;
    }
}

foreach_peripheral!(
    (ucpd, $inst:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            const REGS: crate::pac::ucpd::Ucpd = crate::pac::$inst;
        }

        impl Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(Cc1Pin, Instance);
pin_trait!(Cc2Pin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);
