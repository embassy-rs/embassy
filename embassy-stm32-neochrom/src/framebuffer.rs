//! Framebuffer storage for NeoChrom rendering targets.

use aligned::{A32, Aligned};

use crate::color::{ColorFormat, Rgba8888};

/// Common surface interface for NemaGFX destination/source textures.
pub trait GpuSurface {
    /// Physical base address passed to NemaGFX bind calls.
    fn phys_addr(&self) -> usize;
    /// Width in pixels.
    fn width(&self) -> u32;
    /// Height in pixels.
    fn height(&self) -> u32;
    /// NemaGFX pixel format constant.
    fn format(&self) -> ColorFormat;
    /// Row stride in bytes, or `-1` for tightly packed rows.
    fn stride(&self) -> i32 {
        -1
    }
}

/// RGBA8888 framebuffer suitable as a NemaGFX destination texture.
///
/// The third const generic `N` must equal `W * H` (number of pixels).
pub struct FrameBuffer<const W: u32, const H: u32, const N: usize> {
    pixels: Aligned<A32, [u32; N]>,
}

impl<const W: u32, const H: u32, const N: usize> FrameBuffer<W, H, N> {
    /// Create a zero-initialized framebuffer.
    pub const fn new() -> Self {
        Self {
            pixels: Aligned([0; N]),
        }
    }
}

impl<const W: u32, const H: u32, const N: usize> GpuSurface for FrameBuffer<W, H, N> {
    fn phys_addr(&self) -> usize {
        self.pixels.as_ptr() as usize
    }

    fn width(&self) -> u32 {
        W
    }

    fn height(&self) -> u32 {
        H
    }

    fn format(&self) -> ColorFormat {
        ColorFormat::Rgba8888
    }
}

impl<const W: u32, const H: u32, const N: usize> FrameBuffer<W, H, N> {
    /// Framebuffer width in pixels.
    #[inline]
    pub const fn width(&self) -> u32 {
        W
    }

    /// Framebuffer height in pixels.
    #[inline]
    pub const fn height(&self) -> u32 {
        H
    }

    /// Physical base address for [`crate::ffi::nema_gfx::nema_bind_dst_tex`].
    #[inline]
    pub fn phys_addr(&self) -> usize {
        GpuSurface::phys_addr(self)
    }

    /// Mutable view of pixel data (native-endian RGBA8888 words).
    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut *self.pixels
    }

    /// Fill the CPU-side buffer without using the GPU (debug / fallback).
    pub fn fill_cpu(&mut self, color: Rgba8888) {
        self.pixels_mut().fill(color.bits());
    }
}

impl<const W: u32, const H: u32, const N: usize> Default for FrameBuffer<W, H, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Framebuffer backed by an existing memory region (e.g. AXISRAM used by LTDC).
///
/// Use this when the render target already lives at a fixed address and may use
/// a format other than RGBA8888, such as RGB565 scan-out buffers.
#[derive(Debug, Clone, Copy)]
pub struct ExternalFrameBuffer {
    addr: usize,
    width: u32,
    height: u32,
    format: ColorFormat,
    stride: i32,
}

impl ExternalFrameBuffer {
    /// Bind an existing RGB565 buffer (typical LTDC layer framebuffer).
    pub const fn rgb565(addr: usize, width: u32, height: u32) -> Self {
        Self {
            addr,
            width,
            height,
            format: ColorFormat::Rgb565,
            stride: -1,
        }
    }

    /// Bind an existing buffer with an explicit NemaGFX format.
    pub const fn new(addr: usize, width: u32, height: u32, format: ColorFormat) -> Self {
        Self {
            addr,
            width,
            height,
            format,
            stride: -1,
        }
    }

    /// Override the row stride passed to NemaGFX (`-1` = tightly packed).
    pub const fn with_stride(self, stride: i32) -> Self {
        Self { stride, ..self }
    }
}

impl GpuSurface for ExternalFrameBuffer {
    fn phys_addr(&self) -> usize {
        self.addr
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn format(&self) -> ColorFormat {
        self.format
    }

    fn stride(&self) -> i32 {
        self.stride
    }
}
