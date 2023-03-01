// https://github.com/stm32-rs/stm32-device-signature
// [package]
// name = "stm32-device-signature"
// version = "0.3.3"
// authors = ["Vadim Kaushan <admin@disasm.info>"]
// license = "MIT OR Apache-2.0"
//! This module provides a way to access Device electronic signature
//! items on STM32 microcontrollers.
//!
//! You need to pass one of the features in order to use this crate:
//! * `stm32f0`
//! * `stm32f1`
//! * `stm32f2`
//! * `stm32f3`
//! * `stm32f4`
//! * `stm32f72x`
//! * `stm32f73x`
//! * `stm32f76x`
//! * `stm32f77x`
//! * `stm32g0`
//! * `stm32h72x`
//! * `stm32h73x`
//! * `stm32h74x`
//! * `stm32h75x`
//! * `stm32h7ax`
//! * `stm32h7bx`
//! * `stm32l0`
//! * `stm32l4`
//! * `stm32wb5x`

use cortex_m::interrupt;

#[cfg(any(stm32f0, stm32f3))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FFF_F7AC as _;
}

#[cfg(stm32f1)]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FFF_F7E8 as _;
}

#[cfg(any(stm32f2, stm32f4))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FFF_7A10 as _;
}

#[cfg(any(stm32f72x, stm32f73x))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FF0_7A10 as _;
}

#[cfg(any(stm32f76x, stm32f77x))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FF0_F420 as _;
}

#[cfg(any(stm32g0, stm32l4, stm32wb5x))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FFF_7590 as _;
}

#[cfg(any(stm32h72x, stm32h73x, stm32h74x, stm32h75x))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FF1_E800 as _;
}

#[cfg(any(stm32h7ax, stm32h7bx))]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x08FF_F800 as _;
}

#[cfg(stm32l0)]
mod pointers {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FF8_0050 as _;
}

use pointers::*;

/// Returns a 12-byte unique device ID
pub fn device_id() -> &'static [u8; 12] {
    unsafe { &*DEVICE_ID_PTR.cast::<[u8; 12]>() }
}

/// Returns a string with a hex-encoded unique device ID
pub fn device_id_hex() -> &'static str {
    static mut DEVICE_ID_STR: [u8; 24] = [0; 24];

    unsafe {
        if DEVICE_ID_STR.as_ptr().read_volatile() == 0 {
            interrupt::free(|_| {
                let hex = b"0123456789abcdef";
                for (i, b) in device_id().iter().enumerate() {
                    let lo = b & 0xf;
                    let hi = (b >> 4) & 0xfu8;
                    DEVICE_ID_STR[i * 2] = hex[hi as usize];
                    DEVICE_ID_STR[i * 2 + 1] = hex[lo as usize];
                }
            });
        }

        core::str::from_utf8_unchecked(&DEVICE_ID_STR)
    }
}
