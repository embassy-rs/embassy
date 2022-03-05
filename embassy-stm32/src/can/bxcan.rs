use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::AFType;
use crate::{peripherals, rcc::RccPeripheral};

pub use bxcan;

pub struct Can<'d, T: Instance + bxcan::Instance> {
    phantom: PhantomData<&'d mut T>,
    can: bxcan::Can<T>,
}

impl<'d, T: Instance + bxcan::Instance> Can<'d, T> {
    pub fn new(
        peri: impl Unborrow<Target = T> + 'd,
        rx: impl Unborrow<Target = impl RxPin<T>> + 'd,
        tx: impl Unborrow<Target = impl TxPin<T>> + 'd,
    ) -> Self {
        unborrow!(peri, rx, tx);

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        T::enable();
        T::reset();

        Self {
            phantom: PhantomData,
            can: bxcan::Can::builder(peri).enable(),
        }
    }
}

impl<'d, T: Instance + bxcan::Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        unsafe {
            T::regs().mcr().write(|w| w.set_reset(true));
        }
        T::disable();
    }
}

impl<'d, T: Instance + bxcan::Instance> Deref for Can<'d, T> {
    type Target = bxcan::Can<T>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance + bxcan::Instance> DerefMut for Can<'d, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::can::Can;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::can::Can {
                &crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {}

        unsafe impl bxcan::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.0 as *mut _;
        }
    };
);

foreach_peripheral!(
    (can, CAN) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    // CAN1 and CAN2 is a combination of master and slave instance.
    // CAN1 owns the filter bank and needs to be enabled in order
    // for CAN2 to receive messages.
    (can, CAN1) => {
        cfg_if::cfg_if! {
            if #[cfg(all(
                any(stm32l4, stm32f72, stm32f73),
                not(any(stm32l49, stm32l4a))
            ))] {
                // Most L4 devices and some F7 devices use the name "CAN1"
                // even if there is no "CAN2" peripheral.
                unsafe impl bxcan::FilterOwner for peripherals::CAN1 {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl bxcan::FilterOwner for peripherals::CAN1 {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl bxcan::MasterInstance for peripherals::CAN1 {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN3 {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
