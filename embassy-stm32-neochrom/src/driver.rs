//! NeoChrom GPU driver using NemaGFX.

use core::future::Future;

use heapless::Vec;

use crate::coherency::SurfaceSyncInfo;
use crate::color::Rgba8888;
use crate::command::CommandList;
use crate::error::{Error, InitError};
use crate::ffi::nema_gfx::nema_init;
use crate::framebuffer::GpuSurface;

/// NemaGFX GPU blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum BlendMode {
    /// Clear mode (0).
    Clear = 0x0000,
    /// Source mode (Sa).
    Src = 0x0002,
    /// Standard alpha blending.
    Simple = 0x0201,
    /// Source over destination.
    SrcOver = 0x0202,
    /// Destination over source.
    DstOver = 0x0105,
    /// Additive blending.
    Add = 0x0102,
}

/// Texture filtering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TextureFilter {
    /// Point sampling (nearest neighbor).
    PointSample = 0,
    /// Bilinear filtering (smooth interpolation).
    Bilinear = 1,
}

/// Texture wrapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TextureWrap {
    /// Clamp to edge.
    Clamp = 0,
    /// Repeat texture (tiling).
    Repeat = 4,
    /// Border fill.
    Border = 8,
    /// Mirror texture.
    Mirror = 12,
}

struct FrameState {
    dst: SurfaceSyncInfo,
    surfaces: Vec<SurfaceSyncInfo, 8>,
}

/// NeoChrom GPU context.
///
/// Call [`NeoChrom::new`] once after GPU2D clocks are enabled. Use
/// [`NeoChrom::begin_frame`] / [`NeoChrom::end_frame`] (or [`NeoChrom::end_frame_async`])
/// to batch many draw calls into a single GPU command list.
pub struct NeoChrom {
    cmd: CommandList,
    frame: Option<FrameState>,
}

impl NeoChrom {
    /// Initialize NemaGFX and the platform HAL using the in-tree GPU2D stub.
    #[cfg(feature = "stub-gpu2d")]
    pub fn new() -> Result<Self, InitError> {
        nema_gfx_hal::gpu2d_init_stub().map_err(|_| InitError::Gpu2d)?;
        Self::finish_init()
    }

    /// Initialize NemaGFX using the real GPU2D peripheral.
    #[cfg(not(feature = "stub-gpu2d"))]
    pub fn new(
        peri: embassy_stm32::Peri<'static, embassy_stm32::peripherals::GPU2D>,
        irq: impl embassy_stm32::interrupt::typelevel::Binding<
            <embassy_stm32::peripherals::GPU2D as embassy_stm32::gpu2d::Instance>::Interrupt,
            embassy_stm32::gpu2d::InterruptHandler<embassy_stm32::peripherals::GPU2D>,
        > + 'static,
    ) -> Result<Self, InitError> {
        crate::gpu2d_bridge::init(peri, irq);
        Self::finish_init()
    }

    fn finish_init() -> Result<Self, InitError> {
        let status = unsafe { nema_init() };
        if status != 0 {
            return Err(InitError::NemaGfx);
        }

        unsafe {
            use crate::ffi::nema_gfx::{nema_ext_hold_enable, nema_ext_hold_irq_enable};
            nema_ext_hold_enable(2);
            nema_ext_hold_irq_enable(2);
            nema_ext_hold_enable(3);
            nema_ext_hold_irq_enable(3);
        }

        Ok(Self {
            cmd: CommandList::new(),
            frame: None,
        })
    }

    /// Returns `true` while a batched frame is open.
    pub fn is_frame_active(&self) -> bool {
        self.frame.is_some()
    }

    /// Begin a batched render pass targeting `dst`.
    ///
    /// Issue many draw/blit calls, then finish with [`Self::end_frame`] or
    /// [`Self::end_frame_async`].
    pub fn begin_frame(&mut self, dst: &impl GpuSurface) -> Result<(), Error> {
        if self.frame.is_some() {
            return Err(Error::FrameAlreadyActive);
        }

        self.cmd.bind();
        bind_dst(dst);
        let dst_info = SurfaceSyncInfo::from_surface(dst);
        self.frame = Some(FrameState {
            dst: dst_info,
            surfaces: Vec::new(),
        });
        self.track_surface(dst_info);
        Ok(())
    }

