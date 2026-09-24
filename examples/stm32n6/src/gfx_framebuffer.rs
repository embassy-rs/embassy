//! Framebuffer storage and DrawTarget implementation for embedded-graphics text rendering.

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;

/// Most queued GPU fills in one frame. The measured frame issues ~435 `fill_solid`
/// calls; overflow falls back to a CPU fill rather than silently dropping a rect.
pub const MAX_GPU_FILLS: usize = 512;

/// Fills below this stay on the CPU. A NemaGFX call plus its command-list entry
/// costs more than filling a few dozen pixels with `slice::fill`.
pub const MIN_GPU_FILL_PX: u32 = 256;

/// One queued rectangle fill, in framebuffer coordinates.
#[derive(Clone, Copy)]
pub struct GpuFill {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub color: Rgb565,
}

pub struct Framebuffer<'a> {
    pixels: &'a mut [u16],
    width: u16,
    height: u16,
    /// When set, opaque `fill_solid` calls are queued for the GPU instead of being
    /// written. Off by default, so a framebuffer used without a GPU still fills.
    pub gpu_fills: bool,
    fills: heapless::Vec<GpuFill, MAX_GPU_FILLS>,
    /// Diagnostic counters: which DrawTarget write path the renderers actually use,
    /// and how many pixels each accounts for. A few increments per *call*, not per
    /// pixel, so they are cheap enough to leave in place.
    pub stat_fill_calls: u32,
    pub stat_fill_px: u32,
    pub stat_contig_calls: u32,
    pub stat_contig_px: u32,
    pub stat_iter_calls: u32,
    pub stat_iter_px: u32,
    /// Fills handed to the GPU, and how many times a CPU write arrived first and
    /// forced the queue to be drained back onto the CPU. A non-zero drain count is
    /// correct but slower, and worth seeing rather than inferring.
    pub stat_gpu_fill_calls: u32,
    pub stat_gpu_fill_px: u32,
    pub stat_gpu_fill_drains: u32,
}

impl<'a> Framebuffer<'a> {
    pub fn new(pixels: &'a mut [u16], width: u16, height: u16) -> Self {
        assert_eq!(pixels.len(), width as usize * height as usize);
        Self {
            pixels,
            width,
            height,
            gpu_fills: false,
            fills: heapless::Vec::new(),
            stat_fill_calls: 0,
            stat_fill_px: 0,
            stat_contig_calls: 0,
            stat_contig_px: 0,
            stat_iter_calls: 0,
            stat_iter_px: 0,
            stat_gpu_fill_calls: 0,
            stat_gpu_fill_px: 0,
            stat_gpu_fill_drains: 0,
        }
    }

    /// Rectangles queued for the GPU, in request order.
    pub fn fills(&self) -> &[GpuFill] {
        &self.fills
    }

    /// Call once the queue has been handed to the GPU.
    pub fn clear_fills(&mut self) {
        self.fills.clear();
    }

    /// Apply the queue on the CPU and empty it.
    ///
    /// Required before any CPU write that must land *on top of* a queued fill, and
    /// before any CPU read that must observe one. The GPU executes later, so a
    /// queued fill would otherwise erase a write that came after it.
    fn drain_fills(&mut self) {
        if self.fills.is_empty() {
            return;
        }
        self.stat_gpu_fill_drains += 1;
        let w = self.width as usize;
        let h = self.height as usize;
        for f in self.fills.iter() {
            let x0 = f.x.max(0) as usize;
            let y0 = f.y.max(0) as usize;
            let x1 = (f.x + f.w as i32).clamp(0, self.width as i32) as usize;
            let y1 = (f.y + f.h as i32).clamp(0, self.height as i32) as usize;
            if x1 <= x0 || y1 <= y0 || x0 >= w || y0 >= h {
                continue;
            }
            let raw = RawU16::from(f.color).into_inner();
            for y in y0..y1 {
                let start = y * w + x0;
                self.pixels[start..start + (x1 - x0)].fill(raw);
            }
        }
        self.fills.clear();
    }

    /// Fills handed to the GPU, plus how many times a CPU write arrived first and
    /// forced the queue back onto the CPU. A non-zero drain count means the fills
    /// were *recorded* as offloaded but actually executed on the CPU, which is a very
    /// different thing from them having been offloaded -- and invisible without this.
    pub fn take_gpu_fill_stats(&mut self) -> (u32, u32, u32) {
        let taken = (
            self.stat_gpu_fill_calls,
            self.stat_gpu_fill_px,
            self.stat_gpu_fill_drains,
        );
        self.stat_gpu_fill_calls = 0;
        self.stat_gpu_fill_px = 0;
        self.stat_gpu_fill_drains = 0;
        taken
    }

