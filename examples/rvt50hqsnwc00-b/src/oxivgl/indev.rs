//! Capacitive touch input for OxivGL on the Riverdi RVT50.
//!
//! All LVGL FFI stays inside this module — application code uses [`TouchInput`] only.

use core::ptr;

use defmt::{info, warn};
use oxivgl::driver::LvglDriver;
use oxivgl_sys::{
    lv_display_get_screen_prev, lv_indev_create, lv_indev_data_t, lv_indev_enable, lv_indev_get_active_obj,
    lv_indev_get_display, lv_indev_get_point, lv_indev_get_read_cb, lv_indev_get_state, lv_indev_read,
    lv_indev_set_display, lv_indev_set_mode, lv_indev_set_read_cb, lv_indev_set_type, lv_indev_t,
    lv_indev_mode_t_LV_INDEV_MODE_EVENT, lv_indev_state_t_LV_INDEV_STATE_PRESSED,
    lv_indev_state_t_LV_INDEV_STATE_RELEASED, lv_indev_type_t_LV_INDEV_TYPE_POINTER, lv_point_t,
};

use crate::oxivgl::display::lvgl_display;
use crate::oxivgl::touch_dbg;

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
            // EVENT mode: only explicit `lv_indev_read()` feeds samples (read timer
            // is auto-paused by `lv_indev_set_mode` in LVGL 9.5).
            lv_indev_set_mode(indev, lv_indev_mode_t_LV_INDEV_MODE_EVENT);

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

    /// Push the published sample into LVGL when the display is not animating.
    ///
    /// LVGL drops reads while `disp->prev_scr != NULL` (screen transition).
    /// Returns `true` when `lv_indev_read()` actually ran.
    pub fn sync_read(&self) -> bool {
        assert!(self.registered, "TouchInput::register() was not called");
        // SAFETY: UI task only; set in `register`.
        unsafe {
            let indev = POINTER_INDEV;
            if indev.is_null() {
                return false;
            }

            let disp = lvgl_display();
            if !lv_display_get_screen_prev(disp).is_null() {
                return false;
            }

            lv_indev_read(indev);
            true
        }
    }

    /// Publish, read indev (before and after timer), run LVGL timers.
    pub fn feed(
        &self,
        driver: &LvglDriver,
        sample: TouchSample,
        hit_btn: Option<usize>,
    ) {
        self.publish(sample);
        let before = self.sync_read();
        driver.timer_handler();
        let after = self.sync_read();
        if sample.pressed && !before && !after {
            warn!(
                "oxivgl indev read skipped (prev_scr?) sample=({},{})",
                sample.x, sample.y
            );
        }
        if sample.pressed {
            self.log_debug(sample, hit_btn);
        }
    }

    /// Log LVGL pointer indev state and update [`touch_dbg`] atomics.
    pub fn log_debug(&self, sample: TouchSample, hit_btn: Option<usize>) {
        assert!(self.registered, "TouchInput::register() was not called");
        // SAFETY: UI task only; set in `register`.
        unsafe {
            let indev = POINTER_INDEV;
            if indev.is_null() {
                return;
            }

            let active = lv_indev_get_active_obj();
            let state = lv_indev_get_state(indev);
            let mut pt = lv_point_t {
                x: 0,
                y: 0,
            };
            lv_indev_get_point(indev, &mut pt);

            touch_dbg::publish_indev(active, hit_btn);

            if sample.pressed {
                info!(
                    "oxivgl indev pressed state={} pt=({},{}) active_obj={:08x} layout_hit={:?} sample=({},{})",
                    state,
                    pt.x,
                    pt.y,
                    active as u32,
                    hit_btn,
                    sample.x,
                    sample.y
                );
            }
        }
    }
}

unsafe extern "C" fn pointer_read_cb(_indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if data.is_null() {
        return;
    }

    // SAFETY: UI task only — see `TouchInput::publish`.
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
