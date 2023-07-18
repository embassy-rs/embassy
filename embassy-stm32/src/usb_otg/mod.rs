use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals};

#[cfg(feature = "nightly")]
mod usb;
#[cfg(feature = "nightly")]
pub use usb::*;

// Using Instance::ENDPOINT_COUNT requires feature(const_generic_expr) so just define maximum eps
#[cfg(feature = "nightly")]
const MAX_EP_COUNT: usize = 9;

pub(crate) mod sealed {
    pub trait Instance {
        const HIGH_SPEED: bool;
        const FIFO_DEPTH_WORDS: u16;
        const ENDPOINT_COUNT: usize;

        fn regs() -> crate::pac::otg::Otg;
        #[cfg(feature = "nightly")]
        fn state() -> &'static super::State<{ super::MAX_EP_COUNT }>;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Internal PHY pins
pin_trait!(DpPin, Instance);
pin_trait!(DmPin, Instance);

// External PHY pins
pin_trait!(UlpiClkPin, Instance);
pin_trait!(UlpiDirPin, Instance);
pin_trait!(UlpiNxtPin, Instance);
pin_trait!(UlpiStpPin, Instance);
pin_trait!(UlpiD0Pin, Instance);
pin_trait!(UlpiD1Pin, Instance);
pin_trait!(UlpiD2Pin, Instance);
pin_trait!(UlpiD3Pin, Instance);
pin_trait!(UlpiD4Pin, Instance);
pin_trait!(UlpiD5Pin, Instance);
pin_trait!(UlpiD6Pin, Instance);
pin_trait!(UlpiD7Pin, Instance);

foreach_interrupt!(
    (USB_OTG_FS, otg, $block:ident, GLOBAL, $irq:ident) => {
        impl sealed::Instance for peripherals::USB_OTG_FS {
            const HIGH_SPEED: bool = false;

            cfg_if::cfg_if! {
                if #[cfg(stm32f1)] {
                    const FIFO_DEPTH_WORDS: u16 = 128;
                    const ENDPOINT_COUNT: usize = 8;
                } else if #[cfg(any(
                    stm32f2,
                    stm32f401,
                    stm32f405,
                    stm32f407,
                    stm32f411,
                    stm32f415,
                    stm32f417,
                    stm32f427,
                    stm32f429,
                    stm32f437,
                    stm32f439,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 4;
                } else if #[cfg(any(
                    stm32f412,
                    stm32f413,
                    stm32f423,
                    stm32f446,
                    stm32f469,
                    stm32f479,
                    stm32f7,
                    stm32l4,
                    stm32u5,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(stm32g0x1)] {
                    const FIFO_DEPTH_WORDS: u16 = 512;
                    const ENDPOINT_COUNT: usize = 8;
                } else if #[cfg(stm32h7)] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else if #[cfg(stm32u5)] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 6;
                } else {
                    compile_error!("USB_OTG_FS peripheral is not supported by this chip.");
                }
            }

            fn regs() -> crate::pac::otg::Otg {
                crate::pac::USB_OTG_FS
            }

            #[cfg(feature = "nightly")]
            fn state() -> &'static State<MAX_EP_COUNT> {
                static STATE: State<MAX_EP_COUNT> = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::USB_OTG_FS {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };

    (USB_OTG_HS, otg, $block:ident, GLOBAL, $irq:ident) => {
        impl sealed::Instance for peripherals::USB_OTG_HS {
            const HIGH_SPEED: bool = true;

            cfg_if::cfg_if! {
                if #[cfg(any(
                    stm32f2,
                    stm32f405,
                    stm32f407,
                    stm32f415,
                    stm32f417,
                    stm32f427,
                    stm32f429,
                    stm32f437,
                    stm32f439,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(any(
                    stm32f446,
                    stm32f469,
                    stm32f479,
                    stm32f7,
                    stm32h7,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else if #[cfg(stm32u5)] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else {
                    compile_error!("USB_OTG_HS peripheral is not supported by this chip.");
                }
            }

            fn regs() -> crate::pac::otg::Otg {
                // OTG HS registers are a superset of FS registers
                unsafe { crate::pac::otg::Otg::from_ptr(crate::pac::USB_OTG_HS.as_ptr()) }
            }

            #[cfg(feature = "nightly")]
            fn state() -> &'static State<MAX_EP_COUNT> {
                static STATE: State<MAX_EP_COUNT> = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::USB_OTG_HS {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
