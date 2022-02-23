use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use synopsys_usb_otg::{PhyType, UsbPeripheral};

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
use crate::{peripherals, rcc::RccPeripheral};

pub use embassy_hal_common::usb::*;
pub use synopsys_usb_otg::UsbBus;

macro_rules! config_ulpi_pins {
    ($($pin:ident),*) => {
        unborrow!($($pin),*);
        // NOTE(unsafe) Exclusive access to the registers
        critical_section::with(|_| unsafe {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                $pin.set_speed(Speed::VeryHigh);
            )*
        })
    };
}

pub struct UsbOtg<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    phy_type: PhyType,
}

impl<'d, T: Instance> UsbOtg<'d, T> {
    /// Initializes USB OTG peripheral with internal Full-Speed PHY
    pub fn new_fs(
        _peri: impl Unborrow<Target = T> + 'd,
        dp: impl Unborrow<Target = impl DpPin<T>> + 'd,
        dm: impl Unborrow<Target = impl DmPin<T>> + 'd,
    ) -> Self {
        unborrow!(dp, dm);

        unsafe {
            dp.set_as_af(dp.af_num(), AFType::OutputPushPull);
            dm.set_as_af(dm.af_num(), AFType::OutputPushPull);
        }

        Self {
            phantom: PhantomData,
            phy_type: PhyType::InternalFullSpeed,
        }
    }

    /// Initializes USB OTG peripheral with external High-Speed PHY
    pub fn new_hs_ulpi(
        _peri: impl Unborrow<Target = T> + 'd,
        ulpi_clk: impl Unborrow<Target = impl UlpiClkPin<T>> + 'd,
        ulpi_dir: impl Unborrow<Target = impl UlpiDirPin<T>> + 'd,
        ulpi_nxt: impl Unborrow<Target = impl UlpiNxtPin<T>> + 'd,
        ulpi_stp: impl Unborrow<Target = impl UlpiStpPin<T>> + 'd,
        ulpi_d0: impl Unborrow<Target = impl UlpiD0Pin<T>> + 'd,
        ulpi_d1: impl Unborrow<Target = impl UlpiD1Pin<T>> + 'd,
        ulpi_d2: impl Unborrow<Target = impl UlpiD2Pin<T>> + 'd,
        ulpi_d3: impl Unborrow<Target = impl UlpiD3Pin<T>> + 'd,
        ulpi_d4: impl Unborrow<Target = impl UlpiD4Pin<T>> + 'd,
        ulpi_d5: impl Unborrow<Target = impl UlpiD5Pin<T>> + 'd,
        ulpi_d6: impl Unborrow<Target = impl UlpiD6Pin<T>> + 'd,
        ulpi_d7: impl Unborrow<Target = impl UlpiD7Pin<T>> + 'd,
    ) -> Self {
        config_ulpi_pins!(
            ulpi_clk, ulpi_dir, ulpi_nxt, ulpi_stp, ulpi_d0, ulpi_d1, ulpi_d2, ulpi_d3, ulpi_d4,
            ulpi_d5, ulpi_d6, ulpi_d7
        );

        Self {
            phantom: PhantomData,
            phy_type: PhyType::ExternalHighSpeed,
        }
    }
}

impl<'d, T: Instance> Drop for UsbOtg<'d, T> {
    fn drop(&mut self) {
        T::reset();
        T::disable();
    }
}

unsafe impl<'d, T: Instance> Send for UsbOtg<'d, T> {}
unsafe impl<'d, T: Instance> Sync for UsbOtg<'d, T> {}

unsafe impl<'d, T: Instance> UsbPeripheral for UsbOtg<'d, T> {
    const REGISTERS: *const () = T::REGISTERS;
    const HIGH_SPEED: bool = T::HIGH_SPEED;
    const FIFO_DEPTH_WORDS: usize = T::FIFO_DEPTH_WORDS;
    const ENDPOINT_COUNT: usize = T::ENDPOINT_COUNT;

    fn enable() {
        <T as crate::rcc::sealed::RccPeripheral>::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();
    }

    fn phy_type(&self) -> PhyType {
        self.phy_type
    }

    fn ahb_frequency_hz(&self) -> u32 {
        <T as crate::rcc::sealed::RccPeripheral>::frequency().0
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        const REGISTERS: *const ();
        const HIGH_SPEED: bool;
        const FIFO_DEPTH_WORDS: usize;
        const ENDPOINT_COUNT: usize;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}

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

crate::pac::peripherals!(
    (otgfs, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *const () = crate::pac::$inst.0 as *const ();
            const HIGH_SPEED: bool = false;

            cfg_if::cfg_if! {
                if #[cfg(stm32f1)] {
                    const FIFO_DEPTH_WORDS: usize = 128;
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
                    const FIFO_DEPTH_WORDS: usize = 320;
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
                    const FIFO_DEPTH_WORDS: usize = 320;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(stm32g0x1)] {
                    const FIFO_DEPTH_WORDS: usize = 512;
                    const ENDPOINT_COUNT: usize = 8;
                } else {
                    compile_error!("USB_OTG_FS peripheral is not supported by this chip. Disable \"usb-otg-fs\" feature or select a different chip.");
                }
            }
        }

        impl Instance for peripherals::$inst {}
    };

    (otghs, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *const () = crate::pac::$inst.0 as *const ();
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
                    const FIFO_DEPTH_WORDS: usize = 1024;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(any(
                    stm32f446,
                    stm32f469,
                    stm32f479,
                    stm32f7,
                    stm32h7,
                ))] {
                    const FIFO_DEPTH_WORDS: usize = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else {
                    compile_error!("USB_OTG_HS peripheral is not supported by this chip. Disable \"usb-otg-hs\" feature or select a different chip.");
                }
            }
        }

        impl Instance for peripherals::$inst {}
    };
);

crate::pac::interrupts!(
    ($inst:ident, otgfs, $block:ident, GLOBAL, $irq:ident) => {
        unsafe impl USBInterrupt for crate::interrupt::$irq {}
    };
    ($inst:ident, otghs, $block:ident, GLOBAL, $irq:ident) => {
        unsafe impl USBInterrupt for crate::interrupt::$irq {}
    };
);
