#![macro_use]

pub use embassy_hal_common::usb::*;
use nrf_usbd::{UsbPeripheral, Usbd};
use usb_device::bus::UsbBusAllocator;

pub struct UsbBus;
unsafe impl UsbPeripheral for UsbBus {
    // todo hardcoding
    const REGISTERS: *const () = crate::pac::USBD::ptr() as *const ();
}

impl UsbBus {
    // todo should it consume a USBD peripheral?
    pub fn new() -> UsbBusAllocator<Usbd<UsbBus>> {
        unsafe {
            (*crate::pac::USBD::ptr()).intenset.write(|w| {
                w.sof().set_bit();
                w.usbevent().set_bit();
                w.ep0datadone().set_bit();
                w.ep0setup().set_bit();
                w.usbreset().set_bit()
            })
        };

        Usbd::new(UsbBus)
    }
}

unsafe impl embassy_hal_common::usb::USBInterrupt for crate::interrupt::USBD {}
