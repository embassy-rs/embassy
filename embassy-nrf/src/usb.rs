#![macro_use]

use core::marker::PhantomData;
use embassy::util::Unborrow;

use crate::interrupt::Interrupt;
use crate::pac;
use nrf_usbd::{UsbPeripheral, Usbd};
use usb_device::bus::UsbBusAllocator;

// todo using different type than Usb because T isnt Send
pub struct UsbBus;
unsafe impl UsbPeripheral for UsbBus {
    // todo hardcoding
    const REGISTERS: *const () = crate::pac::USBD::ptr() as *const ();
}

impl UsbBus {
    pub fn new() -> UsbBusAllocator<Usbd<UsbBus>> {
        Usbd::new(UsbBus)
    }
}

unsafe impl embassy_hal_common::usb::USBInterrupt for crate::interrupt::USBD {}

pub struct Usb<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Usb<'d, T> {
    #[allow(unused_unsafe)]
    pub fn new(_usb: impl Unborrow<Target = T> + 'd) -> Self {
        let r = T::regs();

        Self {
            phantom: PhantomData,
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::usbd::RegisterBlock;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_usb {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::usb::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::usbd::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
        }
        impl crate::usb::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
