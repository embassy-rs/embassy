// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared test infrastructure for LVGL integration and leak tests.
//!
//! Provides:
//! - Single-thread enforcement (panics if tests run in parallel)
//! - LVGL driver init (shared instance, host SDL2 dummy backend)
//! - `fresh_screen()` for test isolation
//! - `pump()` for layout/render pass

use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};

use oxivgl::driver::LvglDriver;
use oxivgl::widgets::Screen;

static INIT: Once = Once::new();
static mut DRIVER: Option<LvglDriver> = None;

/// Thread ID of the first caller — used to enforce single-threaded access.
static INIT_THREAD: AtomicUsize = AtomicUsize::new(0);

/// Initialise LVGL once. Panics if:
/// - `SDL_VIDEODRIVER` is not set
/// - Called from a different thread than the first caller (detects parallel test runs)
pub fn ensure_init() {
    let tid = thread_id();
    let prev = INIT_THREAD.compare_exchange(0, tid, Ordering::SeqCst, Ordering::SeqCst);
    match prev {
        Ok(_) => {} // first caller, store our thread ID
        Err(first_tid) => {
            assert_eq!(
                first_tid, tid,
                "LVGL tests must run single-threaded (--test-threads=1). \
                 First init on thread {first_tid}, now called from {tid}."
            );
        }
    }

    INIT.call_once(|| {
        assert!(
            std::env::var("SDL_VIDEODRIVER").is_ok(),
            "SDL_VIDEODRIVER not set — run via: ./run_tests.sh"
        );
        // SAFETY: single-threaded (enforced above).
        unsafe { DRIVER = Some(LvglDriver::init(320, 240)) };
    });
}

/// Get a fresh screen, clearing all widgets from the previous test.
pub fn fresh_screen() -> Screen {
    ensure_init();
    // SAFETY: LVGL initialised; loading a new screen clears the previous one.
    unsafe {
        let new = oxivgl_sys::lv_obj_create(core::ptr::null_mut());
        oxivgl_sys::lv_screen_load(new);
    }
    Screen::active().expect("no active screen after init")
}

/// Pump LVGL timer and force a full layout + refresh pass.
pub fn pump() {
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
    driver.timer_handler();
    unsafe { oxivgl_sys::lv_refr_now(core::ptr::null_mut()) };
}

/// Get a reference to the shared LVGL driver. Panics if not initialised.
pub fn driver() -> &'static LvglDriver {
    ensure_init();
    unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() }
}

fn thread_id() -> usize {
    // Use a thread-local variable's address as a unique thread identifier.
    thread_local! { static ANCHOR: u8 = const { 0 }; }
    ANCHOR.with(|a| a as *const u8 as usize)
}
