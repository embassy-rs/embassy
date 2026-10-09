//! NemaGFX-backed [`RasterSink`]: hands embedded-3dgfx's already-transformed
//! primitives to the NeoChrom GPU2D, so the demo's 3D viewport rasterizes in
//! hardware instead of on the CPU.
//!
//! **What this actually carries:** the demo's meshes are wireframe
//! (`RenderMode::Lines`, `faces: &[]`), so the line path is the one that matters
//! here. Triangles are handled too, for meshes that use a solid render mode.
//!
//! **Iteration 1 -- geometry path only, depth deliberately disabled.** The point is
//! to validate the seam before introducing GPU depth, whose buffer format does not
//! match the engine's `u32` z-buffer:
//!
//!   * do primitives actually arrive (the counts are logged),
//!   * is the coordinate space viewport-local with the right offset, and
//!   * do RGB565 colours survive the RGB565 -> RGBA8888 -> RGB565 round trip?
//!
//! The engine's line primitive carries no depth, so for the wireframe the only
//! expected difference from the CPU path is anti-aliased edges. Geometry in the
//! wrong place means the offset is wrong; wrong colours mean the round trip is.

use core::cell::RefCell;
use core::fmt;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_3dgfx::pipeline::rasterize::draw::sink::RasterSink;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use nalgebra::Point2;
use stm32_bindings::nema_gfx::*;

/// The demo's two meshes are ~24 lines and ~20 triangles; this leaves headroom. On
/// overflow the method returns `false` and the CPU rasterizer draws the rest, so an
/// undersized buffer degrades rather than corrupts.
const MAX_TRIANGLES: usize = 96;
const MAX_LINES: usize = 96;

#[derive(Clone, Copy)]
struct Tri {
    p: [Point2<i32>; 3],
    color: Rgb565,
}

#[derive(Clone, Copy)]
struct Seg {
    a: Point2<i32>,
    b: Point2<i32>,
    color: Rgb565,
}

type TriBuf = RefCell<heapless::Vec<Tri, MAX_TRIANGLES>>;
type LineBuf = RefCell<heapless::Vec<Seg, MAX_LINES>>;

pub struct NemaRasterSink {
    tris: Mutex<CriticalSectionRawMutex, TriBuf>,
    lines: Mutex<CriticalSectionRawMutex, LineBuf>,
}

impl NemaRasterSink {
    pub const fn new() -> Self {
        Self {
            tris: Mutex::new(RefCell::new(heapless::Vec::new())),
            lines: Mutex::new(RefCell::new(heapless::Vec::new())),
        }
    }

