//! C ABI platform hooks required by the prebuilt NemaGFX library.
//!
//! Replaces the former `nema_hal_baremetal.c`; the bodies mirror ST's
//! STM32N6570-DK bare-metal template, with Cube's GPU2D calls routed through
//! the crate's own GPU2D bridge or stub.

#![allow(dead_code)]

use core::ffi::{c_int, c_void};
use core::ptr;
use core::sync::atomic::{AtomicI32, Ordering};

use crate::ffi::nema_gfx as nema;

/// GPU2D breakpoint register, polled by `nema_wait_irq_brk()`.
const GPU2D_BREAKPOINT: u32 = 0x0000_0080;

/// Ring-buffer size handed to `nema_buffer_create()` in [`nema_sys_init`].
const RING_SIZE: c_int = 1024;

/// Last completed command-list id, updated from the GPU2D completion callback.
static LAST_CL_ID: AtomicI32 = AtomicI32::new(-1);

#[allow(non_upper_case_globals)]
static mut RING_BUFFER: nema::nema_ringbuffer_t = nema::nema_ringbuffer_t {
    bo: nema::nema_buffer_t {
        size: 0,
        fd: 0,
        base_virt: ptr::null_mut(),
        base_phys: 0,
    },
    offset: 0,
    last_submission_id: 0,
};

unsafe extern "C" {
    fn HAL_GPU2D_ReadRegister(handle: *mut c_void, reg: u32) -> u32;
    fn HAL_GPU2D_WriteRegister(handle: *mut c_void, reg: u32, value: u32);
    fn HAL_GPU2D_PollCompletion(handle: *mut c_void);
}

/// Host builds have no bump allocator (it would hijack the host allocator);
/// use libc's instead.
#[cfg(not(target_os = "none"))]
unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[cfg(target_os = "none")]
unsafe fn heap_alloc(size: usize) -> *mut c_void {
    crate::bump_alloc::malloc(size)
}

#[cfg(not(target_os = "none"))]
unsafe fn heap_alloc(size: usize) -> *mut c_void {
    malloc(size)
}

#[cfg(target_os = "none")]
unsafe fn heap_free(ptr: *mut c_void) {
    crate::bump_alloc::free(ptr)
}

#[cfg(not(target_os = "none"))]
unsafe fn heap_free(ptr: *mut c_void) {
    free(ptr)
}

/// Initialize the platform: allocate and map the command ring buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_sys_init() -> i32 {
    RING_BUFFER.bo = nema_buffer_create(RING_SIZE);
    let _ = nema_buffer_map(&raw mut RING_BUFFER.bo);

    let ret = nema::nema_rb_init(&raw mut RING_BUFFER, 1);
    if ret < 0 {
        return ret;
    }

    LAST_CL_ID.store(0, Ordering::Relaxed);
    0
}

/// Wait for the GPU to signal an interrupt.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_wait_irq() -> c_int {
    HAL_GPU2D_PollCompletion(ptr::null_mut());
    0
}

/// Block until command list `cl_id` has completed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_wait_irq_cl(cl_id: c_int) -> c_int {
    while LAST_CL_ID.load(Ordering::Relaxed) < cl_id {
        let _ = nema_wait_irq();
    }
    0
}

/// Block until a breakpoint is hit.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_wait_irq_brk(_brk_id: c_int) -> c_int {
    while nema_reg_read(GPU2D_BREAKPOINT) == 0 {
        let _ = nema_wait_irq();
    }
    0
}

/// Read a GPU2D register.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_reg_read(reg: u32) -> u32 {
    HAL_GPU2D_ReadRegister(ptr::null_mut(), reg)
}

/// Write a GPU2D register.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_reg_write(reg: u32, value: u32) {
    HAL_GPU2D_WriteRegister(ptr::null_mut(), reg, value)
}

/// Allocate a buffer the GPU can address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_create(size: c_int) -> nema::nema_buffer_t {
    let base_virt = heap_alloc(size as usize);

    nema::nema_buffer_t {
        size,
        fd: 0,
        base_virt,
        base_phys: base_virt as usize,
    }
}

/// Allocate a buffer from a specific memory pool (a single pool is provided).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_create_pool(_pool: c_int, size: c_int) -> nema::nema_buffer_t {
    nema_buffer_create(size)
}

/// Map a buffer, returning its virtual address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_map(bo: *mut nema::nema_buffer_t) -> *mut c_void {
    (*bo).base_virt
}

/// Unmap a buffer (no-op: buffers are permanently mapped here).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_unmap(_bo: *mut nema::nema_buffer_t) {}

/// Free a buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_destroy(bo: *mut nema::nema_buffer_t) {
    heap_free((*bo).base_virt);
    (*bo).base_virt = ptr::null_mut();
    (*bo).base_phys = 0;
}

/// Physical (GPU-visible) base address of a buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_phys(bo: *mut nema::nema_buffer_t) -> usize {
    (*bo).base_phys
}

/// Write back and invalidate a buffer's cache lines before the GPU reads them.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_buffer_flush(bo: *mut nema::nema_buffer_t) {
    if (*bo).base_virt.is_null() || (*bo).size <= 0 {
        return;
    }

    // The GPU may both read and write these buffers, so the cache must be
    // written back *and* invalidated; a clean-only flush leaves stale lines.
    crate::hal::nema_gfx_hal_dcache_clean_invalidate((*bo).base_virt, (*bo).size as u32);
}

/// Cache-hold hook: ask the GPU to stop the CPU caching while it works.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn platform_disable_cache() {
    nema::nema_ext_hold_assert(2, 1);
}

/// Cache-hold hook: tell the GPU the CPU cache has been refreshed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn platform_invalidate_cache() {
    nema::nema_ext_hold_assert(3, 1);
}

/// Host memory free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_host_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        heap_free(ptr);
    }
}

/// Host memory allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_host_malloc(size: usize) -> *mut c_void {
    heap_alloc(size)
}

/// Mutex lock (single-threaded: always succeeds).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_mutex_lock(_mutex_id: c_int) -> c_int {
    0
}

/// Mutex unlock (single-threaded: always succeeds).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_mutex_unlock(_mutex_id: c_int) -> c_int {
    0
}

/// GPU2D command-list-complete callback, called by the GPU2D bridge/stub.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_GPU2D_CommandListCpltCallback(_handle: *mut c_void, cmd_list_id: u32) {
    LAST_CL_ID.store(cmd_list_id as i32, Ordering::Relaxed);
}
