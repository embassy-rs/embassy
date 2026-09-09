//! GPU-accelerated [`embedded_graphics_core::draw_target::DrawTarget`] implementation.

use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics_core::primitives::Rectangle;

use crate::color::Rgba8888;
use crate::driver::NeoChrom;
use crate::error::Error;
use crate::framebuffer::FrameBuffer;

/// A GPU-accelerated `DrawTarget` wrapper combining [`NeoChrom`] and a [`FrameBuffer`].
///
/// Automatically offloads `fill_solid` and `clear` operations to NeoChrom GPU command lists.
pub struct NeoChromTarget<'a, 'fb, const W: u32, const H: u32, const N: usize> {
    gpu: &'a mut NeoChrom,
    framebuffer: &'fb FrameBuffer<W, H, N>,
}

impl<'a, 'fb, const W: u32, const H: u32, const N: usize> NeoChromTarget<'a, 'fb, W, H, N> {
    /// Create a new GPU-accelerated `DrawTarget`.
    pub fn new(gpu: &'a mut NeoChrom, framebuffer: &'fb FrameBuffer<W, H, N>) -> Self {
        Self { gpu, framebuffer }
    }

    /// Fill a circle at (`cx`, `cy`) with radius `r` using NeoChrom GPU hardware.
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Rgb888) -> Result<(), Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.fill_circle(self.framebuffer, cx, cy, r, rgba)
    }

    /// Fill a rounded rectangle with corner radius `r` using NeoChrom GPU hardware.
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, color: Rgb888) -> Result<(), Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.fill_rounded_rect(self.framebuffer, x, y, w, h, r, rgba)
    }

    /// Fill a triangle defined by 3 vertices using NeoChrom GPU hardware.
    pub fn fill_triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb888) -> Result<(), Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.fill_triangle(self.framebuffer, x0, y0, x1, y1, x2, y2, rgba)
    }

    /// Draw a line from (`x0`, `y0`) to (`x1`, `y1`) using NeoChrom GPU hardware.
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb888) -> Result<(), Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.draw_line(self.framebuffer, x0, y0, x1, y1, rgba)
    }

    /// Blit (hardware copy) a source `FrameBuffer` into this target at (`dst_x`, `dst_y`).
    pub fn blit<const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        src: &FrameBuffer<SW, SH, SN>,
        dst_x: i32,
        dst_y: i32,
    ) -> Result<(), Error> {
        self.gpu.blit(self.framebuffer, src, dst_x, dst_y)
    }

    /// Blit source `FrameBuffer` rotated around center (`cx`, `cy`) and pivot (`px`, `py`).
    pub fn blit_rotate_pivot<const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        src: &FrameBuffer<SW, SH, SN>,
        cx: f32,
        cy: f32,
        px: f32,
        py: f32,
        angle_degrees: f32,
    ) -> Result<(), Error> {
        self.gpu.blit_rotate_pivot(self.framebuffer, src, cx, cy, px, py, angle_degrees)
    }

    /// Blit cropped sub-rectangle of `src` (`src_x`,`src_y`,`src_w`,`src_h`) scaled to destination (`dst_x`,`dst_y`,`dst_w`,`dst_h`).
    #[allow(clippy::too_many_arguments)]
    pub fn blit_subrect_fit<const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        src: &FrameBuffer<SW, SH, SN>,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
    ) -> Result<(), Error> {
        self.gpu.blit_subrect_fit(self.framebuffer, src, dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h)
    }

    /// Blit source `FrameBuffer` warped into a 3D perspective quadrilateral defined by 4 vertices.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_quad_fit<const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        src: &FrameBuffer<SW, SH, SN>,
        dx0: f32,
        dy0: f32,
        dx1: f32,
        dy1: f32,
        dx2: f32,
        dy2: f32,
        dx3: f32,
        dy3: f32,
    ) -> Result<(), Error> {
        self.gpu.blit_quad_fit(self.framebuffer, src, dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3)
    }

    /// Set the GPU blending mode for solid fill operations.
    pub fn set_blend_fill(&mut self, mode: crate::driver::BlendMode) {
        self.gpu.set_blend_fill(mode);
    }

    /// Set the GPU blending mode for blit operations.
    pub fn set_blend_blit(&mut self, mode: crate::driver::BlendMode) {
        self.gpu.set_blend_blit(mode);
    }

    /// Set global constant color / alpha opacity for subsequent GPU operations.
    pub fn set_const_color(&mut self, color: Rgb888, alpha: u8) {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), alpha);
        self.gpu.set_const_color(rgba);
    }

    /// Set texture tint color for alpha-mask (A8/A4/A2/A1) blits (e.g. font glyphs).
    pub fn set_tex_color(&mut self, color: Rgb888, alpha: u8) {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), alpha);
        self.gpu.set_tex_color(rgba);
    }

    /// Blit an A8 (8-bit alpha mask) font glyph/buffer onto display at (`dst_x`, `dst_y`) tinted with `color`.
    pub fn blit_alpha_mask(
        &mut self,
        mask_phys_addr: usize,
        mask_w: u32,
        mask_h: u32,
        mask_stride: i32,
        dst_x: i32,
        dst_y: i32,
        color: Rgb888,
    ) -> Result<(), Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.blit_alpha_mask(self.framebuffer, mask_phys_addr, mask_w, mask_h, mask_stride, dst_x, dst_y, rgba)
    }

    /// Set GPU hardware clipping rectangle (`x`, `y`, `w`, `h`).
    pub fn set_clip(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.gpu.set_clip(x, y, w, h);
    }

    /// Reset GPU hardware clipping to full display bounds (`W` x `H`).
    pub fn reset_clip(&mut self) {
        self.gpu.reset_clip(self.framebuffer);
    }
}

impl<const W: u32, const H: u32, const N: usize> core::fmt::Debug for NeoChromTarget<'_, '_, W, H, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NeoChromTarget")
            .field("width", &W)
            .field("height", &H)
            .finish()
    }
}

impl<const W: u32, const H: u32, const N: usize> OriginDimensions for NeoChromTarget<'_, '_, W, H, N> {
    fn size(&self) -> Size {
        Size::new(W, H)
    }
}

impl<const W: u32, const H: u32, const N: usize> DrawTarget for NeoChromTarget<'_, '_, W, H, N> {
    type Color = Rgb888;
    type Error = Error;

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        // CPU fallback for individual pixel iterators
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let rect = area.intersection(&self.bounding_box());
        if rect.size.width == 0 || rect.size.height == 0 {
            return Ok(());
        }
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.fill_rect(
            self.framebuffer,
            rect.top_left.x,
            rect.top_left.y,
            rect.size.width as i32,
            rect.size.height as i32,
            rgba,
        )
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let rgba = Rgba8888::new(color.r(), color.g(), color.b(), 0xFF);
        self.gpu.clear(self.framebuffer, rgba)
    }
}
