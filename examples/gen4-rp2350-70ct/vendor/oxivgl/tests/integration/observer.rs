// SPDX-License-Identifier: MIT OR Apache-2.0
//! Subject / observer integration tests.

use crate::common::{fresh_screen, pump};

use core::ffi::c_void;
use oxivgl::enums::{ObjFlag, ObjState};
use oxivgl::widgets::{
    Arc, AsLvHandle, Dropdown, Label, Obj, Roller, RollerMode, Slider, Subject,
    observer_get_target, observer_get_target_obj, subject_get_group_element, subject_get_int_raw,
};
use oxivgl_sys::{lv_observer_t, lv_subject_t};

// ── Subject / observer ───────────────────────────────────────────────────────

#[test]
fn subject_int_create_get_set() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(28);
    assert_eq!(subject.get_int(), 28);
    subject.set_int(42);
    assert_eq!(subject.get_int(), 42);
}

#[test]
fn subject_int_previous_value() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(10);
    subject.set_int(20);
    assert_eq!(subject.get_int(), 20);
    assert_eq!(subject.get_previous_int(), 10);
}

#[test]
fn subject_int_drop_safe() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(5);
    subject.set_int(99);
    drop(subject); // lv_subject_deinit must not crash
    pump();
}

#[test]
fn slider_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(50);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    pump();
    // Subject drives slider: value should reflect the subject's initial value.
    assert_eq!(subject.get_int(), 50);
}

#[test]
fn label_bind_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(28);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d C");
    pump();
    // No crash = binding established successfully.
    assert_eq!(subject.get_int(), 28);
}

#[test]
fn arc_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(30);
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100);
    arc.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 30);
}

#[test]
fn roller_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(1);
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC", RollerMode::Normal);
    roller.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 1);
}

#[test]
fn dropdown_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(2);
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 2);
}

#[test]
fn subject_add_observer_obj() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    unsafe extern "C" fn dummy_cb(_obs: *mut lv_observer_t, _sub: *mut lv_subject_t) {}
    subject.add_observer_obj(dummy_cb, &obj, core::ptr::null_mut());
    subject.set_int(42);
    pump();
    // No crash = success.
}

#[test]
fn obj_bind_state_if_eq() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.bind_state_if_eq(&subject, ObjState::DISABLED, 1);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED));
    subject.set_int(1);
    pump();
    assert!(obj.has_state(ObjState::DISABLED));
    subject.set_int(0);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED));
}

#[test]
fn obj_bind_state_if_not_eq() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.bind_state_if_not_eq(&subject, ObjState::DISABLED, 1);
    pump();
    assert!(obj.has_state(ObjState::DISABLED)); // 0 != 1 → disabled
    subject.set_int(1);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED)); // 1 == 1 → not disabled
}

#[test]
fn obj_bind_checked() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::CHECKABLE);
    obj.bind_checked(&subject);
    pump();
    assert!(!obj.has_state(ObjState::CHECKED));
    subject.set_int(1);
    pump();
    assert!(obj.has_state(ObjState::CHECKED));
    subject.set_int(0);
    pump();
    assert!(!obj.has_state(ObjState::CHECKED));
}

#[test]
fn subject_new_group() {
    let _screen = fresh_screen();
    let s0 = Subject::new_int(10);
    let s1 = Subject::new_int(20);
    let s2 = Subject::new_int(30);
    let group = Subject::new_group(&[&s0, &s1, &s2]);
    pump();
    // Group subject drops cleanly (deinit must not crash).
    drop(group);
    drop(s2);
    drop(s1);
    drop(s0);
}

