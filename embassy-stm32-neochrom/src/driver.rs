//! NeoChrom GPU driver using NemaGFX.

use crate::color::Rgba8888;
use crate::error::{Error, InitError};
use crate::ffi::nema_gfx::{
    NEMA_RGBA8888, nema_bind_dst_tex, nema_cl_bind, nema_cl_create, nema_cl_submit, nema_cl_wait, nema_clear, nema_init,
};
use crate::framebuffer::FrameBuffer;

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

/// NeoChrom GPU context.
///
/// Call [`NeoChrom::new`] once after GPU2D clocks are enabled. With the default
/// `stub-gpu2d` feature, a stub HAL is used for link/CI testing.
pub struct NeoChrom {
    _init: (),
}

impl NeoChrom {
    /// Initialize NemaGFX and the platform HAL using the in-tree GPU2D stub.
    ///
    /// Only available with the default `stub-gpu2d` feature (CI / link tests).
    #[cfg(feature = "stub-gpu2d")]
    pub fn new() -> Result<Self, InitError> {
        nema_gfx_hal::gpu2d_init_stub().map_err(|_| InitError::Gpu2d)?;
        Self::finish_init()
    }

    /// Initialize NemaGFX using the real GPU2D peripheral.
    ///
    /// `peri`/`irq` are consumed to enable the GPU2D peripheral clock and
    /// interrupt via [`embassy_stm32::gpu2d::Gpu2d`]. Only available without
    /// the default `stub-gpu2d` feature.
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

