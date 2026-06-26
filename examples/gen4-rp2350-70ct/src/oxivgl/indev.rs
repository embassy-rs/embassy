//! Capacitive touch input for OxivGL on the gen4-RP2350-70CT.
//!
//! All LVGL FFI stays inside this module — application code uses [`TouchInput`] only.

use core::cell::Cell;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use oxivgl_sys::{
    lv_indev_create, lv_indev_data_t, lv_indev_enable, lv_indev_read, lv_indev_set_display, lv_indev_set_read_cb,
    lv_indev_set_type, lv_indev_state_t_LV_INDEV_STATE_PRESSED, lv_indev_state_t_LV_INDEV_STATE_RELEASED, lv_indev_t,
    lv_indev_type_t_LV_INDEV_TYPE_POINTER,
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
static TOUCH_SAMPLE: Mutex<CriticalSectionRawMutex, Cell<TouchSample>> = Mutex::new(Cell::new(TouchSample {
    x: 0,
    y: 0,
    pressed: false,
}));

/// LVGL pointer indev handle, set once in [`TouchInput::register`].
static POINTER_INDEV: AtomicPtr<lv_indev_t> = AtomicPtr::new(ptr::null_mut());

impl TouchInput {
    /// Create the LVGL pointer indev and bind it to the scan-out display.
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
            // Default mode from `lv_indev_create` is TIMER: `lv_timer_handler` polls
            // the read callback. We publish samples in the UI loop before each tick.
            indev
        };

        POINTER_INDEV.store(indev, Ordering::Release);

        defmt::info!(
            "oxivgl touch indev registered (bindgen state offset={})",
            core::mem::offset_of!(lv_indev_data_t, state)
        );

        Self { registered: true }
    }

    /// Store the latest board touch sample for the LVGL read callback.
    pub fn publish(&self, sample: TouchSample) {
        assert!(self.registered, "TouchInput::register() was not called");
        TOUCH_SAMPLE.lock(|cell| cell.set(sample));
    }

    /// Push the published sample into LVGL (`lv_indev_read`).
    pub fn sync_read(&self) -> bool {
        assert!(self.registered, "TouchInput::register() was not called");
        let indev = POINTER_INDEV.load(Ordering::Acquire);
        if indev.is_null() {
            touch_dbg::bump_sync_skip_null();
            return false;
        }

        // SAFETY: FFI only — `indev` is valid (created during init, never freed).
        unsafe {
            lv_indev_read(indev);
        }
        touch_dbg::bump_sync_ok();
        true
    }

    /// Store a touch sample; LVGL reads it on the next `timer_handler` tick.
    pub fn feed(&self, sample: TouchSample) {
        touch_dbg::bump_fed();
        self.publish(sample);
    }
}

/// LVGL 9.5 `lv_indev_data_t` on Cortex-M (short enums): state @32, point @36,
/// continue_reading @60. Bindgen reports state @48 because it widens gesture enums
/// to u32 — never write through the bindgen struct on 32-bit targets.
#[inline(always)]
unsafe fn write_pointer_data(data: *mut lv_indev_data_t, sample: TouchSample) {
    let state = if sample.pressed {
        lv_indev_state_t_LV_INDEV_STATE_PRESSED as u8
    } else {
        lv_indev_state_t_LV_INDEV_STATE_RELEASED as u8
    };

    // SAFETY: `data` is LVGL's valid out-parameter for the duration of the read callback.
    unsafe {
        let base = data.cast::<u8>();
        core::ptr::write(base.add(32), state);
        core::ptr::write(base.add(36).cast(), sample.x);
        core::ptr::write(base.add(40).cast(), sample.y);
        core::ptr::write(base.add(60), 0);
    }
}

unsafe extern "C" fn pointer_read_cb(_indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    touch_dbg::bump_read_cb();
    let sample = TOUCH_SAMPLE.lock(Cell::get);
    // SAFETY: `data` comes from LVGL's indev read path.
    unsafe {
        write_pointer_data(data, sample);
    }
}
