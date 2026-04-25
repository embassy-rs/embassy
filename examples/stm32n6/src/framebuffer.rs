//! In-memory framebuffer that implements `embedded_graphics_core::DrawTarget` for `Rgb565`.
//!
//! Wraps a `&'a mut [u16]` owned by the caller so the same slice can be handed
//! to `Ltdc::set_buffer` as `*const ()`.
//!
//! Only depends on `embedded-graphics-core` — users add `embedded-graphics` themselves
//! if they want primitives, text, etc.

use embedded_graphics_core::Pixel;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::prelude::RawData;
use embedded_graphics_core::primitives::Rectangle;

pub struct Framebuffer<'a> {
    buf: &'a mut [u16],
    width: u16,
    height: u16,
}

impl<'a> Framebuffer<'a> {
    /// Panics if `buf.len() != width * height`.
    pub fn new(buf: &'a mut [u16], width: u16, height: u16) -> Self {
        assert_eq!(buf.len(), width as usize * height as usize);
        Self { buf, width, height }
    }

    pub fn as_ptr(&self) -> *const () {
        self.buf.as_ptr() as *const ()
    }

    pub fn fill(&mut self, color: Rgb565) {
        self.buf.fill(RawU16::from(color).into_inner());
    }

    /// Raw pixel slice for fast per-pixel writes (e.g. gradients). Caller writes
    /// `RawU16::from(color).into_inner()` values directly.
    pub fn pixels_mut(&mut self) -> (&mut [u16], u16) {
        (self.buf, self.width)
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

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let w = self.width as i32;
        let h = self.height as i32;
        let stride = self.width as usize;
        let buf = &mut *self.buf;
        for Pixel(p, color) in pixels {
            if p.x >= 0 && p.y >= 0 && p.x < w && p.y < h {
                let idx = p.y as usize * stride + p.x as usize;
                // SAFETY: bounds checked by the four comparisons above.
                unsafe { *buf.get_unchecked_mut(idx) = RawU16::from(color).into_inner() };
            }
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let rect = area.intersection(&self.bounding_box());
        if rect.size.width == 0 || rect.size.height == 0 {
            return Ok(());
        }
        let raw = RawU16::from(color).into_inner();
        let stride = self.width as usize;
        let x0 = rect.top_left.x as usize;
        let y0 = rect.top_left.y as usize;
        let w = rect.size.width as usize;
        for y in y0..y0 + rect.size.height as usize {
            let start = y * stride + x0;
            self.buf[start..start + w].fill(raw);
        }
        Ok(())
    }

    /// Row-major write used by `Text`, `Image`, and `Styled<Rectangle, ...>` fills.
    /// Faster than the default `draw_iter` fallback because the common "fully inside"
    /// case skips per-pixel bounds checking entirely.
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let stride = self.width as usize;
        let fb_w = self.width as i32;
        let fb_h = self.height as i32;
        let ax = area.top_left.x;
        let ay = area.top_left.y;
        let aw = area.size.width as i32;
        let ah = area.size.height as i32;

        // Fast path: `area` is fully inside the framebuffer — no clipping needed.
        if ax >= 0 && ay >= 0 && ax + aw <= fb_w && ay + ah <= fb_h {
            let x0 = ax as usize;
            let y0 = ay as usize;
            let w = aw as usize;
            let h = ah as usize;
            let buf = &mut *self.buf;
            let mut iter = colors.into_iter();
            for y in y0..y0 + h {
                let row_base = y * stride + x0;
                for x in 0..w {
                    match iter.next() {
                        Some(color) => unsafe {
                            *buf.get_unchecked_mut(row_base + x) = RawU16::from(color).into_inner();
                        },
                        None => return Ok(()),
                    }
                }
            }
            return Ok(());
        }

        // Slow path: partial clipping. Fall back to draw_iter with explicit Pixels so
        // we don't have to reimplement clipping logic.
        self.draw_iter(colors.into_iter().enumerate().take((aw * ah) as usize).map(|(i, c)| {
            let x = ax + (i as i32) % aw;
            let y = ay + (i as i32) / aw;
            Pixel(embedded_graphics_core::geometry::Point::new(x, y), c)
        }))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill(color);
        Ok(())
    }
}
