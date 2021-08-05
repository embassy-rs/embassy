use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::Pin;
use crate::{peripherals, rcc::RccPeripheral};

pub use bxcan::*;

pub struct Can<'d, T: Instance + bxcan::Instance> {
    phantom: PhantomData<&'d mut T>,
    can: bxcan::Can<T>,
}

impl<'d, T: Instance + bxcan::Instance> Can<'d, T> {
    pub fn new(
        peri: impl Unborrow<Target = T> + 'd,
        // irq: impl Unborrow<Target = T::Interrupt> + 'd,
        rx: impl Unborrow<Target = impl RxPin<T>> + 'd,
        tx: impl Unborrow<Target = impl TxPin<T>> + 'd,
    ) -> Self {
        unborrow!(peri, rx, tx);

        unsafe {
            rx.set_as_af(rx.af_num());
            tx.set_as_af(tx.af_num());

            T::enable();
            T::reset();
            // TODO: CAN2 also required CAN1 clock
        }

        Self {
            phantom: PhantomData,
            can: bxcan::Can::new(peri),
        }
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
    use super::*;

    pub trait Instance {}

    pub trait RxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait TxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> {}
pub trait TxPin<T: Instance>: sealed::TxPin<T> {}

crate::pac::peripherals!(
    (bxcan, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {}

        impl Instance for peripherals::$inst {}

        unsafe impl bxcan::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.0 as *mut _;
        }
    };
    // (bxcan, CAN) => {
    //     unsafe impl bxcan::FilterOwner for Can<peripherals::CAN> {
    //         const NUM_FILTER_BANKS: u8 = 14;
    //     }
    // };
);

crate::pac::peripherals!(
    // TODO: rename CAN to CAN1 on yaml level??
    (bxcan, CAN) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    (bxcan, CAN1) => {
        unsafe impl bxcan::FilterOwner for peripherals::CAN1 {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    (bxcan, CAN2) => {
        // TODO: when CAN2 existis, we have 28 filter banks
        unsafe impl bxcan::MasterInstance for peripherals::CAN1 {}
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
    ($inst:ident, bxcan, CAN, $pin:ident, TX, $af:expr) => {
        impl_pin!($inst, $pin, TxPin, $af);
    };
    ($inst:ident, bxcan, CAN, $pin:ident, RX, $af:expr) => {
        impl_pin!($inst, $pin, RxPin, $af);
    };
);