    /// Take the counters, leaving them zeroed for the next interval.
    pub fn take_stats(&mut self) -> (u32, u32, u32, u32, u32, u32) {
        let taken = (
            self.stat_fill_calls,
            self.stat_fill_px,
            self.stat_contig_calls,
            self.stat_contig_px,
            self.stat_iter_calls,
            self.stat_iter_px,
        );
        self.stat_fill_calls = 0;
        self.stat_fill_px = 0;
        self.stat_contig_calls = 0;
        self.stat_contig_px = 0;
        self.stat_iter_calls = 0;
        self.stat_iter_px = 0;
        taken
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.pixels.as_ptr()
    }

    #[allow(dead_code)]
    pub fn pixels_mut(&mut self) -> (&mut [u16], u16) {
        let stride = self.width;
        (self.pixels, stride)
    }
}

impl OriginDimensions for Framebuffer<'_> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl DrawTarget for Framebuffer<'_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // A queued GPU fill would be drawn after these pixels and erase them.
        self.drain_fills();
        let w = self.width as usize;
        let h = self.height as usize;
        let mut n = 0u32;
        for Pixel(Point { x, y }, color) in pixels {
            n += 1;
            let ux = x as usize;
            let uy = y as usize;
            if (x >= 0) && (y >= 0) && ux < w && uy < h {
                self.pixels[uy * w + ux] = RawU16::from(color).into_inner();
            }
        }
        self.stat_iter_calls += 1;
        self.stat_iter_px += n;
        Ok(())
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        let bb = self.bounding_box();
        let clipped = area.intersection(&bb);
        if clipped.is_zero_sized() {
            return Ok(());
        }
        let Some(bottom_right) = clipped.bottom_right() else {
            return Ok(());
        };
        let row_len = (bottom_right.x - clipped.top_left.x + 1) as usize;
        let rows = (bottom_right.y - clipped.top_left.y + 1) as u32;
        let px = row_len as u32 * rows;

        // Queue it for the GPU when asked, if it is worth the call and the queue has
        // room. A fill that cannot be queued is written here instead, so the result
        // is always correct -- just not always offloaded.
        if self.gpu_fills && px >= MIN_GPU_FILL_PX {
            let fill = GpuFill {
                x: clipped.top_left.x,
                y: clipped.top_left.y,
                w: row_len as u32,
                h: rows,
                color,
            };
            if self.fills.push(fill).is_ok() {
                self.stat_gpu_fill_calls += 1;
                self.stat_gpu_fill_px += px;
                return Ok(());
            }
        }

        let w = self.width as usize;
        let c_raw = RawU16::from(color).into_inner();
        for y in clipped.top_left.y..=bottom_right.y {
            let start = (y as usize) * w + (clipped.top_left.x as usize);
            self.pixels[start..start + row_len].fill(c_raw);
        }
        self.stat_fill_calls += 1;
        self.stat_fill_px += px;
        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // Anything queued for the GPU must happen first, or it would be drawn after
        // this and erase it.
        self.drain_fills();
        let bb = self.bounding_box();
        let clipped = area.intersection(&bb);
        if clipped.is_zero_sized() {
            return Ok(());
        }

        let Some(bottom_right) = area.bottom_right() else {
            return Ok(());
        };
        let Some(clipped_br) = clipped.bottom_right() else {
            return Ok(());
        };

        let mut colors = colors.into_iter();
        let w = self.width as usize;

        // Fast path: area is completely inside bounds (applies to all UI text)
        if clipped == *area && area.top_left.x >= 0 && area.top_left.y >= 0 {
            let row_len = (bottom_right.x - area.top_left.x + 1) as usize;
            let rows = (bottom_right.y - area.top_left.y + 1) as u32;
            for y in area.top_left.y..=bottom_right.y {
                let row_start = (y as usize) * w + (area.top_left.x as usize);
                let row_slice = &mut self.pixels[row_start..row_start + row_len];
                for px in row_slice.iter_mut() {
                    if let Some(color) = colors.next() {
                        *px = RawU16::from(color).into_inner();
                    }
                }
            }
            self.stat_contig_calls += 1;
            self.stat_contig_px += row_len as u32 * rows;
            return Ok(());
        }

        for y in area.top_left.y..=bottom_right.y {
            for x in area.top_left.x..=bottom_right.x {
                if let Some(color) = colors.next() {
                    if y >= clipped.top_left.y && y <= clipped_br.y && x >= clipped.top_left.x && x <= clipped_br.x {
                        let idx = y as usize * w + x as usize;
                        self.pixels[idx] = RawU16::from(color).into_inner();
                    }
                }
            }
        }
        self.stat_contig_calls += 1;
        self.stat_contig_px += ((clipped_br.x - clipped.top_left.x + 1).max(0) as u32)
            * ((clipped_br.y - clipped.top_left.y + 1).max(0) as u32);
        Ok(())
    }
}