#[test]
fn subject_notify() {
    let _screen = fresh_screen();
    static CALL_COUNT: core::sync::atomic::AtomicI32 =
        core::sync::atomic::AtomicI32::new(0);

    unsafe extern "C" fn counting_cb(
        _obs: *mut lv_observer_t,
        _sub: *mut lv_subject_t,
    ) {
        CALL_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    CALL_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
    let subject = Subject::new_int(0);
    subject.add_observer(counting_cb, core::ptr::null_mut());
    // Initial notify fires once when observer is added (LVGL behaviour).
    pump();
    let after_add = CALL_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    // Manual notify must fire the callback again.
    subject.notify();
    pump();
    assert!(
        CALL_COUNT.load(core::sync::atomic::Ordering::Relaxed) > after_add,
        "notify() must trigger observer callback"
    );
}

#[test]
fn subject_get_group_element_values() {
    let _screen = fresh_screen();
    let s0 = Subject::new_int(11);
    let s1 = Subject::new_int(22);
    let group = Subject::new_group(&[&s0, &s1]);
    pump();
    // Retrieve member values through the group element accessor.
    // SAFETY: group is a valid group subject; indices 0 and 1 are in bounds.
    let v0 = unsafe { subject_get_int_raw(subject_get_group_element(group.raw_ptr(), 0)) };
    let v1 = unsafe { subject_get_int_raw(subject_get_group_element(group.raw_ptr(), 1)) };
    assert_eq!(v0, 11);
    assert_eq!(v1, 22);
}

#[test]
fn obj_clean() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(parent.get_child_count(), 2);
    parent.clean();
    pump();
    assert_eq!(parent.get_child_count(), 0);
}

#[test]
fn subject_add_observer_with_target() {
    let _screen = fresh_screen();
    static mut TARGET_VAL: i32 = 0;
    unsafe extern "C" fn cb(observer: *mut lv_observer_t, _subject: *mut lv_subject_t) {
        unsafe {
            // SAFETY: target is &mut TARGET_VAL cast to *mut c_void, set below.
            let target = observer_get_target(observer) as *mut i32;
            *target = 99;
        }
    }
    let subject = Subject::new_int(0);
    // SAFETY: TARGET_VAL is a static mut; single-threaded test, no concurrent access.
    // &raw mut avoids creating an intermediate reference to the mutable static.
    subject.add_observer_with_target(
        cb,
        &raw mut TARGET_VAL as *mut c_void,
        core::ptr::null_mut(),
    );
    subject.set_int(1);
    pump();
    // SAFETY: TARGET_VAL written only by `cb` on the same thread; no concurrent access.
    unsafe { assert_eq!(*(&raw const TARGET_VAL), 99); }
}

#[test]
fn subject_on_change() {
    let _screen = fresh_screen();
    static LAST_VALUE: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);
    let subject = Subject::new_int(0);
    subject.on_change(|v| {
        LAST_VALUE.store(v, core::sync::atomic::Ordering::Relaxed);
    });
    pump();
    subject.set_int(42);
    pump();
    assert_eq!(LAST_VALUE.load(core::sync::atomic::Ordering::Relaxed), 42);
}

#[test]
fn label_bind_text_map() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    label.bind_text_map(&subject, |v| match v {
        1 => "one",
        _ => "zero",
    });
    pump();
    // Change to 1
    subject.set_int(1);
    pump();
    // Label should now say "one" — verify no crash + correct binding.
}

#[test]
fn label_bind_text_map_sets_correct_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    label.bind_text_map(&subject, |v| match v {
        1 => "one",
        _ => "zero",
    });
    pump();
    let text = unsafe {
        let ptr = oxivgl_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "zero");
    subject.set_int(1);
    pump();
    let text = unsafe {
        let ptr = oxivgl_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "one");
}

#[test]
fn observer_get_target_obj_returns_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    static TARGET_HANDLE: core::sync::atomic::AtomicPtr<oxivgl_sys::lv_obj_t> =
        core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
    unsafe extern "C" fn cb(
        obs: *mut oxivgl_sys::lv_observer_t,
        _sub: *mut oxivgl_sys::lv_subject_t,
    ) {
        // SAFETY: obs is a valid observer pointer received from LVGL.
        let ptr = unsafe { observer_get_target_obj(obs) };
        TARGET_HANDLE.store(ptr, core::sync::atomic::Ordering::Relaxed);
    }
    subject.add_observer_obj(cb, &label, core::ptr::null_mut());
    subject.set_int(1);
    pump();
    assert_eq!(
        TARGET_HANDLE.load(core::sync::atomic::Ordering::Relaxed),
        label.lv_handle()
    );
}

#[test]
fn subject_drop_before_widgets() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d");
    pump();
    // Drop subject first — lv_subject_deinit removes observer linkage.
    drop(subject);
    pump();
    // Widgets still alive, just no longer bound.
    drop(slider);
    drop(label);
    pump();
    // No crash = both drop orders are safe.
}

// ── Strengthened bind tests ──────────────────────────────────────────────────

