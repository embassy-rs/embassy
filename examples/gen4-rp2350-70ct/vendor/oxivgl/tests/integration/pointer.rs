// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pointer (touchscreen) input device tests.
//!
//! Validates that [`PointerIndev`] registers a POINTER device wired to its
//! [`PointerState`] (and to a polling closure via `new_with`), that dropping it
//! unregisters cleanly, and that a touch at a widget's coordinate presses that
//! widget — direct-touch navigation, no focus group required.

use crate::common::{fresh_screen, pump};

use oxivgl::enums::ObjState;
use oxivgl::indev::{PointerIndev, PointerState};
use oxivgl::widgets::Button;

// Each test that identifies its device by user-data pointer uses its own
// `'static` state, independent of other tests.
static PT_REG: PointerState = PointerState::new();
static PT_TAP: PointerState = PointerState::new();

/// Find the registered POINTER indev whose user data points at `target`
/// (a `*const c_void`). For the closure form, pass the device's user data.
fn find_pointer_by_userdata(target: *const core::ffi::c_void) -> Option<*mut oxivgl_sys::lv_indev_t> {
    // SAFETY: lv_indev_get_next(NULL) walks the global indev list; get_type and
    // get_user_data are safe on any non-null indev.
    unsafe {
        let mut indev = oxivgl_sys::lv_indev_get_next(core::ptr::null_mut());
        while !indev.is_null() {
            let is_pointer = oxivgl_sys::lv_indev_get_type(indev)
                == oxivgl_sys::lv_indev_type_t_LV_INDEV_TYPE_POINTER;
            if is_pointer
                && oxivgl_sys::lv_indev_get_user_data(indev) as *const core::ffi::c_void == target
            {
                return Some(indev);
            }
            indev = oxivgl_sys::lv_indev_get_next(indev);
        }
        None
    }
}

/// Any registered POINTER indev (first found). Used by the closure test, whose
/// user data is an opaque box pointer rather than the state address.
fn find_any_pointer() -> Option<*mut oxivgl_sys::lv_indev_t> {
    // SAFETY: as above.
    unsafe {
        let mut indev = oxivgl_sys::lv_indev_get_next(core::ptr::null_mut());
        while !indev.is_null() {
            if oxivgl_sys::lv_indev_get_type(indev)
                == oxivgl_sys::lv_indev_type_t_LV_INDEV_TYPE_POINTER
            {
                return Some(indev);
            }
            indev = oxivgl_sys::lv_indev_get_next(indev);
        }
        None
    }
}

#[test]
fn pointer_registers_as_pointer_and_unregisters_on_drop() {
    let _screen = fresh_screen();
    let target = &PT_REG as *const PointerState as *const core::ffi::c_void;
    assert!(find_pointer_by_userdata(target).is_none(), "no pointer before creation");

    let pt = PointerIndev::new(&PT_REG).unwrap();
    pump();
    assert!(
        find_pointer_by_userdata(target).is_some(),
        "PointerIndev::new registers a POINTER device wired to its PointerState",
    );

    drop(pt);
    pump();
    assert!(
        find_pointer_by_userdata(target).is_none(),
        "dropping PointerIndev removes the device from LVGL",
    );
}

#[test]
fn pointer_tap_presses_widget_at_coordinate() {
    let screen = fresh_screen();

    let btn = Button::new(&screen).unwrap();
    btn.pos(50, 50).size(100, 40);
    pump();
    assert!(!btn.has_state(ObjState::PRESSED), "button starts un-pressed");

    let pt = PointerIndev::new(&PT_TAP).unwrap();
    let indev = find_pointer_by_userdata(&PT_TAP as *const PointerState as *const core::ffi::c_void)
        .expect("pointer present");

    // Touch the button's centre and read deterministically (the indev read
    // timer would not fire on the test's near-zero wall clock).
    PT_TAP.touch(100, 70);
    // SAFETY: indev is a live POINTER device returned by find_pointer above.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();
    assert!(
        btn.has_state(ObjState::PRESSED),
        "a touch at the button's coordinate presses it",
    );

    // Release: the widget is no longer pressed.
    PT_TAP.release();
    // SAFETY: same live indev.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();
    assert!(
        !btn.has_state(ObjState::PRESSED),
        "release clears the pressed state",
    );

    drop(pt);
}

#[test]
fn pointer_closure_feeds_coordinates() {
    let screen = fresh_screen();

    let btn = Button::new(&screen).unwrap();
    btn.pos(10, 10).size(80, 30);
    pump();

    // Closure reports a fixed touch inside the button on the first read, then
    // release. A Cell lets the FnMut flip its own state between reads.
    let phase = core::cell::Cell::new(0u8);
    let pt = PointerIndev::new_with(move || {
        let p = phase.get();
        phase.set(p + 1);
        if p == 0 { Some((40, 25)) } else { None }
    })
    .unwrap();

    let indev = find_any_pointer().expect("closure pointer present");

    // First read: closure returns Some → press inside the button.
    // SAFETY: indev is a live POINTER device.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();
    assert!(
        btn.has_state(ObjState::PRESSED),
        "closure-reported touch presses the widget at that coordinate",
    );

    // Second read: closure returns None → release.
    // SAFETY: same live indev.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();
    assert!(!btn.has_state(ObjState::PRESSED), "closure release clears press");

    // Dropping reclaims the boxed closure (no leak — see leak_check tests).
    drop(pt);
    pump();
    assert!(find_any_pointer().is_none(), "closure pointer removed on drop");
}
