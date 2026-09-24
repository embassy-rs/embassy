//! Platform HAL for [NemaGFX](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/Middleware/NemaGFX).
//!
//! [`crate::platform`] implements the C ABI hooks the prebuilt NemaGFX library
//! calls; this module exposes the safe-ish Rust surface and the blend helpers
//! that are `static inline` in the NemaGFX headers.
//!
//! # Hardware
//!
//! Before calling [`stm32_bindings::nema_gfx::nema_init`], initialize the GPU2D
//! peripheral clocks and bind the peripheral (see `crate::gpu2d_bridge`). The
//! default `stub-gpu2d` feature links a does-nothing GPU2D for CI/link tests.
//!
//! This module is re-exported at the crate root as `nema_gfx_hal`.

use core::ffi::c_void;

/// Clean and invalidate the D-cache for `addr .. addr + size`.
///
/// Called from the C-ABI `nema_buffer_flush()`. A clean-only flush is not
/// sufficient: the GPU may have overwritten the buffer, so the CPU's cached
/// lines have to be dropped as well, otherwise a later CPU read hits stale data.
///
/// `size == 0` and null pointers are ignored.
#[cfg(target_arch = "arm")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nema_gfx_hal_dcache_clean_invalidate(addr: *mut c_void, size: u32) {
    if addr.is_null() || size == 0 {
        return;
    }

    // SAFETY: `SCB` is a zero-sized handle to a fixed register block; taking it
    // by `steal()` is the usual way to reach it outside `embassy-stm32`.
    let mut scb = unsafe { cortex_m::peripheral::Peripherals::steal().SCB };
    scb.clean_invalidate_dcache_by_address(addr as usize, size as usize);
}

/// Host builds never run the platform HAL, so no caller exists; the symbol is
/// defined only so the crate links standalone.
#[cfg(not(target_arch = "arm"))]
#[unsafe(no_mangle)]
pub extern "C" fn nema_gfx_hal_dcache_clean_invalidate(_addr: *mut c_void, _size: u32) {}

/// Initialize the GPU2D handle using the stub HAL.
///
/// On hardware, bind the real peripheral via `crate::gpu2d_bridge` instead.
#[cfg(feature = "stub-gpu2d")]
pub fn gpu2d_init_stub() -> Result<(), ()> {
    unsafe extern "C" {
        fn nema_gfx_hal_gpu2d_init() -> i32;
    }

    unsafe {
        if nema_gfx_hal_gpu2d_init() == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Blend-mode helpers that are `static inline` in the NemaGFX headers.
///
/// bindgen cannot emit static-inline functions, so these call the real
/// `nema_set_blend()` with the texture units from the header. Do not reimplement
/// the unit selection by hand: `set_blit` and `set_fill` differ only in which
/// texture unit carries the source (`NEMA_TEX1` vs `NEMA_NOTEX`), and passing the
/// fill pattern to a blit leaves the source unit unbound -- the GPU then blends
/// against stale texture state, which renders as uniformly-coloured output rather
/// than as an error.
pub mod blend {
    use crate::ffi::nema_gfx as nema;

    /// Blend mode for filling: source is texture unit 0.
    pub fn set_fill(mode: u32) {
        unsafe {
            nema::nema_set_blend(
                mode,
                nema::nema_tex_t_NEMA_TEX0,
                nema::nema_tex_t_NEMA_NOTEX,
                nema::nema_tex_t_NEMA_NOTEX,
            )
        }
    }

    /// Fill with a composing (third) texture.
    pub fn set_fill_compose(mode: u32) {
        unsafe {
            nema::nema_set_blend(
                mode,
                nema::nema_tex_t_NEMA_TEX0,
                nema::nema_tex_t_NEMA_NOTEX,
                nema::nema_tex_t_NEMA_TEX2,
            )
        }
    }

    /// Blend mode for blitting: source is texture unit 0, foreground **texture unit 1**.
    pub fn set_blit(mode: u32) {
        unsafe {
            nema::nema_set_blend(
                mode,
                nema::nema_tex_t_NEMA_TEX0,
                nema::nema_tex_t_NEMA_TEX1,
                nema::nema_tex_t_NEMA_NOTEX,
            )
        }
    }

    /// Blit with a composing (third) texture.
    pub fn set_blit_compose(mode: u32) {
        unsafe {
            nema::nema_set_blend(
                mode,
                nema::nema_tex_t_NEMA_TEX0,
                nema::nema_tex_t_NEMA_TEX1,
                nema::nema_tex_t_NEMA_TEX2,
            )
        }
    }

    /// Compose an RGBA blend mode from separate source/destination factors.
    pub fn mode(src_bf: u32, dst_bf: u32, blops: u32) -> u32 {
        nema::NEMA_BLOP_MASK & blops | src_bf | (dst_bf << 8)
    }
}
