//! Capacitive touch input for LVGL pointer device.
//!
//! Follows the `g_touch_data` + `touch_read_cb` pattern from `lvgl-port/port.c` and
//! [lv_binding_rust](https://github.com/lvgl/lv_binding_rust) pointer drivers.

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use lvgl::input_device::pointer::Pointer;
use lvgl::input_device::InputDriver;
use lvgl::Display;
use lvgl_sys;

static TOUCH_X: AtomicU16 = AtomicU16::new(0);
static TOUCH_Y: AtomicU16 = AtomicU16::new(0);
static TOUCH_PRESSED: AtomicBool = AtomicBool::new(false);

/// Update touch state from the board I2C driver (call each frame).
pub fn set_touch(x: u16, y: u16, pressed: bool) {
    TOUCH_X.store(x, Ordering::Relaxed);
    TOUCH_Y.store(y, Ordering::Relaxed);
    TOUCH_PRESSED.store(pressed, Ordering::Relaxed);
}

pub struct Rvt50Touch {
    pub inner: Pointer,
}

impl Rvt50Touch {
    pub fn register(display: &Display) -> lvgl::LvResult<Self> {
        let pointer = unsafe { Pointer::new_raw(Some(touch_read_cb), None, display)? };
        Ok(Self { inner: pointer })
    }
}

unsafe extern "C" fn touch_read_cb(
    _drv: *mut lvgl_sys::lv_indev_drv_t,
    data: *mut lvgl_sys::lv_indev_data_t,
) {
    unsafe {
        (*data).point.x = TOUCH_X.load(Ordering::Relaxed) as lvgl_sys::lv_coord_t;
        (*data).point.y = TOUCH_Y.load(Ordering::Relaxed) as lvgl_sys::lv_coord_t;
        (*data).state = if TOUCH_PRESSED.load(Ordering::Relaxed) {
            lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_PRESSED
        } else {
            lvgl_sys::lv_indev_state_t_LV_INDEV_STATE_RELEASED
        };
    }
}
