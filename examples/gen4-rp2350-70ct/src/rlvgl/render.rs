//! Draw a widget tree into the PSRAM RGB565 framebuffer.

use core::slice;

use rlvgl::core::WidgetNode;
use rlvgl::platform::blit::{BlitterRenderer, PixelFmt, Surface};
use rlvgl::platform::CpuBlitter;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const DIRTY_RECTS: usize = 64;

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
