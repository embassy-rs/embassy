use crate::interrupt;
use crate::rcc::RccPeripheral;

#[cfg(feature = "nightly")]
mod usb;
#[cfg(feature = "nightly")]
pub use usb::*;

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::usb::Usb;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral + 'static {
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Internal PHY pins
pin_trait!(DpPin, Instance);
pin_trait!(DmPin, Instance);

foreach_interrupt!(
    ($inst:ident, usb, $block:ident, LP, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            fn regs() -> crate::pac::usb::Usb {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
