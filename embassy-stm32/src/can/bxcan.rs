use core::ops::{Deref, DerefMut};

pub use bxcan;
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::gpio::sealed::AFType;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

pub struct Can<'d, T: Instance> {
    can: bxcan::Can<BxcanInstance<'d, T>>,
}

impl<'d, T: Instance> Can<'d, T> {
    /// Creates a new Bxcan instance, blocking for 11 recessive bits to sync with the CAN bus.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable();
        T::reset();

        Self {
            can: bxcan::Can::builder(BxcanInstance(peri)).enable(),
        }
    }

    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new_disabled(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable();
        T::reset();

        Self {
            can: bxcan::Can::builder(BxcanInstance(peri)).leave_disabled(),
        }
    }
}

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        T::regs().mcr().write(|w| w.set_reset(true));
        T::disable();
    }
}

impl<'d, T: Instance> Deref for Can<'d, T> {
    type Target = bxcan::Can<BxcanInstance<'d, T>>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance> DerefMut for Can<'d, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        const REGISTERS: *mut bxcan::RegisterBlock;

        fn regs() -> &'static crate::pac::can::Can;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}

pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> bxcan::Instance for BxcanInstance<'d, T> {
    const REGISTERS: *mut bxcan::RegisterBlock = T::REGISTERS;
}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.as_ptr() as *mut _;

            fn regs() -> &'static crate::pac::can::Can {
                &crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {}

    };
);

foreach_peripheral!(
    (can, CAN) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN> {
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
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl<'d> bxcan::MasterInstance for BxcanInstance<'d, peripherals::CAN1> {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN3> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
