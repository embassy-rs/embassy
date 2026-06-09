#![no_std]
#![no_main]

//! Minimal LVGL "buttons on screen" demo for the Riverdi RVT50HQSNWC00-B.
//!
//! Three coloured buttons (`A`, `B`, `C`) on the active screen, with a
//! status label that updates from the `Clicked` event handler. Built end to
//! end against the safe [`lv_binding_rust`](https://github.com/lvgl/lv_binding_rust)
//! API (vendored master under `vendor/lv_binding_rust/`).
//!
//! ```bash
//! sudo apt install gcc-arm-none-eabi libnewlib-dev   # or pacman / brew
//! source scripts/lvgl-env.sh                         # bash / zsh
//! cargo run --bin lvgl_buttons --features lvgl,touch
//! ```

extern crate alloc;

#[cfg(not(all(feature = "lvgl", feature = "touch")))]
compile_error!("lvgl_buttons requires --features lvgl,touch");

use alloc::boxed::Box;
use core::sync::atomic::{AtomicU8, Ordering};
use core::time::Duration as CoreDuration;

use cstr_core::{CStr, CString};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board;
use embassy_stm32::i2c::I2c;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use embedded_graphics::geometry::Point;
use lvgl::input_device::pointer::{Pointer, PointerInputData};
use lvgl::input_device::{BufferStatus, InputDriver};
use lvgl::style::Style;
use lvgl::widgets::{Btn, Label};
use lvgl::{
    Align, Color, Display, DisplayRefresh, DrawBuffer, Event, NativeObject, Part, Widget, init as lvgl_init,
    task_handler, tick_inc,
};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const W: u16 = rvt50_board::DISPLAY_WIDTH as u16;
const H: u16 = rvt50_board::DISPLAY_HEIGHT as u16;
const FB_PIXELS: usize = W as usize * H as usize;
const DRAW_LINES: usize = 10;
const DRAW_BUF_PIXELS: usize = W as usize * DRAW_LINES;
const FRAME: Duration = Duration::from_millis(5);

static FB1: StaticCell<[u16; FB_PIXELS]> = StaticCell::new();

/// Touch state shared between the I2C reader and the LVGL input handler.
mod touch {
    use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

    pub static X: AtomicU16 = AtomicU16::new(0);
    pub static Y: AtomicU16 = AtomicU16::new(0);
    pub static PRESSED: AtomicBool = AtomicBool::new(false);

    pub fn set(x: u16, y: u16, pressed: bool) {
        X.store(x, Ordering::Relaxed);
        Y.store(y, Ordering::Relaxed);
        PRESSED.store(pressed, Ordering::Relaxed);
    }
}

/// `0` = no click, `1..=3` = button A/B/C clicked. Read by the UI task.
static LAST_CLICKED: AtomicU8 = AtomicU8::new(0);

fn read_touch() -> BufferStatus {
    let x = touch::X.load(Ordering::Relaxed) as i32;
    let y = touch::Y.load(Ordering::Relaxed) as i32;
    let data = PointerInputData::Touch(Point::new(x, y));
    if touch::PRESSED.load(Ordering::Relaxed) {
        data.pressed().once()
    } else {
        data.released().once()
    }
}

fn flush_rgb565(fb: *mut u16, refresh: &DisplayRefresh<DRAW_BUF_PIXELS>) {
    let area = &refresh.area;
    let line_w = (area.x2 - area.x1 + 1) as usize;
    let count = line_w * (area.y2 - area.y1 + 1) as usize;
    for (i, color) in refresh.colors.iter().enumerate().take(count) {
        let x = area.x1 as usize + i % line_w;
        let y = area.y1 as usize + i / line_w;
        let r = u16::from(color.r()) & 0x1F;
        let g = u16::from(color.g()) & 0x3F;
        let b = u16::from(color.b()) & 0x1F;
        let pixel = (r << 11) | (g << 5) | b;
        // SAFETY: `fb` is a `[u16; FB_PIXELS]` framebuffer owned by the LTDC
        // task; LVGL's flush area stays inside `(W, H)`.
        unsafe { fb.add(y * W as usize + x).write(pixel) };
    }
}

/// Mint a `&'static mut Style` from a builder closure. `add_style` borrows
/// styles for the lifetime of the widget; for our `'static` UI we leak.
fn make_style(f: impl FnOnce(&mut Style)) -> &'static mut Style {
    let mut s = Style::default();
    f(&mut s);
    Box::leak(Box::new(s))
}

