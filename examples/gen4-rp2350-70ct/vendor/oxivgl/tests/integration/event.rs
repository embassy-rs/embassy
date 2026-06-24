// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event-related integration tests.

use crate::common::{fresh_screen, pump};

use oxivgl::enums::{EventCode, ObjFlag};
use oxivgl::widgets::{AsLvHandle, Button, Obj};

// ── Event (simulated) ────────────────────────────────────────────────────────

#[test]
fn event_callback_fires() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static FIRED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn cb(_e: *mut oxivgl_sys::lv_event_t) {
        FIRED.store(true, Ordering::SeqCst);
    }

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    // SAFETY: user_data is null (unused); btn outlives the event handler.
    unsafe {
        btn.on_event(
            cb,
            oxivgl::enums::EventCode::CLICKED,
            core::ptr::null_mut(),
        );
    }

    // Simulate a click event
    // SAFETY: btn handle valid, LVGL initialised.
    unsafe {
        oxivgl_sys::lv_obj_send_event(
            btn.lv_handle(),
            oxivgl_sys::lv_event_code_t_LV_EVENT_CLICKED,
            core::ptr::null_mut(),
        );
    }

    assert!(FIRED.load(Ordering::SeqCst), "event callback should fire");
}

#[test]
fn event_bubble_flag() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.add_flag(ObjFlag::EVENT_BUBBLE);
    // No crash = flag set correctly
}

// ── Event ────────────────────────────────────────────────────────────────────

#[test]
fn event_on_callback_receives_event() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNT: AtomicU32 = AtomicU32::new(0);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        assert_eq!(event.code(), EventCode::CLICKED);
        // target() should return a valid obj
        let _target = event.target();
        COUNT.fetch_add(1, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    assert!(COUNT.load(Ordering::SeqCst) > 0);
}

#[test]
fn event_matches() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static MATCHED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    let _btn_handle = btn.lv_handle();
    // Use on_event with raw callback to capture btn_handle
    unsafe extern "C" fn match_cb(_e: *mut oxivgl_sys::lv_event_t) {
        MATCHED.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    unsafe {
        btn.on_event(match_cb, EventCode::CLICKED, core::ptr::null_mut());
    }
    btn.send_event(EventCode::CLICKED);
    assert!(MATCHED.load(Ordering::SeqCst));
}

#[test]
fn event_bubble_and_current_target() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUBBLED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let child = Button::new(&parent).unwrap();
    child.bubble_events();

    // Register handler on parent; event sent to child should bubble.
    parent.on(EventCode::CLICKED, |event| {
        let _current = event.current_target_handle();
        BUBBLED.store(true, Ordering::SeqCst);
    });
    child.send_event(EventCode::CLICKED);
    assert!(BUBBLED.load(Ordering::SeqCst));
}

// ── Event target_style convenience ───────────────────────────────────────────

#[test]
fn event_target_style_bg_color() {
    use oxivgl::style::color_make;
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        event.target_style_bg_color(
            color_make(0xFF, 0x00, 0x00),
            oxivgl::style::Selector::DEFAULT,
        );
    });
    btn.send_event(EventCode::CLICKED);
    pump();
}

// ── Event::draw_task ────────────────────────────────────────────────────────

#[test]
fn event_draw_task_returns_none_for_clicked() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CHECKED: AtomicBool = AtomicBool::new(false);
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        assert!(event.draw_task().is_none());
        CHECKED.store(true, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    pump();
    assert!(CHECKED.load(Ordering::SeqCst));
}

// ── Event::draw_task and Layer ────────────────────────────────────────────────

#[test]
fn event_draw_task_layer_smoke() {
    // send_draw_task_events() enables DRAW_TASK_ADDED on obj.
    // With SDL dummy backend DRAW_TASK_ADDED may or may not fire headlessly;
    // this is a no-panic smoke test.
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.send_draw_task_events();
    obj.on(EventCode::DRAW_TASK_ADDED, |ev| {
        if let Some(task) = ev.draw_task() {
            let _layer = task.layer(); // None or Some — both are fine
            let _area = task.area();
            let _base = task.base();
        }
    });
    pump();
}

#[test]
fn event_layer_returns_none_for_clicked() {
    // Event::layer() only works for draw events, not CLICKED.
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |ev| {
        assert!(ev.layer().is_none());
    });
    btn.send_event(EventCode::CLICKED);
    pump();
}

// ── Event::current_target_handle via static fn ───────────────────────────────

#[test]
fn event_current_target_handle_static() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CHECKED: AtomicBool = AtomicBool::new(false);

    fn check_cb(ev: &oxivgl::event::Event) {
        assert_eq!(ev.code(), EventCode::CLICKED);
        let _current = ev.current_target_handle();
        CHECKED.store(true, Ordering::SeqCst);
    }

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, check_cb);
    btn.send_event(EventCode::CLICKED);
    assert!(CHECKED.load(Ordering::SeqCst));
}

// ── EventCode DEFOCUSED ─────────────────────────────────────────────────────

#[test]
fn event_code_defocused_value() {
    assert_eq!(
        EventCode::DEFOCUSED.0,
        oxivgl_sys::lv_event_code_t_LV_EVENT_DEFOCUSED
    );
}
