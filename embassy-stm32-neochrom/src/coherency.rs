//! CPU cache maintenance for GPU-accessible framebuffers.

use crate::color::ColorFormat;
use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_stride_size, nema_texture_size};
use crate::framebuffer::GpuSurface;

/// Snapshot of a [`GpuSurface`] for post-GPU cache maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSyncInfo {
    pub(crate) phys_addr: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: ColorFormat,
    pub(crate) stride: i32,
}

impl SurfaceSyncInfo {
    /// Capture sync metadata from any GPU surface.
    pub fn from_surface(surface: &impl GpuSurface) -> Self {
        Self {
            phys_addr: surface.phys_addr(),
            width: surface.width(),
            height: surface.height(),
            format: surface.format(),
            stride: surface.stride(),
        }
    }

    fn byte_len(self) -> usize {
        let w = self.width as i32;
        let h = self.height as i32;
        if w <= 0 || h <= 0 {
            return 0;
        }

        let fmt = self.format.nema_format();
        if self.stride >= 0 {
            return (self.stride * h) as usize;
        }

        unsafe {
            let size = nema_texture_size(fmt, NEMA_TEX_BORDER as u8, w, h);
            if size > 0 {
                return size as usize;
            }

            let stride = nema_stride_size(fmt, NEMA_TEX_BORDER as u8, w);
            (stride * h) as usize
        }
    }
}

/// Prepare CPU-written surfaces before the GPU reads them.
pub fn sync_before_gpu(surfaces: &[SurfaceSyncInfo]) {
    for surface in surfaces {
        clean_dcache(*surface);
    }
}

/// Ensure CPU/LTDC observers see GPU-written surface contents.
pub fn sync_after_gpu(surfaces: &[SurfaceSyncInfo]) {
    #[cfg(any(feature = "n6", feature = "u5"))]
    embassy_stm32::icache::invalidate();

    for surface in surfaces {
        clean_invalidate_dcache(*surface);
    }
}

fn clean_dcache(surface: SurfaceSyncInfo) {
    let len = surface.byte_len();
    if len == 0 {
        return;
    }
    dcache_maintain(surface.phys_addr, len, MaintainOp::Clean);
}

fn clean_invalidate_dcache(surface: SurfaceSyncInfo) {
    let len = surface.byte_len();
    if len == 0 {
        return;
    }
    dcache_maintain(surface.phys_addr, len, MaintainOp::CleanInvalidate);
}

#[derive(Clone, Copy)]
enum MaintainOp {
    Clean,
    CleanInvalidate,
}

fn dcache_maintain(addr: usize, len: usize, op: MaintainOp) {
    #[cfg(all(any(feature = "n6", feature = "u5", feature = "h7rs"), target_arch = "arm"))]
    {
        unsafe extern "C" {
            fn SCB_CleanDCache_by_Addr(addr: *mut u32, dsize: i32);
            fn SCB_CleanInvalidateDCache_by_Addr(addr: *mut u32, dsize: i32);
        }

        const LINE: usize = 32;
        let start = addr & !(LINE - 1);
        let end = (addr + len + LINE - 1) & !(LINE - 1);
        let mut ptr = start;
        while ptr < end {
            let chunk = core::cmp::min(LINE, end - ptr);
            unsafe {
                match op {
                    MaintainOp::Clean => SCB_CleanDCache_by_Addr(ptr as *mut u32, chunk as i32),
                    MaintainOp::CleanInvalidate => SCB_CleanInvalidateDCache_by_Addr(ptr as *mut u32, chunk as i32),
                }
            }
            ptr += LINE;
        }
    }

    #[cfg(not(all(any(feature = "n6", feature = "u5", feature = "h7rs"), target_arch = "arm")))]
    let _ = (addr, len, op);
}
