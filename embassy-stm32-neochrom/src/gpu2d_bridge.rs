//! Real GPU2D hardware bridge.
//!
//! Implements the C ABI the platform HAL ([`crate::platform`]) expects,
//! backed by [`embassy_stm32::gpu2d::Gpu2d`], instead of the CI-only stub in
//! [`crate::gpu2d_stub`].
//!
//! `embassy_stm32::gpu2d::Gpu2d` owns the GPU2D error interrupt and an async
//! `wait_command_list_complete()`, but NemaGFX's own C code
//! (`nema_wait_irq`/`nema_wait_irq_cl`) is synchronous — it can't `.await`.
//! [`error_hook`] takes the interrupt over entirely (see
//! [`embassy_stm32::gpu2d::set_error_hook`]), so this module also owns
//! waking [`crate::command::CommandList`]'s async submission path
//! ([`poll_wait`]/[`command_list_complete`]/[`complete_command_list`]), and
//! bridges the synchronous C polling loop through [`HAL_GPU2D_PollCompletion`],
//! which `nema_wait_irq`/`nema_wait_irq_cl`'s busy-wait loop calls directly.
//! This works whether or not the real interrupt fires in between polls —
//! `command_list_complete()` reads the level-triggered flag directly.
//!
//! The error interrupt additionally carries the GPU2D cache-coherency
//! handshake. When NemaGFX's vector-rendering path needs the CPU to drop or
//! refresh its cache, it asserts a *hold*: the command-list processor halts,
//! raises the error interrupt, and stays halted until the CPU performs the
//! maintenance and deasserts the hold. [`service_error`] implements the CPU
//! half of that handshake. It runs from both the interrupt hook and the poll
//! loop, so a hold is serviced whether the interrupt fires or the CPU is
//! already spinning in `nema_wait_irq_cl`.

use core::cell::RefCell;
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Waker};

use critical_section::Mutex;
use embassy_stm32::Peri;
use embassy_stm32::gpu2d::{Gpu2d, Instance as Gpu2dInstance, InterruptHandler as Gpu2dInterruptHandler};
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::peripherals::GPU2D;

use crate::ffi::nema_gfx::nema_ext_hold_deassert_imm;

static GPU2D_DRIVER: Mutex<RefCell<Option<Gpu2d<'static, GPU2D>>>> = Mutex::new(RefCell::new(None));

/// Latched GPU2D peripheral error, see [`take_hardware_error`].
static HARDWARE_ERROR: AtomicBool = AtomicBool::new(false);

/// Waker for [`crate::command::CommandList`]'s async submission path.
static WAKER: Mutex<RefCell<Option<Waker>>> = Mutex::new(RefCell::new(None));

/// Take the latched GPU2D peripheral error, clearing it.
///
/// The cache-hold handshake and genuine errors share the error interrupt, and
/// the handshake is transparent to callers, so this exposes the error case that
/// `service_error` clears out of the status register.
pub fn take_hardware_error() -> bool {
    HARDWARE_ERROR.swap(false, Ordering::AcqRel)
}

/// C ABI mirror of STM32Cube's `GPU2D_HandleTypeDef`.
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

/********** System interrupt / cache-hold handshake **********/

/// `GPU2D_SYS_INTERRUPT`, which the PAC models only up to bit 0 (`er`) while
/// bits 2 and 3 carry the GPU-initiated cache-hold requests. Accessed raw, the
/// same way `HAL_GPU2D_ReadRegister`/`WriteRegister` already do.
const SYS_INTERRUPT: usize = 0xFF8;

/// A genuine peripheral error was signalled.
const SYS_INTERRUPT_ERR: u32 = 1 << 0;
/// Hold 2: the GPU needs the CPU to stop caching while it works.
const SYS_INTERRUPT_HOLD_DISABLE_CACHE: u32 = 1 << 2;
/// Hold 3: the GPU has written memory the CPU may still have cached.
const SYS_INTERRUPT_HOLD_INVALIDATE_CACHE: u32 = 1 << 3;

#[inline]
fn sys_interrupt_reg() -> *mut u32 {
    unsafe { (embassy_stm32::pac::GPU2D.as_ptr() as *mut u8).add(SYS_INTERRUPT) as *mut u32 }
}