#[test]
fn slider_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(50);
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(0, 100);
    slider.bind_value(&subject);
    pump();
    assert_eq!(slider.get_value(), 50);
    subject.set_int(75);
    pump();
    assert_eq!(slider.get_value(), 75);
}

#[test]
fn arc_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(30);
    let arc = Arc::new(&screen).unwrap();
    arc.bind_value(&subject);
    pump();
    assert_eq!(arc.get_value_raw(), 30);
    subject.set_int(60);
    pump();
    assert_eq!(arc.get_value_raw(), 60);
}

#[test]
fn roller_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(2);
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC\nD", RollerMode::Normal);
    roller.bind_value(&subject);
    pump();
    assert_eq!(roller.get_selected(), 2); // "C"
    subject.set_int(0);
    pump();
    assert_eq!(roller.get_selected(), 0); // "A"
}

#[test]
fn dropdown_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(1);
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.bind_value(&subject);
    pump();
    assert_eq!(dd.get_selected(), 1); // "Y"
    subject.set_int(2);
    pump();
    assert_eq!(dd.get_selected(), 2); // "Z"
}

#[test]
fn label_bind_text_sets_correct_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(28);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d C");
    pump();
    let text = unsafe {
        let ptr = oxivgl_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "28 C");
    subject.set_int(42);
    pump();
    let text = unsafe {
        let ptr = oxivgl_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "42 C");
}

// ── Interaction tests ────────────────────────────────────────────────────────

#[test]
fn subject_group_notifies_on_member_change() {
    let _screen = fresh_screen();
    static FIRE_COUNT: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    unsafe extern "C" fn cb(_obs: *mut lv_observer_t, _sub: *mut lv_subject_t) {
        FIRE_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
    let s1 = Subject::new_int(0);
    let s2 = Subject::new_int(0);
    let group = Subject::new_group(&[&s1, &s2]);
    FIRE_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
    group.add_observer(cb, core::ptr::null_mut());
    pump();
    let after_add = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    // Change member s1 — group observer should fire.
    s1.set_int(10);
    pump();
    let after_s1 = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    assert!(after_s1 > after_add, "group observer must fire when member changes");
    // Change member s2 — group observer should fire again.
    s2.set_int(20);
    pump();
    let after_s2 = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    assert!(after_s2 > after_s1, "group observer must fire when another member changes");
}

#[test]
fn subject_multiple_observers() {
    let _screen = fresh_screen();
    static COUNT_A: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    static COUNT_B: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    let subject = Subject::new_int(0);
    subject.on_change(|_| { COUNT_A.fetch_add(1, core::sync::atomic::Ordering::Relaxed); });
    subject.on_change(|_| { COUNT_B.fetch_add(1, core::sync::atomic::Ordering::Relaxed); });
    COUNT_A.store(0, core::sync::atomic::Ordering::Relaxed);
    COUNT_B.store(0, core::sync::atomic::Ordering::Relaxed);
    subject.set_int(1);
    pump();
    assert!(COUNT_A.load(core::sync::atomic::Ordering::Relaxed) > 0, "first observer must fire");
    assert!(COUNT_B.load(core::sync::atomic::Ordering::Relaxed) > 0, "second observer must fire");
}

#[test]
fn widget_drop_then_subject_set() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    pump();
    drop(slider); // widget drops, LVGL removes observer
    pump();
    subject.set_int(99); // must not crash — no observers left
    pump();
}

#[test]
fn subject_on_change_fires_on_registration() {
    let _screen = fresh_screen();
    static INITIAL: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);
    INITIAL.store(-1, core::sync::atomic::Ordering::Relaxed);
    let subject = Subject::new_int(42);
    subject.on_change(|v| { INITIAL.store(v, core::sync::atomic::Ordering::Relaxed); });
    pump();
    // LVGL fires observers on registration with current value.
    assert_eq!(INITIAL.load(core::sync::atomic::Ordering::Relaxed), 42);
}

#[test]
fn subject_previous_int_tracks_last_change() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(0);
    subject.set_int(1);
    subject.set_int(2);
    subject.set_int(3);
    assert_eq!(subject.get_previous_int(), 2);
    assert_eq!(subject.get_int(), 3);
}

// ── Edge case tests ──────────────────────────────────────────────────────────

#[test]
fn subject_empty_group() {
    let _screen = fresh_screen();
    let group = Subject::new_group(&[]);
    pump();
    group.notify();
    pump();
    // No crash = success.
}
