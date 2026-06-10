//! RGB565 LTDC framebuffer renderer for [rlvgl](https://github.com/SoftOboros/rlvgl).

use rlvgl::core::bitmap_font::FONT_6X10;
use rlvgl::core::renderer::Renderer;
use rlvgl::core::widget::{Color, Rect};

/// Renderer that writes into an LTDC RGB565 framebuffer.
pub struct Rgb565Renderer<'a> {
    buffer: &'a mut [u16],
    width: usize,
    height: usize,
}

impl<'a> Rgb565Renderer<'a> {
    /// Wrap a row-major RGB565 framebuffer.
    pub fn new(buffer: &'a mut [u16], width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Fill the entire framebuffer with a solid color.
    pub fn clear(&mut self, color: Color) {
        let px = Self::rgb565_from_color(color);
        for pixel in self.buffer.iter_mut() {
            *pixel = px;
        }
    }

    /// Convert an rlvgl color to RGB565.
    pub fn rgb565_from_color(color: Color) -> u16 {
        rgb_to_rgb565(color.0, color.1, color.2)
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }
        self.buffer[y * self.width + x] = color_to_rgb565(color);
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if color.3 == 0 {
            return;
        }
        if color.3 == 255 {
            self.put_pixel(x, y, color);
            return;
        }
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        let (dr, dg, db) = rgb565_to_rgb(self.buffer[idx]);
        let alpha = color.3 as u16;
        let inv = 255 - alpha;
        let r = ((color.0 as u16 * alpha + dr as u16 * inv) / 255) as u8;
        let g = ((color.1 as u16 * alpha + dg as u16 * inv) / 255) as u8;
        let b = ((color.2 as u16 * alpha + db as u16 * inv) / 255) as u8;
        self.buffer[idx] = rgb_to_rgb565(r, g, b);
    }
}

impl Renderer for Rgb565Renderer<'_> {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.width).min(self.width as i32);
        let y1 = (rect.y + rect.height).min(self.height as i32);
        for y in y0..y1 {
            for x in x0..x1 {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blend_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.width).min(self.width as i32);
        let y1 = (rect.y + rect.height).min(self.height as i32);
        for y in y0..y1 {
            for x in x0..x1 {
                self.blend_pixel(x, y, color);
            }
        }
    }

    fn draw_text(&mut self, position: (i32, i32), text: &str, color: Color) {
        FONT_6X10.draw_str(self, position.0, position.1, text, color);
    }
}

fn color_to_rgb565(color: Color) -> u16 {
    Rgb565Renderer::rgb565_from_color(color)
}

fn rgb_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn rgb565_to_rgb(color: u16) -> (u8, u8, u8) {
    let r5 = (color >> 11) & 0x1F;
    let g6 = (color >> 5) & 0x3F;
    let b5 = color & 0x1F;
    let r = ((r5 << 3) | (r5 >> 2)) as u8;
    let g = ((g6 << 2) | (g6 >> 4)) as u8;
    let b = ((b5 << 3) | (b5 >> 2)) as u8;
    (r, g, b)
}