use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::iso_8859_1::FONT_7X13;
use embedded_graphics::text::{Baseline, Text};

struct MiniSampler {
    pixels: [u8; 7 * 13],
}

impl OriginDimensions for MiniSampler {
    fn size(&self) -> Size {
        Size::new(7, 13)
    }
}

impl DrawTarget for MiniSampler {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x >= 0 && x < 7 && y >= 0 && y < 13 {
                if color != Rgb565::BLACK {
                    self.pixels[(y * 7 + x) as usize] = 1;
                }
            }
        }
        Ok(())
    }
}

/// Ultra-fast pre-sampled 7x13 font cache.
/// Avoids embedded-graphics runtime iterator bit-scanning over 128-char atlas images,
/// providing direct ~15 microsecond text rendering.
pub struct FastMonoFont {
    // ASCII 32 (' ') ..= 127: 96 characters
    // 13 rows per char, 7 pixels per row (stored in bits 6..0 of u8)
    glyphs: [[u8; 13]; 96],
}

impl FastMonoFont {
    pub fn new() -> Self {
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .build();

        let mut glyphs = [[0u8; 13]; 96];

        for c_val in 32u8..128u8 {
            let ch = c_val as char;
            let mut s = [0u8; 4];
            let s_str = ch.encode_utf8(&mut s);

            let mut sampler = MiniSampler { pixels: [0; 7 * 13] };
            let _ = Text::with_baseline(s_str, Point::zero(), style, Baseline::Top).draw(&mut sampler);

            let idx = (c_val - 32) as usize;
            for y in 0..13 {
                let mut row_byte = 0u8;
                for x in 0..7 {
                    if sampler.pixels[y * 7 + x] != 0 {
                        row_byte |= 1 << (6 - x);
                    }
                }
                glyphs[idx][y] = row_byte;
            }
        }

        Self { glyphs }
    }

    /// Bake this font into an RGB565 atlas for GPU blitting.
    ///
    /// The background colour is baked in, so a `NEMA_BL_SRC` blit of a cell reproduces
    /// exactly what `draw_str` would have written -- no mask or blend setup needed, which
    /// matters because a 1-bpp source would only ever give black and white.
    ///
    /// Laid out as `cols` cells of 7x13 per row; cell `i` for character `32 + i` sits at
    /// `((i % cols) * 7, (i / cols) * 13)`, so drawing a glyph needs no search:
    /// `nema_blit_subrect(x, y, 7, 13, sx, sy)`. Returns the atlas size in pixels.
    pub fn build_atlas(&self, out: &mut [u16], cols: usize, fg: Rgb565, bg: Rgb565) -> (u32, u32) {
        const GW: usize = 7;
        const GH: usize = 13;
        let cols = cols.max(1);
        let rows = 96usize.div_ceil(cols);
        let aw = cols * GW;
        let fg_raw = RawU16::from(fg).into_inner();
        let bg_raw = RawU16::from(bg).into_inner();

        for i in 0..96usize {
            let cx = (i % cols) * GW;
            let cy = (i / cols) * GH;
            for y in 0..GH {
                let bits = self.glyphs[i][y];
                for x in 0..GW {
                    let on = (bits >> (6 - x)) & 1 != 0;
                    out[(cy + y) * aw + cx + x] = if on { fg_raw } else { bg_raw };
                }
            }
        }
        (aw as u32, (rows * GH) as u32)
    }

    pub fn draw_str(&self, fb: &mut Framebuffer, x0: usize, y0: usize, text: &str, fg: Rgb565, bg: Rgb565) {
        let fg_raw = RawU16::from(fg).into_inner();
        let bg_raw = RawU16::from(bg).into_inner();
        let mut cur_x = x0;
        let w = fb.width as usize;
        let h = fb.height as usize;

        for b in text.bytes() {
            let idx = if (32..=127).contains(&b) { (b - 32) as usize } else { 0 };
            let glyph = &self.glyphs[idx];

            if cur_x + 7 > w {
                break;
            }

            for dy in 0..13 {
                let y = y0 + dy;
                if y >= h {
                    break;
                }
                let row_bits = glyph[dy];
                let row_offset = y * w + cur_x;
                for dx in 0..7 {
                    let c = if (row_bits & (1 << (6 - dx))) != 0 {
                        fg_raw
                    } else {
                        bg_raw
                    };
                    fb.pixels[row_offset + dx] = c;
                }
            }
            cur_x += 7;
        }
    }
}