    /// Submit the open frame and block until the GPU finishes.
    pub fn end_frame(&mut self) -> Result<(), Error> {
        let surfaces = self.take_frame_surfaces()?;
        self.cmd.submit_and_wait(&surfaces)
    }

    /// Submit the open frame and await GPU completion without blocking the executor.
    pub fn end_frame_async(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        async move {
            let surfaces = self.take_frame_surfaces()?;
            self.cmd.submit_and_wait_async(&surfaces).await
        }
    }

    /// Clear the active frame destination to `color`.
    pub fn clear(&mut self, framebuffer: &impl GpuSurface, color: Rgba8888) -> Result<(), Error> {
        if self.frame.is_some() {
            self.clear_in_frame(color)
        } else {
            self.begin_frame(framebuffer)?;
            self.clear_in_frame(color)?;
            self.end_frame()
        }
    }

    /// Clear the active batched frame.
    pub fn clear_in_frame(&mut self, color: Rgba8888) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_clear;
            nema_clear(color.bits());
        });
        Ok(())
    }

    /// Fill a rectangle at (`x`, `y`) with dimensions `w`x`h`.
    pub fn fill_rect(
        &mut self,
        framebuffer: &impl GpuSurface,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.fill_rect_in_frame(x, y, w, h, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.fill_rect_in_frame(x, y, w, h, color)?;
            self.end_frame()
        }
    }

    /// Fill a rectangle in the active batched frame.
    pub fn fill_rect_in_frame(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba8888) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_fill_rect;
            nema_fill_rect(x, y, w, h, color.bits());
        });
        Ok(())
    }

    /// Draw a 1-pixel outline rectangle.
    pub fn draw_stroke_rect(
        &mut self,
        framebuffer: &impl GpuSurface,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_rect_in_frame(x, y, w, h, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_rect_in_frame(x, y, w, h, color)?;
            self.end_frame()
        }
    }

    /// Draw a 1-pixel outline rectangle in the active batched frame.
    pub fn draw_stroke_rect_in_frame(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba8888) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_rect;
            nema_draw_rect(x, y, w, h, color.bits());
        });
        Ok(())
    }

    /// Draw a stroked rectangle with anti-aliasing and explicit border width.
    pub fn draw_stroke_rect_aa(
        &mut self,
        framebuffer: &impl GpuSurface,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_rect_aa_in_frame(x, y, w, h, border_width, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_rect_aa_in_frame(x, y, w, h, border_width, color)?;
            self.end_frame()
        }
    }

    /// Draw a stroked anti-aliased rectangle in the active batched frame.
    pub fn draw_stroke_rect_aa_in_frame(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_rounded_rect_aa;
            nema_draw_rounded_rect_aa(x, y, w, h, 0.0, border_width, color.bits());
        });
        Ok(())
    }

    /// Draw a line from (`x0`, `y0`) to (`x1`, `y1`).
    pub fn draw_line(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_line_in_frame(x0, y0, x1, y1, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_line_in_frame(x0, y0, x1, y1, color)?;
            self.end_frame()
        }
    }

    /// Draw a line in the active batched frame.
    pub fn draw_line_in_frame(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba8888) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_line;
            nema_draw_line(x0, y0, x1, y1, color.bits());
        });
        Ok(())
    }

    /// Draw a stroked line with explicit width and anti-aliasing.
    pub fn draw_stroke_line_aa(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_line_aa_in_frame(x0, y0, x1, y1, width, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_line_aa_in_frame(x0, y0, x1, y1, width, color)?;
            self.end_frame()
        }
    }

    /// Draw a stroked anti-aliased line in the active batched frame.
    pub fn draw_stroke_line_aa_in_frame(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_line_aa;
            nema_draw_line_aa(x0, y0, x1, y1, width, color.bits());
        });
        Ok(())
    }

    /// Fill a circle at (`cx`, `cy`) with radius `r`.
    pub fn fill_circle(
        &mut self,
        framebuffer: &impl GpuSurface,
        cx: i32,
        cy: i32,
        r: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.fill_circle_in_frame(cx, cy, r, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.fill_circle_in_frame(cx, cy, r, color)?;
            self.end_frame()
        }
    }

    /// Fill a circle in the active batched frame.
    pub fn fill_circle_in_frame(&mut self, cx: i32, cy: i32, r: i32, color: Rgba8888) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_fill_circle;
            nema_fill_circle(cx, cy, r, color.bits());
        });
        Ok(())
    }

    /// Draw a stroked circle with explicit width and anti-aliasing.
    pub fn draw_stroke_circle_aa(
        &mut self,
        framebuffer: &impl GpuSurface,
        cx: f32,
        cy: f32,
        r: f32,
        width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_circle_aa_in_frame(cx, cy, r, width, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_circle_aa_in_frame(cx, cy, r, width, color)?;
            self.end_frame()
        }
    }

    /// Draw a stroked anti-aliased circle in the active batched frame.
    pub fn draw_stroke_circle_aa_in_frame(
        &mut self,
        cx: f32,
        cy: f32,
        r: f32,
        width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_circle_aa;
            nema_draw_circle_aa(cx, cy, r, width, color.bits());
        });
        Ok(())
    }

    /// Blit `src` into `dst` at (`dst_x`, `dst_y`).
    pub fn blit(&mut self, dst: &impl GpuSurface, src: &impl GpuSurface, dst_x: i32, dst_y: i32) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_in_frame(src, dst_x, dst_y)
        } else {
            self.begin_frame(dst)?;
            self.blit_in_frame(src, dst_x, dst_y)?;
            self.end_frame()
        }
    }

    /// Blit `src` into the active batched frame.
    pub fn blit_in_frame(&mut self, src: &impl GpuSurface, dst_x: i32, dst_y: i32) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit;
            nema_blit(dst_x, dst_y);
        });
        Ok(())
    }

    /// Fill a rounded rectangle with corner radius `r`.
    pub fn fill_rounded_rect(
        &mut self,
        framebuffer: &impl GpuSurface,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.fill_rounded_rect_in_frame(x, y, w, h, r, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.fill_rounded_rect_in_frame(x, y, w, h, r, color)?;
            self.end_frame()
        }
    }

    /// Fill a rounded rectangle in the active batched frame.
    pub fn fill_rounded_rect_in_frame(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_fill_rounded_rect;
            nema_fill_rounded_rect(x, y, w, h, r, color.bits());
        });
        Ok(())
    }

    /// Fill a triangle defined by 3 vertices.
    pub fn fill_triangle(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.fill_triangle_in_frame(x0, y0, x1, y1, x2, y2, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.fill_triangle_in_frame(x0, y0, x1, y1, x2, y2, color)?;
            self.end_frame()
        }
    }

    /// Fill a triangle in the active batched frame.
    pub fn fill_triangle_in_frame(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_fill_triangle;
            nema_fill_triangle(x0, y0, x1, y1, x2, y2, color.bits());
        });
        Ok(())
    }

    /// Draw a stroked triangle with explicit border width and anti-aliasing.
    pub fn draw_stroke_triangle_aa(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_triangle_aa_in_frame(x0, y0, x1, y1, x2, y2, border_width, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_triangle_aa_in_frame(x0, y0, x1, y1, x2, y2, border_width, color)?;
            self.end_frame()
        }
    }

    /// Draw a stroked anti-aliased triangle in the active batched frame.
    pub fn draw_stroke_triangle_aa_in_frame(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_triangle_aa;
            nema_draw_triangle_aa(x0, y0, x1, y1, x2, y2, border_width, color.bits());
        });
        Ok(())
    }

    /// Fill a quadrilateral defined by 4 vertices.
    pub fn fill_quad(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.fill_quad_in_frame(x0, y0, x1, y1, x2, y2, x3, y3, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.fill_quad_in_frame(x0, y0, x1, y1, x2, y2, x3, y3, color)?;
            self.end_frame()
        }
    }

    /// Fill a quadrilateral in the active batched frame.
    pub fn fill_quad_in_frame(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_fill_quad;
            nema_fill_quad(x0, y0, x1, y1, x2, y2, x3, y3, color.bits());
        });
        Ok(())
    }

    /// Draw a stroked convex quadrilateral with explicit border width and anti-aliasing.
    pub fn draw_stroke_quad_aa(
        &mut self,
        framebuffer: &impl GpuSurface,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.draw_stroke_quad_aa_in_frame(x0, y0, x1, y1, x2, y2, x3, y3, border_width, color)
        } else {
            self.begin_frame(framebuffer)?;
            self.draw_stroke_quad_aa_in_frame(x0, y0, x1, y1, x2, y2, x3, y3, border_width, color)?;
            self.end_frame()
        }
    }

    /// Draw a stroked anti-aliased quadrilateral in the active batched frame.
    pub fn draw_stroke_quad_aa_in_frame(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        border_width: f32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        self.ensure_frame()?;
        record_dst_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_draw_quad_aa;
            nema_draw_quad_aa(x0, y0, x1, y1, x2, y2, x3, y3, border_width, color.bits());
        });
        Ok(())
    }

    /// Blit `src` scaled to `dst_w` x `dst_h` at (`dst_x`, `dst_y`).
    pub fn blit_rect_fit(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_rect_fit_in_frame(src, dst_x, dst_y, dst_w, dst_h)
        } else {
            self.begin_frame(dst)?;
            self.blit_rect_fit_in_frame(src, dst_x, dst_y, dst_w, dst_h)?;
            self.end_frame()
        }
    }

    /// Scaled blit into the active batched frame.
    pub fn blit_rect_fit_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_rect_fit;
            nema_blit_rect_fit(dst_x, dst_y, dst_w, dst_h);
        });
        Ok(())
    }

    /// Blit `src` rotated by `angle_degrees` at (`dst_x`, `dst_y`).
    pub fn blit_rotate(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        angle_degrees: u32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_rotate_in_frame(src, dst_x, dst_y, angle_degrees)
        } else {
            self.begin_frame(dst)?;
            self.blit_rotate_in_frame(src, dst_x, dst_y, angle_degrees)?;
            self.end_frame()
        }
    }

    /// Rotated blit into the active batched frame.
    pub fn blit_rotate_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        angle_degrees: u32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_rotate;
            nema_blit_rotate(dst_x, dst_y, angle_degrees);
        });
        Ok(())
    }

    /// Blit `src` with rotation around center (`cx`, `cy`) and pivot (`px`, `py`).
    pub fn blit_rotate_pivot(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        cx: f32,
        cy: f32,
        px: f32,
        py: f32,
        angle_degrees: f32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_rotate_pivot_in_frame(src, cx, cy, px, py, angle_degrees)
        } else {
            self.begin_frame(dst)?;
            self.blit_rotate_pivot_in_frame(src, cx, cy, px, py, angle_degrees)?;
            self.end_frame()
        }
    }

    /// Pivot rotation blit into the active batched frame.
    pub fn blit_rotate_pivot_in_frame(
        &mut self,
        src: &impl GpuSurface,
        cx: f32,
        cy: f32,
        px: f32,
        py: f32,
        angle_degrees: f32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_rotate_pivot;
            nema_blit_rotate_pivot(cx, cy, px, py, angle_degrees);
        });
        Ok(())
    }

    /// Blit a sub-rectangle of `src`, scaled to a destination rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_subrect_fit(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_subrect_fit_in_frame(src, dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h)
        } else {
            self.begin_frame(dst)?;
            self.blit_subrect_fit_in_frame(src, dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h)?;
            self.end_frame()
        }
    }

    /// Sub-rectangle scaled blit into the active batched frame.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_subrect_fit_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_subrect_fit;
            nema_blit_subrect_fit(dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h);
        });
        Ok(())
    }

    /// Blit `src` warped into a destination quadrilateral.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_quad_fit(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        dx1: f32,
        dy1: f32,
        dx2: f32,
        dy2: f32,
        dx3: f32,
        dy3: f32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_quad_fit_in_frame(src, dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3)
        } else {
            self.begin_frame(dst)?;
            self.blit_quad_fit_in_frame(src, dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3)?;
            self.end_frame()
        }
    }

    /// Quadrilateral warp blit into the active batched frame.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_quad_fit_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        dx1: f32,
        dy1: f32,
        dx2: f32,
        dy2: f32,
        dx3: f32,
        dy3: f32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_quad_fit;
            nema_blit_quad_fit(dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3);
        });
        Ok(())
    }

    /// Blit `src` mapped to a destination triangle using texture corner indices.
    pub fn blit_tri_fit(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        v0: i32,
        dx1: f32,
        dy1: f32,
        v1: i32,
        dx2: f32,
        dy2: f32,
        v2: i32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_tri_fit_in_frame(src, dx0, dy0, v0, dx1, dy1, v1, dx2, dy2, v2)
        } else {
            self.begin_frame(dst)?;
            self.blit_tri_fit_in_frame(src, dx0, dy0, v0, dx1, dy1, v1, dx2, dy2, v2)?;
            self.end_frame()
        }
    }

    /// Textured triangle blit into the active batched frame.
    pub fn blit_tri_fit_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        v0: i32,
        dx1: f32,
        dy1: f32,
        v1: i32,
        dx2: f32,
        dy2: f32,
        v2: i32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_tri_fit;
            nema_blit_tri_fit(dx0, dy0, v0, dx1, dy1, v1, dx2, dy2, v2);
        });
        Ok(())
    }

    /// Blit a triangular region of `src` with explicit UV coordinates.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_tri_uv(
        &mut self,
        dst: &impl GpuSurface,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        dw0: f32,
        dx1: f32,
        dy1: f32,
        dw1: f32,
        dx2: f32,
        dy2: f32,
        dw2: f32,
        sx0: f32,
        sy0: f32,
        sx1: f32,
        sy1: f32,
        sx2: f32,
        sy2: f32,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_tri_uv_in_frame(
                src, dx0, dy0, dw0, dx1, dy1, dw1, dx2, dy2, dw2, sx0, sy0, sx1, sy1, sx2, sy2,
            )
        } else {
            self.begin_frame(dst)?;
            self.blit_tri_uv_in_frame(
                src, dx0, dy0, dw0, dx1, dy1, dw1, dx2, dy2, dw2, sx0, sy0, sx1, sy1, sx2, sy2,
            )?;
            self.end_frame()
        }
    }

    /// Textured triangle blit with UVs into the active batched frame.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_tri_uv_in_frame(
        &mut self,
        src: &impl GpuSurface,
        dx0: f32,
        dy0: f32,
        dw0: f32,
        dx1: f32,
        dy1: f32,
        dw1: f32,
        dx2: f32,
        dy2: f32,
        dw2: f32,
        sx0: f32,
        sy0: f32,
        sx1: f32,
        sy1: f32,
        sx2: f32,
        sy2: f32,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_blit_src(frame.dst, src);
        self.track_surface(SurfaceSyncInfo::from_surface(src));
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit_tri_uv;
            nema_blit_tri_uv(
                dx0, dy0, dw0, dx1, dy1, dw1, dx2, dy2, dw2, sx0, sy0, sx1, sy1, sx2, sy2,
            );
        });
        Ok(())
    }

    /// Set the GPU blending mode for solid fill operations.
    pub fn set_blend_fill(&mut self, mode: BlendMode) {
        unsafe {
            use crate::ffi::nema_gfx::{nema_set_blend, nema_tex_t_NEMA_NOTEX, nema_tex_t_NEMA_TEX0};
            nema_set_blend(
                mode as u32,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );
        }
    }

    /// Set the GPU blending mode for blit operations.
    pub fn set_blend_blit(&mut self, mode: BlendMode) {
        unsafe {
            use crate::ffi::nema_gfx::{
                nema_set_blend, nema_tex_t_NEMA_NOTEX, nema_tex_t_NEMA_TEX0, nema_tex_t_NEMA_TEX1,
            };
            nema_set_blend(
                mode as u32,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_TEX1,
                nema_tex_t_NEMA_NOTEX,
            );
        }
    }

    /// Set constant global color and alpha opacity for subsequent GPU operations.
    pub fn set_const_color(&mut self, color: Rgba8888) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_const_color;
            nema_set_const_color(color.bits());
        }
    }

    /// Set texture tint color for alpha-mask blits.
    pub fn set_tex_color(&mut self, color: Rgba8888) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_tex_color;
            nema_set_tex_color(color.bits());
        }
    }

    /// Blit an A8 alpha mask tinted with `color`.
    pub fn blit_alpha_mask(
        &mut self,
        dst: &impl GpuSurface,
        mask_phys_addr: usize,
        mask_w: u32,
        mask_h: u32,
        mask_stride: i32,
        dst_x: i32,
        dst_y: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        if self.frame.is_some() {
            self.blit_alpha_mask_in_frame(mask_phys_addr, mask_w, mask_h, mask_stride, dst_x, dst_y, color)
        } else {
            self.begin_frame(dst)?;
            self.blit_alpha_mask_in_frame(mask_phys_addr, mask_w, mask_h, mask_stride, dst_x, dst_y, color)?;
            self.end_frame()
        }
    }

    /// Alpha-mask blit into the active batched frame.
    pub fn blit_alpha_mask_in_frame(
        &mut self,
        mask_phys_addr: usize,
        mask_w: u32,
        mask_h: u32,
        mask_stride: i32,
        dst_x: i32,
        dst_y: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        let frame = self.ensure_frame()?;
        bind_dst_info(frame.dst);
        unsafe {
            use crate::ffi::nema_gfx::{
                NEMA_A8, NEMA_FILTER_BL, nema_bind_src_tex, nema_set_blend, nema_set_tex_color, nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_TEX0, nema_tex_t_NEMA_TEX1,
            };
            nema_bind_src_tex(
                mask_phys_addr,
                mask_w,
                mask_h,
                NEMA_A8,
                mask_stride,
                NEMA_FILTER_BL as u8,
            );
            nema_set_tex_color(color.bits());
            nema_set_blend(
                BlendMode::Simple as u32,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_TEX1,
                nema_tex_t_NEMA_NOTEX,
            );
        }
        record_blit_op(|| unsafe {
            use crate::ffi::nema_gfx::nema_blit;
            nema_blit(dst_x, dst_y);
        });
        Ok(())
    }

    /// Set GPU hardware clipping rectangle (`x`, `y`, `w`, `h`).
    pub fn set_clip(&mut self, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_clip;
            nema_set_clip(x, y, w, h);
        }
    }

    /// Reset GPU hardware clipping rectangle to cover the full target framebuffer bounds.
    pub fn reset_clip(&mut self, framebuffer: &impl GpuSurface) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_clip;
            nema_set_clip(0, 0, framebuffer.width(), framebuffer.height());
        }
    }

    fn ensure_frame(&mut self) -> Result<&mut FrameState, Error> {
        self.frame.as_mut().ok_or(Error::NoActiveFrame)
    }

    fn track_surface(&mut self, surface: SurfaceSyncInfo) {
        if let Some(frame) = self.frame.as_mut() {
            if frame.surfaces.iter().all(|s| s != &surface) {
                let _ = frame.surfaces.push(surface);
            }
        }
    }

    fn take_frame_surfaces(&mut self) -> Result<Vec<SurfaceSyncInfo, 8>, Error> {
        let frame = self.frame.take().ok_or(Error::NoActiveFrame)?;
        Ok(frame.surfaces)
    }
}

fn bind_dst(framebuffer: &impl GpuSurface) {
    bind_dst_info(SurfaceSyncInfo::from_surface(framebuffer));
}

fn bind_dst_info(info: SurfaceSyncInfo) {
    unsafe {
        use crate::ffi::nema_gfx::nema_bind_dst_tex;
        nema_bind_dst_tex(
            info.phys_addr,
            info.width,
            info.height,
            info.format.nema_format(),
            info.stride,
        );
    }
}

fn bind_blit_src(dst: SurfaceSyncInfo, src: &impl GpuSurface) {
    unsafe {
        use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex};
        bind_dst_info(dst);
        nema_bind_src_tex(
            src.phys_addr(),
            src.width(),
            src.height(),
            src.format().nema_format(),
            src.stride(),
            NEMA_TEX_BORDER as u8,
        );
    }
}

fn record_dst_op(op: impl FnOnce()) {
    op();
}

fn record_blit_op(op: impl FnOnce()) {
    op();
}
