//! Draw a widget tree into the PSRAM RGB565 framebuffer.

use core::slice;

use rlvgl::core::WidgetNode;
use rlvgl::core::renderer::Renderer;
use rlvgl::core::widget::{Color, Rect as WidgetRect};
use rlvgl::platform::CpuBlitter;
use rlvgl::platform::blit::{BlitterRenderer, PixelFmt, Surface};

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const DIRTY_RECTS: usize = 64;

/// Number of scan lines rendered into SRAM at once for tiled full-screen draws.
///
/// `800 × 39 × RGB565 = 62.4 KiB`, small enough to keep in SRAM beside the heap
/// and the scan-out bounce buffers, while still giving the renderer and the
/// final PSRAM copy reasonably linear memory access.
pub const TILE_LINES: usize = 39;

/// Render `root` into the scan-out framebuffer (continuous PIO DMA reads PSRAM).
pub fn render_tree(fb: *mut u16, root: &WidgetNode) {
    if fb.is_null() {
        return;
    }

    let byte_len = DISPLAY_WIDTH * DISPLAY_HEIGHT * 2;
    let buf = unsafe { slice::from_raw_parts_mut(fb.cast::<u8>(), byte_len) };

    let mut blitter = CpuBlitter;
    let surface = Surface::new(
        buf,
        DISPLAY_WIDTH * 2,
        PixelFmt::Rgb565,
        DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
    );
    let mut renderer = BlitterRenderer::<_, DIRTY_RECTS>::new(&mut blitter, surface);
    root.draw(&mut renderer);
}

/// Render the full widget tree through an SRAM-sized tile, then copy each tile
/// linearly into PSRAM.
///
/// This keeps the expensive widget drawing phase off the QMI/PSRAM bus. The
/// final framebuffer update still writes the same 768 KiB for a true full-screen
/// repaint, but it does so as wide, sequential row copies instead of many small
/// widget/pixel writes directly into PSRAM.
pub fn render_tree_tiled(fb: *mut u16, tile: &mut [u16; DISPLAY_WIDTH * TILE_LINES], root: &WidgetNode) {
    if fb.is_null() {
        return;
    }

    let mut y = 0;
    while y < DISPLAY_HEIGHT {
        let lines = (DISPLAY_HEIGHT - y).min(TILE_LINES);
        let tile_words = DISPLAY_WIDTH * lines;
        let tile_bytes = tile_words * 2;
        let tile_buf = unsafe { slice::from_raw_parts_mut(tile.as_mut_ptr().cast::<u8>(), tile_bytes) };

        let mut blitter = CpuBlitter;
        let surface = Surface::new(
            tile_buf,
            DISPLAY_WIDTH * 2,
            PixelFmt::Rgb565,
            DISPLAY_WIDTH as u32,
            lines as u32,
        );
        let mut renderer = BlitterRenderer::<_, DIRTY_RECTS>::new(&mut blitter, surface);
        let mut tiled = TileRenderer::new(&mut renderer, y as i32, lines as i32);
        root.draw(&mut tiled);

        unsafe {
            core::ptr::copy_nonoverlapping(tile.as_ptr(), fb.add(y * DISPLAY_WIDTH), tile_words);
        }

        y += lines;
    }
}

struct TileRenderer<'a, R> {
    inner: &'a mut R,
    y_offset: i32,
    height: i32,
}

impl<'a, R> TileRenderer<'a, R> {
    fn new(inner: &'a mut R, y_offset: i32, height: i32) -> Self {
        Self {
            inner,
            y_offset,
            height,
        }
    }

    fn clip_rect(&self, rect: WidgetRect) -> Option<WidgetRect> {
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(self.y_offset);
        let x1 = (rect.x + rect.width).min(DISPLAY_WIDTH as i32);
        let y1 = (rect.y + rect.height).min(self.y_offset + self.height);

        if x0 >= x1 || y0 >= y1 {
            return None;
        }

        Some(WidgetRect {
            x: x0,
            y: y0 - self.y_offset,
            width: x1 - x0,
            height: y1 - y0,
        })
    }
}

impl<R: Renderer> Renderer for TileRenderer<'_, R> {
    fn fill_rect(&mut self, rect: WidgetRect, color: Color) {
        if let Some(rect) = self.clip_rect(rect) {
            self.inner.fill_rect(rect, color);
        }
    }

    fn blend_rect(&mut self, rect: WidgetRect, color: Color) {
        if let Some(rect) = self.clip_rect(rect) {
            self.inner.blend_rect(rect, color);
        }
    }

    fn draw_text(&mut self, position: (i32, i32), text: &str, color: Color) {
        // Let the inner renderer's pixel-level clipping handle glyphs that
        // straddle a tile boundary. Only skip text baselines far outside the
        // tile to avoid repeated work on unrelated tiles.
        let y = position.1;
        if y < self.y_offset - 32 || y >= self.y_offset + self.height + 32 {
            return;
        }

        self.inner
            .draw_text((position.0, position.1 - self.y_offset), text, color);
    }

    fn draw_pixels(&mut self, position: (i32, i32), pixels: &[Color], width: u32, height: u32) {
        for py in 0..height as i32 {
            let dst_y = position.1 + py;
            if dst_y < self.y_offset || dst_y >= self.y_offset + self.height {
                continue;
            }

            for px in 0..width as i32 {
                let dst_x = position.0 + px;
                if dst_x < 0 || dst_x >= DISPLAY_WIDTH as i32 {
                    continue;
                }

                let idx = (py as u32 * width + px as u32) as usize;
                if let Some(&color) = pixels.get(idx) {
                    self.inner.fill_rect(
                        WidgetRect {
                            x: dst_x,
                            y: dst_y - self.y_offset,
                            width: 1,
                            height: 1,
                        },
                        color,
                    );
                }
            }
        }
    }
}

/// Partial render: draw a single `node` (and its children) into the live
/// scan-out framebuffer.
///
/// This mirrors the C reference's single-framebuffer + partial-flush strategy:
/// with one persistent PSRAM framebuffer the background and all static widgets
/// already reside in memory, so only the few widgets that actually changed this
/// frame need to be re-drawn. Each `node` writes just its own small bounds
/// instead of re-writing the whole 800×480 (768 KiB) frame, which removes the
/// PSRAM write pressure that was starving the scan-out refill DMA on the shared
/// QMI bus (vertical roll / flicker).
pub fn render_node(fb: *mut u16, node: &WidgetNode) {
    if fb.is_null() {
        return;
    }

    let byte_len = DISPLAY_WIDTH * DISPLAY_HEIGHT * 2;
    let buf = unsafe { slice::from_raw_parts_mut(fb.cast::<u8>(), byte_len) };

    let mut blitter = CpuBlitter;
    let surface = Surface::new(
        buf,
        DISPLAY_WIDTH * 2,
        PixelFmt::Rgb565,
        DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
    );
    let mut renderer = BlitterRenderer::<_, DIRTY_RECTS>::new(&mut blitter, surface);
    node.draw(&mut renderer);
}
