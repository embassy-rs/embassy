#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! Minimal [rlvgl](https://github.com/SoftOboros/rlvgl) demo on the Riverdi RVT50.
//!
//! Draws a title label and a tappable counter button on the 800×480 LTDC panel.
//!
//! ```bash
//! cargo run --bin rlvgl_demo --features rlvgl
//! cargo run --bin rlvgl_demo --features rlvgl,touch
//! ```

extern crate alloc;

use core::mem::MaybeUninit;
#[cfg(feature = "touch")]
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};

use defmt::{info, unwrap};
use embedded_alloc::LlffHeap as Heap;
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rlvgl::demo;
use embassy_rvt50hqsnwc00_b_examples::rlvgl::display::Rgb565Renderer;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use rlvgl::core::event::Event;
use rlvgl::core::widget::Color;
use {defmt_rtt as _, panic_probe as _};

const FB_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const HEAP_SIZE: usize = 256 * 1024;
const BG: Color = Color(16, 28, 48, 255);

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static mut FB1: [u16; FB_PIXELS] = [0; FB_PIXELS];
static mut FB2: [u16; FB_PIXELS] = [0; FB_PIXELS];

#[cfg(feature = "touch")]
mod touch_dbg {
    use super::*;

    pub static X: AtomicU16 = AtomicU16::new(0);
    pub static Y: AtomicU16 = AtomicU16::new(0);
    pub static PRESSED: AtomicBool = AtomicBool::new(false);
    pub static I2C_OK: AtomicBool = AtomicBool::new(false);
    pub static RAW_STATUS: AtomicU8 = AtomicU8::new(0);

    pub fn publish(touch: rvt50_board::TouchPoint) {
        X.store(touch.x, Ordering::Relaxed);
        Y.store(touch.y, Ordering::Relaxed);
        PRESSED.store(touch.pressed, Ordering::Relaxed);
        I2C_OK.store(touch.i2c_ok, Ordering::Relaxed);
        RAW_STATUS.store(touch.raw_status, Ordering::Relaxed);
    }
}

fn init_heap() {
    // SAFETY: called once before any allocation.
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

fn framebuffers() -> (&'static mut [u16; FB_PIXELS], &'static mut [u16; FB_PIXELS]) {
    // SAFETY: static mut framebuffers are only accessed from the UI task.
    unsafe { (&mut FB1, &mut FB2) }
}

fn prefill_background(fb: &mut [u16]) {
    let px = Rgb565Renderer::rgb565_from_color(BG);
    fb.fill(px);
}

async fn setup_ltdc(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);

    let (fb1, fb2) = framebuffers();
    prefill_background(fb1);
    prefill_background(fb2);

    ltdc.init_buffer(LtdcLayer::Layer1, fb1.as_ptr() as *const _);
    ltdc.reload().await.unwrap();
}

fn draw_frame(buffer: &mut [u16], root: &rlvgl::core::WidgetNode) {
    let mut renderer = Rgb565Renderer::new(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    // The root container paints the full-screen background; no separate clear needed.
    root.draw(&mut renderer);
}

async fn present(
    ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    buffer: *const u16,
) {
    ltdc.set_buffer(LtdcLayer::Layer1, buffer as *const _)
        .await
        .unwrap();
}

#[cfg(not(feature = "touch"))]
async fn render_loop(mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) -> ! {
    setup_ltdc(&mut ltdc).await;

    let root = demo::build_demo(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);
    let (fb1, fb2) = framebuffers();
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
async fn render_loop_touch(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    mut i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) -> ! {
    setup_ltdc(&mut ltdc).await;

    let root = demo::build_demo(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);
    let (fb1, fb2) = framebuffers();
    let mut draw_fb2 = true;
    let mut touch_was_pressed = false;
    let mut last_x = 0i32;
    let mut last_y = 0i32;

    loop {
        // Same order as the former `lvgl_touch` demo: sample touch, process
        // input, then redraw and present.
        let touch = rvt50_board::read_touch(&mut i2c);
        touch_dbg::publish(touch);

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

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!("RVT50 rlvgl demo ({}x{})", DISPLAY_WIDTH, DISPLAY_HEIGHT);

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    spawner.spawn(unwrap!(heartbeat_info_task()));

    #[cfg(feature = "touch")]
    {
        let rvt50_board::DisplayResources { ltdc, i2c, touch_int: _ } =
            rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(touch_info_task()));
        spawner.spawn(unwrap!(ui_touch_task(ltdc, i2c)));
    }

    #[cfg(not(feature = "touch"))]
    {
        let rvt50_board::DisplayResources { ltdc } = rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(ui_task(ltdc)));
    }

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn heartbeat_info_task() -> ! {
    loop {
        info!("rlvgl demo heartbeat");
        Timer::after_secs(5).await;
    }
}

#[cfg(feature = "touch")]
#[embassy_executor::task]
async fn touch_info_task() -> ! {
    let mut last_pressed = false;
    let mut last_i2c_ok = false;
    let mut last_raw = 0u8;
    let mut last_x = 0u16;
    let mut last_y = 0u16;

    loop {
        let pressed = touch_dbg::PRESSED.load(Ordering::Relaxed);
        let i2c_ok = touch_dbg::I2C_OK.load(Ordering::Relaxed);
        let raw = touch_dbg::RAW_STATUS.load(Ordering::Relaxed);
        let x = touch_dbg::X.load(Ordering::Relaxed);
        let y = touch_dbg::Y.load(Ordering::Relaxed);

        if pressed != last_pressed
            || i2c_ok != last_i2c_ok
            || raw != last_raw
            || (pressed && (x != last_x || y != last_y))
        {
            info!(
                "touch sample i2c_ok={} raw=0x{:02x} pressed={} x={} y={}",
                i2c_ok,
                raw,
                pressed,
                x,
                y
            );
            last_pressed = pressed;
            last_i2c_ok = i2c_ok;
            last_raw = raw;
            last_x = x;
            last_y = y;
        }

        Timer::after_millis(100).await;
    }
}

#[cfg(not(feature = "touch"))]
#[embassy_executor::task]
async fn ui_task(ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) -> ! {
    render_loop(ltdc).await
}

#[cfg(feature = "touch")]
#[embassy_executor::task]
async fn ui_touch_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) -> ! {
    render_loop_touch(ltdc, i2c).await
}