    /// Emit everything buffered into the viewport rectangle at `(off_x, off_y)`.
    ///
    /// Returns `(triangles, lines)` emitted. The caller owns the command list and is
    /// responsible for submitting and waiting -- the sink deliberately does not
    /// submit, so the order of GPU work in the frame stays visible at the call site
    /// rather than hidden inside the sink.
    pub fn flush(
        &self,
        dst_ptr: usize,
        dst_w: u32,
        dst_h: u32,
        dst_stride: i32,
        off_x: i32,
        off_y: i32,
    ) -> (usize, usize) {
        let (n_tris, n_lines) = (
            self.tris.lock(|cell| cell.borrow().len()),
            self.lines.lock(|cell| cell.borrow().len()),
        );
        if n_tris == 0 && n_lines == 0 {
            return (0, 0);
        }

        unsafe {
            nema_bind_dst_tex(dst_ptr, dst_w, dst_h, NEMA_RGB565, dst_stride);
            // Confine drawing to the viewport. The engine only culls primitives
            // that are *entirely* offscreen, so a partially visible one would
            // otherwise spill across the surrounding dashboard cards.
            nema_set_clip(off_x, off_y, dst_w, dst_h);
            nema_set_blend(
                NEMA_BF_ONE,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );
            nema_enable_depth(0);
        }

        // The engine hands us viewport-local coordinates: it is constructed as
        // K3dengine::new(260, 200) and the demo's ViewportTarget maps local ->
        // framebuffer with the offset passed in here.
        self.tris.lock(|cell| {
            let mut tris = cell.borrow_mut();
            for t in tris.iter() {
                let (r, g, b) = expand(t.color);
                unsafe {
                    nema_fill_triangle_f(
                        (t.p[0].x + off_x) as f32,
                        (t.p[0].y + off_y) as f32,
                        (t.p[1].x + off_x) as f32,
                        (t.p[1].y + off_y) as f32,
                        (t.p[2].x + off_x) as f32,
                        (t.p[2].y + off_y) as f32,
                        nema_rgba(r, g, b, 255),
                    );
                }
            }
            tris.clear();
        });

        self.lines.lock(|cell| {
            let mut lines = cell.borrow_mut();
            for l in lines.iter() {
                let (r, g, b) = expand(l.color);
                unsafe {
                    // The GPU's own AA line, replacing CPU Bresenham: one call per
                    // segment instead of a draw_iter per pixel.
                    nema_draw_line_aa(
                        (l.a.x + off_x) as f32,
                        (l.a.y + off_y) as f32,
                        (l.b.x + off_x) as f32,
                        (l.b.y + off_y) as f32,
                        1.0,
                        nema_rgba(r, g, b, 255),
                    );
                }
            }
            lines.clear();
        });

        (n_tris, n_lines)
    }
}

impl fmt::Debug for NemaRasterSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NemaRasterSink")
    }
}

impl RasterSink for NemaRasterSink {
    fn triangle(&self, points: &[Point2<i32>; 3], _depths: &[f32; 3], color: Rgb565) -> bool {
        self.tris.lock(|cell| {
            let mut tris = cell.borrow_mut();
            tris.push(Tri { p: *points, color }).is_ok()
        })
    }

    fn line(&self, a: Point2<i32>, b: Point2<i32>, color: Rgb565) -> bool {
        self.lines.lock(|cell| {
            let mut lines = cell.borrow_mut();
            lines.push(Seg { a, b, color }).is_ok()
        })
    }
}

/// RGB565 -> the 8-bit channels `nema_rgba` wants. Replicating the high bits
/// (`r << 3 | r >> 2`) means the GPU's RGB565 destination quantises back to exactly
/// the colour we were given, so this round trip is lossless.
fn expand(c: Rgb565) -> (u8, u8, u8) {
    let r = c.r();
    let g = c.g();
    let b = c.b();
    ((r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2))
}

/// The engine holds its sink by `'static` reference, so it lives here.
pub static NEMA_RASTER_SINK: NemaRasterSink = NemaRasterSink::new();

/// Emit queued framebuffer fills into the currently bound command list.
///
/// Binds the destination itself rather than assuming one, and resets the clip to the
/// whole destination -- the caller may have left a viewport clip in place. Returns
/// how many fills were emitted.
pub fn emit_fills(
    fills: &[crate::framebuffer::GpuFill],
    dst_ptr: usize,
    dst_w: u32,
    dst_h: u32,
    dst_stride: i32,
) -> usize {
    if fills.is_empty() {
        return 0;
    }
    unsafe {
        nema_bind_dst_tex(dst_ptr, dst_w, dst_h, NEMA_RGB565, dst_stride);
        nema_set_clip(0, 0, dst_w, dst_h);
        nema_set_blend(
            NEMA_BF_ONE,
            nema_tex_t_NEMA_TEX0,
            nema_tex_t_NEMA_NOTEX,
            nema_tex_t_NEMA_NOTEX,
        );
        nema_enable_depth(0);
        for f in fills {
            let (r, g, b) = expand(f.color);
            nema_fill_rect_f(f.x as f32, f.y as f32, f.w as f32, f.h as f32, nema_rgba(r, g, b, 255));
        }
    }
    fills.len()
}
