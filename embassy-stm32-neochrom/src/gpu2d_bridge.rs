//! Real GPU2D hardware bridge.
//!
//! Implements the C ABI `nema-gfx-hal`'s baremetal HAL expects from
//! `hal_gpu2d_shim.h`, backed by [`embassy_stm32::gpu2d::Gpu2d`], instead of
//! the CI-only stub in `gpu2d_hal_stub.c`.
//!
//! `embassy_stm32::gpu2d::Gpu2d` now has a real interrupt handler and an
//! async `wait_command_list_complete()`, but NemaGFX's own C code
//! (`nema_wait_irq`/`nema_wait_irq_cl`) is synchronous — it can't `.await`.
//! So `HAL_GPU2D_PollCompletion` bridges the two by synchronously reading
//! (and clearing) the same command-list-complete flag from `nema_wait_irq`'s
//! busy-wait loop, and forwards completions to
//! `HAL_GPU2D_CommandListCpltCallback`, the hook `nema_hal_baremetal.c`
//! already defines. This works whether or not the real interrupt fires in
//! between polls — `command_list_complete()` reads the level-triggered flag
//! directly. A future async-native API on `NeoChrom` could instead submit
//! via `nema_cl_submit_no_irq` and `.await` `wait_command_list_complete()`
//! directly, bypassing this polling bridge entirely.

use core::cell::RefCell;
use core::ffi::c_void;

use critical_section::Mutex;
use embassy_stm32::Peri;
use embassy_stm32::gpu2d::{Gpu2d, Instance as Gpu2dInstance, InterruptHandler as Gpu2dInterruptHandler};
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::peripherals::GPU2D;

static GPU2D_DRIVER: Mutex<RefCell<Option<Gpu2d<'static, GPU2D>>>> = Mutex::new(RefCell::new(None));

/// C ABI mirror of `hal_gpu2d_shim.h`'s `GPU2D_HandleTypeDef`.
#[repr(C)]
pub struct Gpu2dHandleTypeDef {
    instance: *mut c_void,
}

/// Sentinel written to `hgpu2d.instance` once the real driver is bound.
const INITIALIZED: *mut c_void = 1 as *mut c_void;

#[unsafe(no_mangle)]
static mut hgpu2d: Gpu2dHandleTypeDef = Gpu2dHandleTypeDef {
    instance: core::ptr::null_mut(),
};

unsafe extern "C" {
    fn HAL_GPU2D_CommandListCpltCallback(handle: *mut Gpu2dHandleTypeDef, cmd_list_id: u32);
}

/// Bind the real GPU2D peripheral. Call once, before `NeoChrom::new()`.
pub fn init(
    peri: Peri<'static, GPU2D>,
    irq: impl Binding<<GPU2D as Gpu2dInstance>::Interrupt, Gpu2dInterruptHandler<GPU2D>> + 'static,
) {
    let driver = Gpu2d::new(peri, irq);
    critical_section::with(|cs| {
        *GPU2D_DRIVER.borrow(cs).borrow_mut() = Some(driver);
    });
    unsafe {
        hgpu2d.instance = INITIALIZED;
    }
}

/// # Safety
/// Called only by `nema-gfx-hal`'s C code with `&hgpu2d`.
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_ReadRegister(_handle: *mut Gpu2dHandleTypeDef, reg: u32) -> u32 {
    let base = embassy_stm32::pac::GPU2D.as_ptr() as *const u8;
    unsafe { core::ptr::read_volatile(base.add(reg as usize) as *const u32) }
}

/// # Safety
/// Called only by `nema-gfx-hal`'s C code with `&hgpu2d`.
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_WriteRegister(_handle: *mut Gpu2dHandleTypeDef, reg: u32, value: u32) {
    let base = embassy_stm32::pac::GPU2D.as_ptr() as *mut u8;
    unsafe { core::ptr::write_volatile(base.add(reg as usize) as *mut u32, value) }
}

/// # Safety
/// Called only by `nema-gfx-hal`'s C code (`nema_wait_irq`) with `&hgpu2d`.
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_PollCompletion(handle: *mut Gpu2dHandleTypeDef) {
    critical_section::with(|cs| {
        let mut slot = GPU2D_DRIVER.borrow(cs).borrow_mut();
        let Some(driver) = slot.as_mut() else {
            return;
        };

        if driver.command_list_complete() {
            let id = driver.last_command_list_id();
            driver.clear_command_list_complete();
            unsafe { HAL_GPU2D_CommandListCpltCallback(handle, id) };
        }

        // TODO: surface hardware SystemError through `crate::Error` instead
        // of silently draining it.
        let _ = driver.take_error();
    });
}
