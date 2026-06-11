//! Capacitive touch input for OxivGL on the Riverdi RVT50.
//!
//! All LVGL FFI stays inside this module — application code uses [`TouchInput`] only.
//!
//! Shared state is held in safe sync primitives instead of `static mut`:
//! - [`struct@TOUCH_SAMPLE`] — a critical-section [`Mutex`] around a [`Cell`], so
//!   [`TouchInput::publish`] needs **no `unsafe`** and stays sound even if the
//!   sample is ever published from another task or ISR.
//! - [`POINTER_INDEV`] — an [`AtomicPtr`] to the LVGL indev; loads/stores are
//!   safe, `unsafe` remains only at the LVGL FFI call boundary.

use core::cell::Cell;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

use defmt::{debug, info, warn};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use oxivgl::driver::LvglDriver;
use oxivgl_sys::{
    lv_display_get_screen_prev, lv_indev_create, lv_indev_data_t, lv_indev_enable, lv_indev_get_active_obj,
    lv_indev_get_display, lv_indev_get_point, lv_indev_get_read_cb, lv_indev_get_state,
    lv_indev_mode_t_LV_INDEV_MODE_EVENT, lv_indev_read, lv_indev_set_display, lv_indev_set_mode, lv_indev_set_read_cb,
    lv_indev_set_type, lv_indev_state_t_LV_INDEV_STATE_PRESSED, lv_indev_state_t_LV_INDEV_STATE_RELEASED, lv_indev_t,
    lv_indev_type_t_LV_INDEV_TYPE_POINTER, lv_point_t,
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

/// Latest published sample, read back by [`pointer_read_cb`].
///
/// A critical-section mutex makes publish/read race-free without `unsafe`.
static TOUCH_SAMPLE: Mutex<CriticalSectionRawMutex, Cell<TouchSample>> = Mutex::new(Cell::new(TouchSample {
    x: 0,
    y: 0,
    pressed: false,
}));

/// LVGL pointer indev handle, set once in [`TouchInput::register`].
static POINTER_INDEV: AtomicPtr<lv_indev_t> = AtomicPtr::new(ptr::null_mut());

impl TouchInput {
    /// Create the LVGL pointer indev and bind it to the LTDC display.
    pub fn register() -> Self {
        let disp = lvgl_display();
        assert!(!disp.is_null(), "LVGL display must be initialised first");

        // SAFETY: `lv_init()` completed in `LvglDriver::init`; display is valid.
        let indev = unsafe {
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

            indev
        };

        POINTER_INDEV.store(indev, Ordering::Release);

        Self { registered: true }
    }

    /// Store the latest board touch sample for the LVGL read callback.
    ///
    /// Entirely safe: the sample lives behind a critical-section mutex.
    pub fn publish(&self, sample: TouchSample) {
        assert!(self.registered, "TouchInput::register() was not called");
        TOUCH_SAMPLE.lock(|cell| cell.set(sample));
    }

    /// Push the published sample into LVGL when the display is not animating.
    ///
    /// LVGL drops reads while `disp->prev_scr != NULL` (screen transition).
    /// Returns `true` when `lv_indev_read()` actually ran.
    pub fn sync_read(&self) -> bool {
        assert!(self.registered, "TouchInput::register() was not called");
        let indev = POINTER_INDEV.load(Ordering::Acquire);
        if indev.is_null() {
            return false;
        }

        let disp = lvgl_display();
        // SAFETY: FFI only — `disp` and `indev` are valid (created during init,
        // never freed); UI task is the sole LVGL caller.
        unsafe {
            if !lv_display_get_screen_prev(disp).is_null() {
                debug!("oxivgl indev read deferred (screen transition)");
                return false;
            }
            lv_indev_read(indev);
        }
        true
    }

    /// Publish, read indev (before and after timer), run LVGL timers.
    pub fn feed(&self, driver: &LvglDriver, sample: TouchSample, hit_btn: Option<usize>) {
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
        let indev = POINTER_INDEV.load(Ordering::Acquire);
        if indev.is_null() {
            return;
        }

        let mut pt = lv_point_t { x: 0, y: 0 };
        // SAFETY: FFI only — `indev` is valid (see `sync_read`); `pt` is a live
        // out-parameter for the duration of the call.
        let (active, state) = unsafe {
            lv_indev_get_point(indev, &mut pt);
            (lv_indev_get_active_obj(), lv_indev_get_state(indev))
        };

        touch_dbg::publish_indev(active, hit_btn);

        if sample.pressed {
            info!(
                "oxivgl indev pressed state={} pt=({},{}) active_obj={:08x} layout_hit={:?} sample=({},{})",
                state, pt.x, pt.y, active as u32, hit_btn, sample.x, sample.y
            );
        }
    }
}

unsafe extern "C" fn pointer_read_cb(_indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    // SAFETY: `data` is a valid out-parameter from LVGL for the duration of this callback.
    let Some(out) = (unsafe { data.as_mut() }) else {
        return;
    };

    let sample = TOUCH_SAMPLE.lock(Cell::get);

    out.point.x = sample.x;
    out.point.y = sample.y;
    out.state = if sample.pressed {
        lv_indev_state_t_LV_INDEV_STATE_PRESSED
    } else {
        lv_indev_state_t_LV_INDEV_STATE_RELEASED
    };
    out.continue_reading = false;
}
