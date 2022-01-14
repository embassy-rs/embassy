use crate::pac::{RCC, USB};

pub use embassy_hal_common::usb::*;

unsafe impl embassy_hal_common::usb::USBInterrupt for crate::interrupt::USB_LP {}

pub struct Peripheral {}

unsafe impl Sync for Peripheral {}

unsafe impl stm32_usbd::UsbPeripheral for Peripheral {
    const REGISTERS: *const () = USB as *const ();

    #[cfg(any(stm32l0, stm32l1))]
    const DP_PULL_UP_FEATURE: bool = true;
    #[cfg(any(stm32f1, stm32f3))]
    const DP_PULL_UP_FEATURE: bool = false;

    const EP_MEMORY: *const () = 0x4000_6000 as _;

    #[cfg(any(stm32f1, stm32l1, stm32f303xb, stm32f303xc))]
    const EP_MEMORY_SIZE: usize = 512;
    #[cfg(any(stm32l0, stm32f303xd, stm32f303xe))]
    const EP_MEMORY_SIZE: usize = 1024;

    #[cfg(any(stm32f1, stm32l1, stm32f303xb, stm32f303xc))]
    const EP_MEMORY_ACCESS_2X16: bool = false;
    #[cfg(any(stm32l0, stm32f303xd, stm32f303xe))]
    const EP_MEMORY_ACCESS_2X16: bool = true;

    fn enable() {
        unsafe {
            cortex_m::interrupt::free(|_| {
                // Enable USB peripheral
                RCC.apb1enr().modify(|w| w.set_usben(true));

                // Reset USB peripheral
                RCC.apb1rstr().modify(|w| w.set_usbrst(true));
                RCC.apb1rstr().modify(|w| w.set_usbrst(false));
            });
        }
    }

    fn startup_delay() {
        // There is a chip specific startup delay.
        // 72 MHz is the highest frequency across all chips with this peripheral,
        // so this should ensure a minimum 1µs wait time.
        cortex_m::asm::delay(72);
    }
}

pub type UsbBus = stm32_usbd::UsbBus<Peripheral>;
