//! Stub GPU2D HAL for link tests and bring-up without a real peripheral.
//!
//! On hardware, bind the real peripheral via [`crate::gpu2d_bridge`] and build
//! without the default `stub-gpu2d` feature.

use core::ffi::c_void;

/// GPU2D breakpoint register, polled by `nema_wait_irq_brk()`.
const GPU2D_BREAKPOINT: u32 = 0x0000_0080;

const HAL_OK: i32 = 0;
const HAL_ERROR: i32 = 1;

/// C ABI mirror of STM32Cube's `GPU2D_HandleTypeDef`.
#[repr(C)]
pub struct Gpu2dHandleTypeDef {
    instance: *mut c_void,
}

#[unsafe(no_mangle)]
static mut hgpu2d: Gpu2dHandleTypeDef = Gpu2dHandleTypeDef {
    instance: core::ptr::null_mut(),
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_GPU2D_Init(handle: *mut Gpu2dHandleTypeDef) -> i32 {
    if handle.is_null() {
        return HAL_ERROR;
    }
    if (*handle).instance.is_null() {
        (*handle).instance = 0x5000_0000 as *mut c_void;
    }
    HAL_OK
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_GPU2D_ReadRegister(_handle: *mut Gpu2dHandleTypeDef, reg: u32) -> u32 {
    if reg == GPU2D_BREAKPOINT { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_GPU2D_WriteRegister(_handle: *mut Gpu2dHandleTypeDef, _reg: u32, _value: u32) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_GPU2D_PollCompletion(_handle: *mut Gpu2dHandleTypeDef) {
    // The stub never completes anything; nothing to poll.
}

/// Initialize the stubbed GPU2D handle; `0` on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_gfx_hal_gpu2d_init() -> i32 {
    if HAL_GPU2D_Init(&raw mut hgpu2d) == HAL_OK {
        0
    } else {
        -1
    }
}