/// Read and clear `SYS_INTERRUPT`, servicing any cache-hold request.
///
/// Returns the raw bits that were pending.
///
/// The register is write-1-to-clear, and this runs from both the error
/// interrupt and the `nema_wait_irq` poll loop, so the read/modify is done in a
/// critical section. The cache maintenance itself happens outside it: an
/// invalidation blocks until the hardware reports completion.
fn service_error() -> u32 {
    let pending = critical_section::with(|_| unsafe {
        let reg = sys_interrupt_reg();
        let val = core::ptr::read_volatile(reg);
        if val != 0 {
            core::ptr::write_volatile(reg, val);
        }
        val
    });

    if pending & SYS_INTERRUPT_ERR != 0 {
        // A real peripheral error. Latch it for `take_hardware_error`; the
        // command-list path only sees "the IRQ fired".
        HARDWARE_ERROR.store(true, Ordering::Release);
    }

    if pending & SYS_INTERRUPT_HOLD_DISABLE_CACHE != 0 {
        // Hold 2. The ICACHE only exists on N6/U5; H7RS has GPU2D but no
        // ICACHE block, where there is nothing to disable.
        #[cfg(any(feature = "n6", feature = "u5"))]
        embassy_stm32::icache::disable_global();

        // Release the command-list processor, which is halted on the hold.
        unsafe { nema_ext_hold_deassert_imm(2) };
    }

    if pending & SYS_INTERRUPT_HOLD_INVALIDATE_CACHE != 0 {
        // Hold 3.
        #[cfg(any(feature = "n6", feature = "u5"))]
        {
            embassy_stm32::icache::enable_global();
            embassy_stm32::icache::invalidate_global();
        }

        unsafe { nema_ext_hold_deassert_imm(3) };
    }

    pending
}

/// Returns whether the command-list-complete flag is currently set.
pub fn command_list_complete() -> bool {
    critical_section::with(|cs| {
        GPU2D_DRIVER
            .borrow(cs)
            .borrow()
            .as_ref()
            .is_some_and(Gpu2d::command_list_complete)
    })
}

/// Clear the command-list-complete flag and notify NemaGFX.
pub fn complete_command_list() {
    critical_section::with(|cs| {
        let mut slot = GPU2D_DRIVER.borrow(cs).borrow_mut();
        let Some(driver) = slot.as_mut() else {
            return;
        };

        if driver.command_list_complete() {
            let id = driver.last_command_list_id();
            driver.clear_command_list_complete();
            unsafe { HAL_GPU2D_CommandListCpltCallback(core::ptr::addr_of_mut!(hgpu2d), id) };
        }
    });
}

/// Register the async waker used by [`poll_wait`].
pub fn poll_wait(cx: &Context<'_>) {
    critical_section::with(|cs| {
        *WAKER.borrow(cs).borrow_mut() = Some(cx.waker().clone());
    });

    if command_list_complete() {
        wake();
    }
}

fn wake() {
    critical_section::with(|cs| {
        if let Some(waker) = WAKER.borrow(cs).borrow_mut().take() {
            waker.wake();
        }
    });
}

/// `fn()` adapter so [`service_error`] can be handed to `gpu2d::set_error_hook`.
///
/// Also wakes the async submission path: with a hook installed,
/// `embassy_stm32::gpu2d`'s own waker for `wait_command_list_complete()` never
/// fires (see that module's `InterruptHandler`), so this module must notice
/// command-list completions itself.
fn error_hook() {
    let _ = service_error();
    if command_list_complete() {
        wake();
    }
}

/// C ABI entry point for the GPU2D platform HAL.
///
/// # Safety
/// Called only from [`crate::platform`].
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_ErrorCallback(_handle: *mut Gpu2dHandleTypeDef) {
    let _ = service_error();
}

/********** C ABI **********/

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

    // Route the error interrupt to the cache-hold handshake. Installing a hook
    // also stops `InterruptHandler` masking the interrupt, which it must not:
    // the GPU asserts a hold per command list that needs one.
    embassy_stm32::gpu2d::set_error_hook(Some(error_hook));
}

/// # Safety
/// Called only by [`crate::platform`].
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_ReadRegister(_handle: *mut Gpu2dHandleTypeDef, reg: u32) -> u32 {
    let base = embassy_stm32::pac::GPU2D.as_ptr() as *const u8;
    unsafe { core::ptr::read_volatile(base.add(reg as usize) as *const u32) }
}

/// # Safety
/// Called only by [`crate::platform`].
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_WriteRegister(_handle: *mut Gpu2dHandleTypeDef, reg: u32, value: u32) {
    let base = embassy_stm32::pac::GPU2D.as_ptr() as *mut u8;
    unsafe { core::ptr::write_volatile(base.add(reg as usize) as *mut u32, value) }
}

/// # Safety
/// Called only by [`crate::platform`] (`nema_wait_irq`).
#[unsafe(no_mangle)]
unsafe extern "C" fn HAL_GPU2D_PollCompletion(handle: *mut Gpu2dHandleTypeDef) {
    // Service holds first: the GPU is halted until the CPU reacts, and the
    // interrupt may not have fired yet. Errors are latched for
    // `take_hardware_error`; the driver's `take_error()` is not used here since
    // this clears the whole status register.
    let _ = service_error();

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
    });
}
