//! Minimal 8-byte aligned bump allocator for NemaGFX bring-up.
//!
//! Suitable for bring-up only: `free()` is a no-op. It is compiled only for
//! bare-metal targets, so host builds use the system allocator instead (this
//! allocator would otherwise hijack `cargo test`'s allocator).

use core::ffi::c_void;

/// Pool size, matching ST's template.
const NEMA_BUMP_ALLOC_SIZE: usize = 128 * 1024;

/// The pool *must* be 8-byte aligned: NemaGFX rejects an unaligned command-list
/// buffer (`NEMA_ERR_INVALID_CL_ALIGMENT`), and with no command list bound every
/// drawing call then fails silently.
#[repr(align(8))]
struct BumpPool(#[allow(dead_code)] [u8; NEMA_BUMP_ALLOC_SIZE]);

static mut BUMP_POOL: BumpPool = BumpPool([0; NEMA_BUMP_ALLOC_SIZE]);
static mut BUMP_OFFSET: usize = 0;

/// Allocate `size` bytes, 8-byte aligned, from the static pool.
///
/// # Safety
/// Not thread-safe; NemaGFX only allocates from its single-threaded init path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let pool = (&raw mut BUMP_POOL).cast::<u8>();
    let mut offset = BUMP_OFFSET;

    // Round the returned pointer up to 8 bytes as well as the size: the
    // alignment an allocation receives must not depend on where the pool landed.
    let misalign = (pool.add(offset) as usize) & 7;
    if misalign != 0 {
        offset += 8 - misalign;
    }

    let aligned = (size + 7) & !7usize;
    if offset + aligned > NEMA_BUMP_ALLOC_SIZE {
        return core::ptr::null_mut();
    }

    let ptr = pool.add(offset).cast::<c_void>();
    BUMP_OFFSET = offset + aligned;
    ptr
}

/// No-op: the pool is never reclaimed.
///
/// # Safety
/// Always safe to call; the pointer is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(_ptr: *mut c_void) {}
