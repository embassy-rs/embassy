use crate::gpio::sealed::{AFType, Pin};
use crate::{peripherals, rcc::RccPeripheral};
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
pub use embassy_hal_common::usb::*;
pub use synopsys_usb_otg::UsbBus;
use synopsys_usb_otg::UsbPeripheral;

pub struct UsbOtgFs<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> UsbOtgFs<'d, T> {
    pub fn new(
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
        }
    }
}

impl<'d, T: Instance> Drop for UsbOtgFs<'d, T> {
    fn drop(&mut self) {
        T::reset();
        T::disable();
    }
}

unsafe impl<'d, T: Instance> Send for UsbOtgFs<'d, T> {}
unsafe impl<'d, T: Instance> Sync for UsbOtgFs<'d, T> {}

unsafe impl<'d, T: Instance> UsbPeripheral for UsbOtgFs<'d, T> {
    const REGISTERS: *const () = T::REGISTERS;

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

    fn enable() {
        <T as crate::rcc::sealed::RccPeripheral>::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();
    }

    fn ahb_frequency_hz(&self) -> u32 {
        <T as crate::rcc::sealed::RccPeripheral>::frequency().0
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGISTERS: *const ();
    }

    pub trait DpPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait DmPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}
pub trait DpPin<T: Instance>: sealed::DpPin<T> {}
pub trait DmPin<T: Instance>: sealed::DmPin<T> {}

crate::pac::peripherals!(
    (otgfs, $inst:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            const REGISTERS: *const () = crate::pac::$inst.0 as *const ();
        }

        impl Instance for peripherals::$inst {}
    };
);

crate::pac::interrupts!(
    ($inst:ident, otgfs, $block:ident, GLOBAL, $irq:ident) => {
        unsafe impl USBInterrupt for crate::interrupt::$irq {}
    };
);

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $signal:ident, $af:expr) => {
        impl $signal<peripherals::$inst> for peripherals::$pin {}

        impl sealed::$signal<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, otgfs, OTG_FS, $pin:ident, DP, $af:expr) => {
        impl_pin!($inst, $pin, DpPin, $af);
    };
    ($inst:ident, otgfs, OTG_FS, $pin:ident, DM, $af:expr) => {
        impl_pin!($inst, $pin, DmPin, $af);
    };
    ($inst:ident, otgfs, OTG_FS, $pin:ident, DP) => {
        impl_pin!($inst, $pin, DpPin, 0);
    };
    ($inst:ident, otgfs, OTG_FS, $pin:ident, DM) => {
        impl_pin!($inst, $pin, DmPin, 0);
    };
);
