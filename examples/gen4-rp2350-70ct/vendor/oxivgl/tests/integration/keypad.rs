// SPDX-License-Identifier: MIT OR Apache-2.0
//! Keypad input device tests.
//!
//! Validates that [`KeypadIndev`] registers a KEYPAD device wired to its
//! [`KeypadState`], that dropping it unregisters cleanly, and that pressing a
//! navigation key moves focus within a bound [`Group`].

use crate::common::{fresh_screen, pump};

use oxivgl::enums::{Key, ObjState};
use oxivgl::group::Group;
use oxivgl::indev::{KeypadIndev, KeypadState};
use oxivgl::widgets::Button;

// Each test uses its own `'static` state so `find_keypad` can identify the
// device it created by user-data pointer, independent of other tests.
static KP_REG: KeypadState = KeypadState::new();
static KP_FOCUS: KeypadState = KeypadState::new();
static KP_SEND: KeypadState = KeypadState::new();
static KP_BURST: KeypadState = KeypadState::new();
static KP_REPEAT: KeypadState = KeypadState::new();

/// Find the registered KEYPAD indev whose user data points at `state`.
fn find_keypad(state: &KeypadState) -> Option<*mut oxivgl_sys::lv_indev_t> {
    let target = state as *const KeypadState as *const core::ffi::c_void;
    // SAFETY: lv_indev_get_next(NULL) walks the global indev list; get_type
    // and get_user_data are safe on any non-null indev.
    unsafe {
        let mut indev = oxivgl_sys::lv_indev_get_next(core::ptr::null_mut());
        while !indev.is_null() {
            let is_keypad = oxivgl_sys::lv_indev_get_type(indev)
                == oxivgl_sys::lv_indev_type_t_LV_INDEV_TYPE_KEYPAD;
            if is_keypad
                && oxivgl_sys::lv_indev_get_user_data(indev) as *const core::ffi::c_void == target
            {
                return Some(indev);
            }
            indev = oxivgl_sys::lv_indev_get_next(indev);
        }
        None
    }
}

#[test]
fn keypad_registers_as_keypad_and_unregisters_on_drop() {
    let _screen = fresh_screen();
    assert!(find_keypad(&KP_REG).is_none(), "no keypad before creation");

    let kp = KeypadIndev::new(&KP_REG).unwrap();
    pump();
    assert!(
        find_keypad(&KP_REG).is_some(),
        "KeypadIndev::new registers a KEYPAD device wired to its KeypadState",
    );

    drop(kp);
    pump();
    assert!(
        find_keypad(&KP_REG).is_none(),
        "dropping KeypadIndev removes the device from LVGL",
    );
}

#[test]
fn keypad_next_key_moves_group_focus() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let first = Button::new(&screen).unwrap();
    let second = Button::new(&screen).unwrap();
    group.add_obj(&first);
    group.add_obj(&second);

    let kp = KeypadIndev::new(&KP_FOCUS).unwrap();
    kp.set_group(&group);
    pump();

    // The first added object is focused initially.
    assert!(first.has_state(ObjState::FOCUSED), "first item focused on add");

    let indev = find_keypad(&KP_FOCUS).expect("keypad present");

    // Drive a single NEXT press/release through the read_cb deterministically
    // (the indev read timer would not fire on the test's near-zero wall clock).
    KP_FOCUS.press(Key::NEXT);
    // SAFETY: indev is a live KEYPAD device returned by find_keypad above.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    KP_FOCUS.release();
    // SAFETY: same live indev.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();

    assert!(
        second.has_state(ObjState::FOCUSED),
        "NEXT advances focus to the second group member",
    );
    assert!(
        !first.has_state(ObjState::FOCUSED),
        "first member loses focus after NEXT",
    );

    drop(kp);
}

#[test]
fn keypad_send_one_shot_moves_focus_once() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let a = Button::new(&screen).unwrap();
    let b = Button::new(&screen).unwrap();
    let c = Button::new(&screen).unwrap();
    group.add_obj(&a);
    group.add_obj(&b);
    group.add_obj(&c);

    // EVENT mode: LVGL never polls; we drain via read().
    let kp = KeypadIndev::new_event(&KP_SEND).unwrap();
    kp.set_group(&group);
    pump();
    assert!(a.has_state(ObjState::FOCUSED), "first item focused on add");

    // One discrete event → exactly one focus step.
    KP_SEND.send(Key::NEXT);
    kp.read();
    pump();
    assert!(b.has_state(ObjState::FOCUSED), "send(NEXT) advances exactly one step");
    assert!(!a.has_state(ObjState::FOCUSED));
    assert!(!c.has_state(ObjState::FOCUSED), "no extra repeat step");

    // A read with nothing queued must NOT advance focus — proves LVGL adds no
    // repeat of its own (the key is never held across reads).
    kp.read();
    pump();
    assert!(b.has_state(ObjState::FOCUSED), "idle read does not repeat");

    drop(kp);
}

#[test]
fn keypad_with_repeat_sets_long_press_timing_and_still_navigates() {
    use core::time::Duration;
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let a = Button::new(&screen).unwrap();
    let b = Button::new(&screen).unwrap();
    group.add_obj(&a);
    group.add_obj(&b);

    let kp = KeypadIndev::new(&KP_REPEAT)
        .unwrap()
        .with_repeat(Duration::from_millis(400), Duration::from_millis(80));
    kp.set_group(&group);
    pump();

    let indev = find_keypad(&KP_REPEAT).expect("keypad present");
    // with_repeat is a thin pass-through to LVGL's long-press timing fields.
    // SAFETY: indev is a live KEYPAD device; the timing fields are plain u16.
    unsafe {
        assert_eq!((*indev).long_press_time, 400, "long_press_time set from `after`");
        assert_eq!(
            (*indev).long_press_repeat_time, 80,
            "long_press_repeat_time set from `every`",
        );
    }

    // The device still works: a held NEXT advances focus once on press.
    assert!(a.has_state(ObjState::FOCUSED), "first item focused on add");
    KP_REPEAT.press(Key::NEXT);
    // SAFETY: same live indev.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    KP_REPEAT.release();
    // SAFETY: same live indev.
    unsafe { oxivgl_sys::lv_indev_read(indev) };
    pump();
    assert!(b.has_state(ObjState::FOCUSED), "NEXT still advances focus with repeat enabled");

    drop(kp);
}

#[test]
fn keypad_send_burst_drains_in_order() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let items: Vec<_> = (0..4).map(|_| Button::new(&screen).unwrap()).collect();
    for it in &items {
        group.add_obj(it);
    }

    let kp = KeypadIndev::new_event(&KP_BURST).unwrap();
    kp.set_group(&group);
    pump();
    assert!(items[0].has_state(ObjState::FOCUSED), "first item focused on add");

    // Three discrete events queued, then one read() — the whole burst drains
    // (ring queue + one-shot release phases), advancing focus by three.
    KP_BURST.send(Key::NEXT);
    KP_BURST.send(Key::NEXT);
    KP_BURST.send(Key::NEXT);
    assert!(KP_BURST.has_pending(), "queue holds the burst until read");
    kp.read();
    pump();
    assert!(!KP_BURST.has_pending(), "read() drained the whole queue");
    assert!(items[3].has_state(ObjState::FOCUSED), "burst of 3 NEXT advanced 3 steps");

    drop(kp);
}
