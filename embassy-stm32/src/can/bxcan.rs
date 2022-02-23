use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::AFType;
use crate::{peripherals, rcc::RccPeripheral};

pub use bxcan::*;

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

crate::pac::peripherals!(
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

crate::pac::peripherals!(
    (can, CAN) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    // Only correct when CAN2 also exists… Fix on yaml level?
    // There are only 14 filter banks when CAN2 is not available.
    (can, CAN1) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN1 {
            const NUM_FILTER_BANKS: u8 = 28;
        }
    };
    (can, CAN2) => {
        // CAN2 is always a slave instance where CAN1 is the master instance
        unsafe impl bxcan::MasterInstance for peripherals::CAN1 {}
    };
    (can, CAN3) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN3 {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
