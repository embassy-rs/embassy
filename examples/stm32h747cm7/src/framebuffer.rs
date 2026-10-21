//! ARGB8888 framebuffer drawing a color bar test pattern

use core::ptr::NonNull;

use embassy_stm32::dsihost::panel::DsiPanel;

use crate::glass::Glass;

/// Color bars in ARGB8888, left to right.
const BARS: [u32; 8] = [
    0xFFFF_FFFF, // white
    0xFFFF_FF00, // yellow
    0xFF00_FFFF, // cyan
    0xFF00_FF00, // green
    0xFFFF_00FF, // magenta
    0xFFFF_0000, // red
    0xFF00_00FF, // blue
    0xFF00_0000, // black
];

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

    /// Fill the whole framebuffer with vertical color bars.
    pub fn draw_bars(&mut self) {
        for y in 0..Self::HEIGHT {
            self.draw_bars_row(y, false);
        }
    }

    /// Draw row `y` of the color bars, with the colors inverted if `inverted` is set.
    pub fn draw_bars_row(&mut self, y: usize, inverted: bool) {
        let row = &mut self.as_mut_slice()[y * Self::WIDTH..][..Self::WIDTH];
        for (x, px) in row.iter_mut().enumerate() {
            let color = BARS[x * BARS.len() / Self::WIDTH];
            *px = if inverted { color ^ 0x00FF_FFFF } else { color };
        }
    }
}