        Ok(Self { _init: () })
    }

    /// Clear `framebuffer` to `color` using the GPU command list path.
    pub fn clear<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_clear(color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Fill a rectangle at (`x`, `y`) with dimensions `w`x`h` using the GPU.
    pub fn fill_rect<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::nema_fill_rect;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_fill_rect(x, y, w, h, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Draw a line from (`x0`, `y0`) to (`x1`, `y1`) using the GPU.
    pub fn draw_line<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::nema_draw_line;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_draw_line(x0, y0, x1, y1, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Fill a circle at (`cx`, `cy`) with radius `r` using the GPU.
    pub fn fill_circle<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        cx: i32,
        cy: i32,
        r: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::nema_fill_circle;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_fill_circle(cx, cy, r, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit (copy) source framebuffer `src` into destination framebuffer `dst` at (`dst_x`, `dst_y`).
    pub fn blit<const DW: u32, const DH: u32, const DN: usize, const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
        src: &FrameBuffer<SW, SH, SN>,
        dst_x: i32,
        dst_y: i32,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit(dst_x, dst_y);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Fill a rounded rectangle with corner radius `r` using the GPU.
    pub fn fill_rounded_rect<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::nema_fill_rounded_rect;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_fill_rounded_rect(x, y, w, h, r, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Fill a triangle defined by 3 vertices (`x0`,`y0`), (`x1`,`y1`), (`x2`,`y2`) using the GPU.
    pub fn fill_triangle<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::nema_fill_triangle;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_fill_triangle(x0, y0, x1, y1, x2, y2, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Fill a quadrilateral defined by 4 vertices using the GPU.
    pub fn fill_quad<const W: u32, const H: u32, const N: usize>(
        &mut self,
        framebuffer: &FrameBuffer<W, H, N>,
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
        unsafe {
            use crate::ffi::nema_gfx::nema_fill_quad;
            nema_bind_dst_tex(framebuffer.phys_addr(), W, H, NEMA_RGBA8888, -1);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_fill_quad(x0, y0, x1, y1, x2, y2, x3, y3, color.bits());
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit source texture `src` to `dst` at (`dst_x`, `dst_y`) scaled to `dst_w` x `dst_h`.
    pub fn blit_rect_fit<
        const DW: u32,
        const DH: u32,
        const DN: usize,
        const SW: u32,
        const SH: u32,
        const SN: usize,
    >(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
        src: &FrameBuffer<SW, SH, SN>,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit_rect_fit};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit_rect_fit(dst_x, dst_y, dst_w, dst_h);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit source texture `src` to `dst` at (`dst_x`, `dst_y`) with rotation `angle_degrees`.
    pub fn blit_rotate<const DW: u32, const DH: u32, const DN: usize, const SW: u32, const SH: u32, const SN: usize>(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
        src: &FrameBuffer<SW, SH, SN>,
        dst_x: i32,
        dst_y: i32,
        angle_degrees: u32,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit_rotate};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit_rotate(dst_x, dst_y, angle_degrees);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit source texture `src` to `dst` with rotation around center (`cx`, `cy`) and pivot (`px`, `py`).
    pub fn blit_rotate_pivot<
        const DW: u32,
        const DH: u32,
        const DN: usize,
        const SW: u32,
        const SH: u32,
        const SN: usize,
    >(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
        src: &FrameBuffer<SW, SH, SN>,
        cx: f32,
        cy: f32,
        px: f32,
        py: f32,
        angle_degrees: f32,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit_rotate_pivot};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit_rotate_pivot(cx, cy, px, py, angle_degrees);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit sub-rectangle of source texture `src` (`src_x`,`src_y`,`src_w`,`src_h`) to destination `dst` (`dst_x`,`dst_y`,`dst_w`,`dst_h`).
    #[allow(clippy::too_many_arguments)]
    pub fn blit_subrect_fit<
        const DW: u32,
        const DH: u32,
        const DN: usize,
        const SW: u32,
        const SH: u32,
        const SN: usize,
    >(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
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
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit_subrect_fit};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit_subrect_fit(dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Blit source texture `src` to destination `dst` warped into quadrilateral defined by 4 vertices.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_quad_fit<
        const DW: u32,
        const DH: u32,
        const DN: usize,
        const SW: u32,
        const SH: u32,
        const SN: usize,
    >(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
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
        unsafe {
            use crate::ffi::nema_gfx::{NEMA_TEX_BORDER, nema_bind_src_tex, nema_blit_quad_fit};
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, NEMA_RGBA8888, -1);
            nema_bind_src_tex(src.phys_addr(), SW, SH, NEMA_RGBA8888, -1, NEMA_TEX_BORDER as u8);

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit_quad_fit(dx0, dy0, dx1, dy1, dx2, dy2, dx3, dy3);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

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

    /// Set texture tint color for alpha-mask (A8/A4/A2/A1) blits (e.g. font glyphs).
    pub fn set_tex_color(&mut self, color: Rgba8888) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_tex_color;
            nema_set_tex_color(color.bits());
        }
    }

    /// Blit an A8 (8-bit alpha mask) font glyph/buffer onto destination framebuffer at (`dst_x`, `dst_y`) tinted with `color`.
    pub fn blit_alpha_mask<const DW: u32, const DH: u32, const DN: usize>(
        &mut self,
        dst: &FrameBuffer<DW, DH, DN>,
        mask_phys_addr: usize,
        mask_w: u32,
        mask_h: u32,
        mask_stride: i32,
        dst_x: i32,
        dst_y: i32,
        color: Rgba8888,
    ) -> Result<(), Error> {
        unsafe {
            use crate::ffi::nema_gfx::{
                NEMA_A8, NEMA_FILTER_BL, nema_bind_src_tex, nema_blit, nema_set_blend, nema_set_tex_color,
                nema_tex_t_NEMA_NOTEX, nema_tex_t_NEMA_TEX0, nema_tex_t_NEMA_TEX1,
            };
            nema_bind_dst_tex(dst.phys_addr(), DW, DH, crate::ffi::nema_gfx::NEMA_RGBA8888, -1);
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

            let mut cl = nema_cl_create();
            nema_cl_bind(&mut cl);
            nema_blit(dst_x, dst_y);
            nema_cl_submit(&mut cl);
            if nema_cl_wait(&mut cl) != 0 {
                return Err(Error::CommandListWait);
            }
        }

        Ok(())
    }

    /// Set GPU hardware clipping rectangle (`x`, `y`, `w`, `h`).
    ///
    /// Any GPU draw/blit operation outside this bounding box is automatically discarded by hardware.
    pub fn set_clip(&mut self, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_clip;
            nema_set_clip(x, y, w, h);
        }
    }

    /// Reset GPU hardware clipping rectangle to cover the full target framebuffer bounds.
    pub fn reset_clip<const W: u32, const H: u32, const N: usize>(&mut self, _framebuffer: &FrameBuffer<W, H, N>) {
        unsafe {
            use crate::ffi::nema_gfx::nema_set_clip;
            nema_set_clip(0, 0, W, H);
        }
    }
}
