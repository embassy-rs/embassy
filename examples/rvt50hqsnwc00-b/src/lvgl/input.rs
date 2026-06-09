//! Capacitive touch input for LVGL pointer device.
//!
//! Built on top of the safe [`Pointer::register`](lvgl::input_device::pointer::Pointer)
//! handler from [lv_binding_rust](https://github.com/lvgl/lv_binding_rust). Touch
//! coordinates are produced by the board I2C driver and stored in atomics so the
//! handler closure can read them without locks.

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use embedded_graphics::geometry::Point;
use lvgl::input_device::pointer::{Pointer, PointerInputData};
use lvgl::input_device::{BufferStatus, InputDriver};
use lvgl::{Display, LvResult};

static TOUCH_X: AtomicU16 = AtomicU16::new(0);
static TOUCH_Y: AtomicU16 = AtomicU16::new(0);
static TOUCH_PRESSED: AtomicBool = AtomicBool::new(false);

/// Update touch state from the board I2C driver (call each frame).
pub fn set_touch(x: u16, y: u16, pressed: bool) {
    TOUCH_X.store(x, Ordering::Relaxed);
    TOUCH_Y.store(y, Ordering::Relaxed);
    TOUCH_PRESSED.store(pressed, Ordering::Relaxed);
}

fn read_touch() -> BufferStatus {
    let x = TOUCH_X.load(Ordering::Relaxed) as i32;
    let y = TOUCH_Y.load(Ordering::Relaxed) as i32;
    let data = PointerInputData::Touch(Point::new(x, y));
    if TOUCH_PRESSED.load(Ordering::Relaxed) {
        data.pressed().once()
    } else {
        data.released().once()
    }
}

/// LVGL pointer input device backed by the static touch state above.
pub struct Rvt50Touch {
    pub inner: Pointer,
}

impl Rvt50Touch {
    pub fn register(display: &Display) -> LvResult<Self> {
        let inner = Pointer::register(read_touch, display)?;
        Ok(Self { inner })
    }
}
