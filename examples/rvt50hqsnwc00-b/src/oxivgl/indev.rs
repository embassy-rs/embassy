//! Capacitive touch input device for OxivGL on the Riverdi RVT50.

use core::ptr;

use oxivgl_sys::{
    lv_display_t, lv_indev_create, lv_indev_data_t, lv_indev_enable, lv_indev_read,
    lv_indev_set_display, lv_indev_set_read_cb, lv_indev_set_type, lv_indev_t,
    lv_indev_state_t_LV_INDEV_STATE_PRESSED, lv_indev_state_t_LV_INDEV_STATE_RELEASED,
    lv_indev_type_t_LV_INDEV_TYPE_POINTER,
};

/// Latest touch sample written by the UI task before `lv_timer_handler()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TouchSample {
    pub x: i32,
    pub y: i32,
    pub pressed: bool,
}

static mut TOUCH_SAMPLE: TouchSample = TouchSample {
    x: 0,
    y: 0,
    pressed: false,
};

static mut POINTER_INDEV: *mut lv_indev_t = ptr::null_mut();

/// Publish a touch sample for the LVGL pointer input read callback.
pub fn publish_touch(sample: TouchSample) {
    // SAFETY: written from the UI task, read from LVGL indev callback on same task.
    unsafe {
        TOUCH_SAMPLE = sample;
    }
}

/// Create a pointer input device backed by [`publish_touch`] samples.
///
/// # Safety
/// `lv_init()` must have been called. `disp` must be the active LVGL display.
pub unsafe fn register_pointer_indev(disp: *mut lv_display_t) -> *mut lv_indev_t {
    assert!(!disp.is_null(), "LVGL display cannot be null");
    // SAFETY: lv_init() was called by `LvglDriver::init`.
    let indev = unsafe { lv_indev_create() };
    assert!(!indev.is_null(), "lv_indev_create returned NULL");
    unsafe {
        lv_indev_set_type(indev, lv_indev_type_t_LV_INDEV_TYPE_POINTER);
        lv_indev_set_display(indev, disp);
        lv_indev_set_read_cb(indev, Some(pointer_read_cb));
        lv_indev_enable(indev, true);
        POINTER_INDEV = indev;
    }
    indev
}

/// Feed the latest [`publish_touch`] sample into LVGL immediately.
///
/// Call this after [`publish_touch`] and before `lv_timer_handler()` so press /
/// release edges are not missed while LTDC is presenting.
///
/// # Safety
/// [`register_pointer_indev`] must have been called.
pub unsafe fn read_pointer_indev() {
    // SAFETY: UI task only; set in `register_pointer_indev`.
    let indev = unsafe { POINTER_INDEV };
    if !indev.is_null() {
        // SAFETY: `indev` is a valid pointer returned by `lv_indev_create`.
        unsafe {
            lv_indev_read(indev);
        }
    }
}

unsafe extern "C" fn pointer_read_cb(_indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if data.is_null() {
        return;
    }

    // SAFETY: `data` is a valid out-parameter from LVGL for the duration of this callback.
    let out = unsafe { &mut *data };
    // SAFETY: single-task UI loop — see `publish_touch`.
    let sample = unsafe { TOUCH_SAMPLE };

    out.point.x = sample.x;
    out.point.y = sample.y;
    out.state = if sample.pressed {
        lv_indev_state_t_LV_INDEV_STATE_PRESSED
    } else {
        lv_indev_state_t_LV_INDEV_STATE_RELEASED
    };
    out.continue_reading = false;
}
