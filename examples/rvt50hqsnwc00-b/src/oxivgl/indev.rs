//! Capacitive touch input for OxivGL on the Riverdi RVT50.
//!
//! All LVGL FFI stays inside this module — application code uses [`TouchInput`] only.

use core::ptr;

use defmt::info;
use oxivgl_sys::{
    lv_display_get_screen_prev, lv_indev_create, lv_indev_data_t, lv_indev_enable, lv_indev_get_display,
    lv_indev_get_read_cb, lv_indev_read, lv_indev_set_display, lv_indev_set_mode, lv_indev_set_read_cb,
    lv_indev_set_type, lv_indev_t, lv_indev_mode_t_LV_INDEV_MODE_EVENT,
    lv_indev_state_t_LV_INDEV_STATE_PRESSED, lv_indev_state_t_LV_INDEV_STATE_RELEASED,
    lv_indev_type_t_LV_INDEV_TYPE_POINTER, lv_timer_pause, lv_indev_get_read_timer,
};

use crate::oxivgl::display::lvgl_display;

/// Latest touch sample written by the UI task before [`TouchInput::sync_read`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchSample {
    /// Horizontal coordinate in display pixels.
    pub x: i32,
    /// Vertical coordinate in display pixels.
    pub y: i32,
    /// `true` while the panel reports active contact.
    pub pressed: bool,
}

/// LVGL pointer input device fed by [`TouchInput`].
///
/// Register once after the widget tree is built (see [`TouchInput::register`]).
pub struct TouchInput {
    registered: bool,
}

static mut TOUCH_SAMPLE: TouchSample = TouchSample {
    x: 0,
    y: 0,
    pressed: false,
};

static mut POINTER_INDEV: *mut lv_indev_t = ptr::null_mut();

impl TouchInput {
    /// Create the LVGL pointer indev and bind it to the LTDC display.
    ///
    /// Call after widgets are created so the active screen and layout exist.
    pub fn register() -> Self {
        let disp = lvgl_display();
        assert!(!disp.is_null(), "LVGL display must be initialised first");

        // SAFETY: `lv_init()` completed in `LvglDriver::init`; display is valid.
        unsafe {
            let indev = lv_indev_create();
            assert!(!indev.is_null(), "lv_indev_create returned NULL");
            lv_indev_set_type(indev, lv_indev_type_t_LV_INDEV_TYPE_POINTER);
            lv_indev_set_display(indev, disp);
            lv_indev_set_read_cb(indev, Some(pointer_read_cb));
            lv_indev_enable(indev, true);
            // Only [`sync_read`] feeds samples — avoids racing the internal read timer.
            lv_indev_set_mode(indev, lv_indev_mode_t_LV_INDEV_MODE_EVENT);
            let read_timer = lv_indev_get_read_timer(indev);
            if !read_timer.is_null() {
                lv_timer_pause(read_timer);
            }

            let linked_disp = lv_indev_get_display(indev);
            let prev_scr = lv_display_get_screen_prev(disp);
            let read_cb = lv_indev_get_read_cb(indev);
            info!(
                "oxivgl indev registered disp_ok={} prev_scr={} read_cb={}",
                !linked_disp.is_null(),
                !prev_scr.is_null(),
                read_cb.is_some()
            );

            POINTER_INDEV = indev;
        }

        Self { registered: true }
    }

    /// Store the latest board touch sample for the LVGL read callback.
    pub fn publish(&self, sample: TouchSample) {
        assert!(self.registered, "TouchInput::register() was not called");
        // SAFETY: UI task only.
        unsafe {
            TOUCH_SAMPLE = sample;
        }
    }

    /// Feed the published sample into LVGL.
    ///
    /// Call **after** `LvglDriver::timer_handler()` so refresh/screen state is settled
    /// (LVGL blocks input while `disp->prev_scr` is active during refresh).
    pub fn sync_read(&self) {
        assert!(self.registered, "TouchInput::register() was not called");
        // SAFETY: UI task only; set in `register`.
        unsafe {
            read_pointer_indev();
        }
    }

    /// Publish a sample and read it into LVGL immediately (used during present batches).
    pub fn pump(&self, sample: TouchSample) {
        self.publish(sample);
        self.sync_read();
    }
}

/// Feed the latest sample into LVGL (called from [`TouchInput::sync_read`]).
unsafe fn read_pointer_indev() {
    // SAFETY: UI task only; set in `register`.
    let indev = unsafe { POINTER_INDEV };
    if indev.is_null() {
        return;
    }

    // SAFETY: `indev` is a valid pointer returned by `lv_indev_create`.
    unsafe {
        lv_indev_read(indev);
    }
}

unsafe extern "C" fn pointer_read_cb(_indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if data.is_null() {
        return;
    }

    // SAFETY: single-task UI loop — see `TouchInput::publish`.
    let sample = unsafe { TOUCH_SAMPLE };
    // SAFETY: `data` is a valid out-parameter from LVGL for the duration of this callback.
    let out = unsafe { &mut *data };

    out.point.x = sample.x;
    out.point.y = sample.y;
    out.state = if sample.pressed {
        lv_indev_state_t_LV_INDEV_STATE_PRESSED
    } else {
        lv_indev_state_t_LV_INDEV_STATE_RELEASED
    };
    out.continue_reading = false;
}