fn cstr(s: &str) -> &'static CStr {
    Box::leak(Box::new(CString::new(s).unwrap())).as_c_str()
}

#[embassy_executor::task]
async fn ui_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    mut i2c: I2c<'static, Blocking, embassy_stm32::i2c::Master>,
) {
    info!("LVGL buttons demo {}x{}", W, H);

    ltdc.init_layer(
        &LtdcLayerConfig {
            pixel_format: ltdc::PixelFormat::Rgb565,
            layer: LtdcLayer::Layer1,
            window_x0: 0,
            window_x1: W as _,
            window_y0: 0,
            window_y1: H as _,
        },
        None,
    );

    let fb = FB1.init([0; FB_PIXELS]);
    let fb_ptr = fb.as_mut_ptr();

    // ---- LVGL wiring ------------------------------------------------------
    lvgl_init();
    let display = Display::register(
        DrawBuffer::<DRAW_BUF_PIXELS>::default(),
        W as u32,
        H as u32,
        move |refresh| flush_rgb565(fb_ptr, refresh),
    )
    .expect("display register");
    let _touch = Pointer::register(read_touch, &display).expect("pointer register");

    // ---- Widgets ----------------------------------------------------------
    let mut screen = display.get_scr_act().expect("scr_act");
    screen.add_style(
        Part::Main,
        make_style(|s| s.set_bg_color(Color::from_rgb((0x10, 0x18, 0x28)))),
    );

    let mut title = Label::create(&mut screen).expect("title");
    title.add_style(
        Part::Main,
        make_style(|s| s.set_text_color(Color::from_rgb((0xE8, 0xEE, 0xF4)))),
    );
    title.set_text(cstr("LVGL buttons on Embassy"));
    title.set_align(Align::TopMid, 0, 24);

    let mut status = Label::create(&mut screen).expect("status");
    status.add_style(
        Part::Main,
        make_style(|s| s.set_text_color(Color::from_rgb((0x90, 0xA0, 0xB0)))),
    );
    status.set_text(cstr("tap a button"));
    status.set_align(Align::BottomMid, 0, -24);

    // Three buttons across the middle of the screen.
    spawn_button(&mut screen, "A", -260, (0xFF, 0x55, 0x55), 1);
    spawn_button(&mut screen, "B", 0, (0x55, 0xCC, 0x55), 2);
    spawn_button(&mut screen, "C", 260, (0x55, 0x88, 0xFF), 3);

    // ---- Frame loop -------------------------------------------------------
    ltdc.set_buffer(LtdcLayer::Layer1, fb_ptr as *const _).await.unwrap();

    let mut last_seen = 0u8;
    loop {
        let t = rvt50_board::read_touch(&mut i2c);
        touch::set(t.x, t.y, t.pressed);

        tick_inc(CoreDuration::from_millis(5));
        task_handler();

        let clicked = LAST_CLICKED.load(Ordering::Relaxed);
        if clicked != last_seen && clicked != 0 {
            let txt = match clicked {
                1 => cstr("A clicked"),
                2 => cstr("B clicked"),
                _ => cstr("C clicked"),
            };
            status.set_text(txt);
            last_seen = clicked;
        }

        ltdc.set_buffer(LtdcLayer::Layer1, fb_ptr as *const _).await.unwrap();
        Timer::after(FRAME).await;
    }
}

fn spawn_button<'a, P: NativeObject>(parent: &'a mut P, label: &str, x_offset: i32, rgb: (u8, u8, u8), id: u8) {
    let mut button = Btn::create(parent).expect("btn");
    button.add_style(
        Part::Main,
        make_style(|s| {
            s.set_bg_color(Color::from_rgb(rgb));
            s.set_radius(12);
            s.set_border_width(0);
        }),
    );
    button.set_size(140, 80);
    button.set_align(Align::Center, x_offset as i32 / 2, 0);

    let mut text = Label::create(&mut button).expect("btn label");
    text.add_style(
        Part::Main,
        make_style(|s| s.set_text_color(Color::from_rgb((0x10, 0x18, 0x28)))),
    );
    text.set_text(cstr(label));
    text.set_align(Align::Center, 0, 0);

    button
        .on_event(move |_btn, ev| {
            if matches!(ev, Event::Clicked) {
                LAST_CLICKED.store(id, Ordering::Relaxed);
            }
        })
        .expect("on_event");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50 — LVGL buttons demo");

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let rvt50_board::DisplayResources { ltdc, i2c } = rvt50_board::init_display(p).await;
    spawner.spawn(unwrap!(ui_task(ltdc, i2c)));

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
