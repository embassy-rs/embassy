//! Shared LTDC render loop for rlvgl demos on the Riverdi RVT50.

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::mem::MaybeUninit;

#[cfg(feature = "touch")]
use defmt::info;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use rlvgl::core::WidgetNode;
use rlvgl::core::event::Event;
use rlvgl::core::widget::Color;

use crate::rlvgl::display::Rgb565Renderer;
#[cfg(feature = "touch")]
use crate::rvt50_board;
use crate::rvt50_board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub const FB_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub const HEAP_SIZE: usize = 256 * 1024;
pub const BG: Color = Color(16, 28, 48, 255);

pub fn init_heap(heap: &'static Heap, heap_mem: &'static mut MaybeUninit<[u8; HEAP_SIZE]>) {
    // SAFETY: called once before any allocation.
    unsafe {
        heap.init(heap_mem.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

pub fn framebuffers(
    fb1: &'static mut [u16; FB_PIXELS],
    fb2: &'static mut [u16; FB_PIXELS],
) -> (&'static mut [u16; FB_PIXELS], &'static mut [u16; FB_PIXELS]) {
    (fb1, fb2)
}

pub fn prefill_background(fb: &mut [u16]) {
    let px = Rgb565Renderer::rgb565_from_color(BG);
    fb.fill(px);
}

pub async fn init_ltdc_layer(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);
}

pub fn draw_frame(buffer: &mut [u16], root: &WidgetNode) {
    let mut renderer = Rgb565Renderer::new(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    root.draw(&mut renderer);
}

pub async fn present(
    ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    buffer: *const u16,
) {
    ltdc.set_buffer(LtdcLayer::Layer1, buffer as *const _)
        .await
        .unwrap();
}

pub async fn render_loop(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    root: Rc<RefCell<WidgetNode>>,
    fb1: &'static mut [u16; FB_PIXELS],
    fb2: &'static mut [u16; FB_PIXELS],
) -> ! {
    init_ltdc_layer(&mut ltdc).await;
    prefill_background(fb1);
    prefill_background(fb2);
    ltdc.init_buffer(LtdcLayer::Layer1, fb1.as_ptr() as *const _);
    ltdc.reload().await.unwrap();

    let mut draw_fb2 = true;

    loop {
        if draw_fb2 {
            draw_frame(fb2, &root.borrow());
            present(&mut ltdc, fb2.as_ptr()).await;
        } else {
            draw_frame(fb1, &root.borrow());
            present(&mut ltdc, fb1.as_ptr()).await;
        }

        draw_fb2 = !draw_fb2;
        root.borrow_mut().dispatch_event(&Event::Tick);
        Timer::after(Duration::from_millis(16)).await;
    }
}

#[cfg(feature = "touch")]
pub async fn render_loop_touch(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    mut i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
    root: Rc<RefCell<WidgetNode>>,
    fb1: &'static mut [u16; FB_PIXELS],
    fb2: &'static mut [u16; FB_PIXELS],
) -> ! {
    init_ltdc_layer(&mut ltdc).await;
    prefill_background(fb1);
    prefill_background(fb2);
    ltdc.init_buffer(LtdcLayer::Layer1, fb1.as_ptr() as *const _);
    ltdc.reload().await.unwrap();

    let mut draw_fb2 = true;
    let mut touch_was_pressed = false;
    let mut last_x = 0i32;
    let mut last_y = 0i32;

    loop {
        let touch = rvt50_board::read_touch(&mut i2c);

        if touch.pressed && !touch_was_pressed {
            let x = touch.x as i32;
            let y = touch.y as i32;
            info!("touch PressDown x={} y={}", x, y);
            let handled = root
                .borrow_mut()
                .dispatch_event(&Event::PressDown { x, y });
            info!("touch PressDown handled={}", handled);
            last_x = x;
            last_y = y;
        } else if touch.pressed {
            let x = touch.x as i32;
            let y = touch.y as i32;
            if x != last_x || y != last_y {
                root.borrow_mut()
                    .dispatch_event(&Event::PointerMove { x, y });
                last_x = x;
                last_y = y;
            }
        } else if !touch.pressed && touch_was_pressed {
            info!("touch PressRelease x={} y={}", last_x, last_y);
            let handled = root.borrow_mut().dispatch_event(&Event::PressRelease {
                x: last_x,
                y: last_y,
            });
            info!("touch PressRelease handled={}", handled);
        }
        touch_was_pressed = touch.pressed;

        root.borrow_mut().dispatch_event(&Event::Tick);

        if draw_fb2 {
            draw_frame(fb2, &root.borrow());
            present(&mut ltdc, fb2.as_ptr()).await;
        } else {
            draw_frame(fb1, &root.borrow());
            present(&mut ltdc, fb1.as_ptr()).await;
        }

        draw_fb2 = !draw_fb2;
        Timer::after(Duration::from_millis(5)).await;
    }
}

/// Type alias for the heap used by rlvgl demos.
pub type Heap = embedded_alloc::LlffHeap;
