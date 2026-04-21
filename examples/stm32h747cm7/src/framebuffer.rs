//! Framebuffer implementing embedded_graphics DrawTarget

use core::ptr::NonNull;

use defmt::warn;
use embassy_stm32::dsihost::panel::DsiPanel;
use mousefood::embedded_graphics::prelude::*;
use mousefood::prelude::Rgb888;

use crate::glass::Glass;

// Framebuffer
pub struct Framebuffer {
    ptr: NonNull<u32>,
    len: usize,
}

#[allow(unused)]
impl Framebuffer {
    pub const WIDTH: usize = Glass::ACTIVE_WIDTH as usize;
    pub const HEIGHT: usize = Glass::ACTIVE_HEIGHT as usize;

    pub const unsafe fn new(ptr: *mut u32, len: usize) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
        }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const u32 {
        self.ptr.as_ptr()
    }

    /// Get a linear index from x,y coordinates
    #[inline]
    fn index_of(x: u32, y: u32) -> Option<usize> {
        if x < Self::WIDTH as u32 && y < Self::HEIGHT as u32 {
            Some(y as usize * Self::WIDTH + x as usize)
        } else {
            None
        }
    }

    /// Set an RGB888 color with alpha set to 0xff
    #[inline]
    fn set(&mut self, index: usize, color: Rgb888) {
        if index < self.len {
            let r = color.r() as u32;
            let g = color.g() as u32;
            let b = color.b() as u32;

            self.as_mut_slice()[index] = (0xff << 24) | (r << 16) | (g << 8) | b;
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Rgb888 {
        let px = self.as_slice()[index];

        let r = ((px >> 16) & 0xFF) as u8;
        let g = ((px >> 8) & 0xFF) as u8;
        let b = (px & 0xFF) as u8;

        Rgb888::new(r, g, b)
    }
}

impl OriginDimensions for &'static mut Framebuffer {
    fn size(&self) -> Size {
        Size {
            width: Framebuffer::WIDTH as u32,
            height: Framebuffer::HEIGHT as u32,
        }
    }
}

impl DrawTarget for &'static mut Framebuffer {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if let Some(index) = Framebuffer::index_of(point.x as u32, point.y as u32) {
                self.set(index, color);
            } else {
                warn!("Pixel coordinates out of range");
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &mousefood::embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(area.points().zip(colors).map(|(pos, color)| Pixel(pos, color)))
    }

    fn fill_solid(
        &mut self,
        area: &mousefood::embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }
}
