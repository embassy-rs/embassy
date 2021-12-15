#![macro_use]

use crate::interrupt::Interrupt;
use crate::pac;

use core::marker::PhantomData;
use embassy::util::Unborrow;
use nrf_usbd::{UsbPeripheral, Usbd};
use usb_device::bus::UsbBusAllocator;

pub use embassy_hal_common::usb::*;

pub struct UsbBus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

unsafe impl<'d, T: Instance> UsbPeripheral for UsbBus<'d, T> {
    // todo how to use T::regs
    const REGISTERS: *const () = pac::USBD::ptr() as *const ();
}

impl<'d, T: Instance> UsbBus<'d, T> {
    pub fn new(_usb: impl Unborrow<Target = T> + 'd) -> UsbBusAllocator<Usbd<UsbBus<'d, T>>> {
        let r = T::regs();

        r.intenset.write(|w| {
            w.sof().set_bit();
            w.usbevent().set_bit();
            w.ep0datadone().set_bit();
            w.ep0setup().set_bit();
            w.usbreset().set_bit()
        });

        Usbd::new(UsbBus {
            phantom: PhantomData,
        })
    }
}

unsafe impl embassy_hal_common::usb::USBInterrupt for crate::interrupt::USBD {}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::usbd::RegisterBlock;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static + Send {
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
