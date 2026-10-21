//! Isolated GPIO/PORT register fixtures for HAL unit tests.

extern crate std;

use std::boxed::Box;

use crate::Peri;
use crate::gpio::AnyPin;

pub(crate) fn pin() -> Peri<'static, AnyPin> {
    pin_at(0, 0)
}

pub(crate) fn pin_at(port_index: u8, pin_index: u8) -> Peri<'static, AnyPin> {
    let gpio_memory = Box::leak(Box::new([0u32; 256]));
    let port_memory = Box::leak(Box::new([0u32; 256]));
    // SAFETY: these uniquely allocated, aligned blocks cover the registers used
    // by the fixture. No real GPIO singleton or hardware address is accessed.
    let gpio = unsafe { crate::pac::gpio::Gpio::from_ptr(gpio_memory.as_mut_ptr().cast()) };
    // SAFETY: the isolated block covers PORT and its PCR array for the fixture's lifetime.
    let port = unsafe { crate::pac::port::Port::from_ptr(port_memory.as_mut_ptr().cast()) };
    let pin = AnyPin::new(
        port_index,
        pin_index,
        gpio,
        port,
        port.pcr(usize::from(pin_index)),
        true,
    );
    // SAFETY: this pin owns only the isolated test registers.
    unsafe { Peri::new_unchecked(pin) }
}
