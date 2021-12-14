#![macro_use]

use core::marker::PhantomData;
use embassy::util::Unborrow;

use crate::interrupt::Interrupt;
use crate::pac;
use embassy_hal_common::usb::{ClassSet, IntoClassSet, USBInterrupt};
pub use embassy_hal_common::usb::{ReadInterface, State, UsbSerial, WriteInterface};
use nrf_usbd::{UsbPeripheral, Usbd};
use usb_device::{bus::UsbBusAllocator, class_prelude::UsbBus, device::UsbDevice};

pub struct UsbThing;
unsafe impl UsbPeripheral for UsbThing {
    // todo hardcoding
    const REGISTERS: *const () = crate::pac::USBD::ptr() as *const ();
}

impl UsbThing {
    pub fn new() -> UsbBusAllocator<Usbd<UsbThing>> {
        Usbd::new(UsbThing)
    }
}

unsafe impl embassy_hal_common::usb::USBInterrupt for crate::interrupt::USBD {}

pub struct Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    // Don't you dare moving out `PeripheralMutex`
    usb: embassy_hal_common::usb::Usb<'bus, B, T, I>,
}

impl<'bus, B, T, I> Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    pub unsafe fn new<S: IntoClassSet<B, T>>(
        state: &'bus mut State<'bus, B, T, I>,
        device: UsbDevice<'bus, B>,
        class_set: S,
        irq: I,
    ) -> Self {
        let usb = embassy_hal_common::usb::Usb::new(state, device, class_set, irq);

        Self { usb }
    }
}
