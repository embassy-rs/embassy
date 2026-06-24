use crate::common::{fresh_screen, pump};
use oxivgl::anim::{anim_set_x, Anim, AnimHandle};
use oxivgl::widgets::{Arc, Bar, Obj, Slider};

// ── Animation ────────────────────────────────────────────────────────────────

#[test]
fn anim_start_returns_handle() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    let _handle: AnimHandle = a.start();
    pump();
}

#[test]
fn anim_pause_for_during_animation() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 200)
        .set_duration(1000)
        .set_exec_cb(Some(anim_set_x));

    let handle = a.start();
    pump();

    // animation just started (1000 ms), guaranteed still running.
    handle.pause_for(500);
    pump();
}

#[test]
fn anim_start_discard_handle() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    // Discard return value — mirrors anim1/anim2 usage.
    let _ = a.start();
    pump();
}

// ── AnimTimeline ─────────────────────────────────────────────────────────────

#[test]
fn anim_timeline_create_add_start() {
    use oxivgl::anim::{AnimTimeline, Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(300)
        .set_exec_cb(Some(anim_set_x));

    let mut tl = AnimTimeline::new();
    tl.add(0, &a);
    let duration = tl.start();
    assert!(duration > 0);
    pump();
}

#[test]
fn anim_timeline_pause_reverse_progress() {
    use oxivgl::anim::{AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX, Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    let mut tl = AnimTimeline::new();
    tl.add(0, &a);
    tl.start();
    pump();
    tl.pause();
    tl.set_reverse(true);
    tl.set_progress(ANIM_TIMELINE_PROGRESS_MAX / 2);
    pump();
    // Drop cleans up
}

#[test]
fn anim_timeline_drop() {
    use oxivgl::anim::AnimTimeline;
    let _screen = fresh_screen();
    let tl = AnimTimeline::new();
    drop(tl); // should not panic
    pump();
}

// ── Anim builder setters ─────────────────────────────────────────────────────

#[test]
fn anim_path_cb_setters() {
    use oxivgl::anim::{Anim, anim_set_x, anim_path_overshoot, anim_path_ease_in,
        anim_path_ease_out, anim_path_ease_in_out, anim_path_bounce};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(300)
        .set_path_cb(Some(anim_path_overshoot))
        .set_exec_cb(Some(anim_set_x));
    let _ = a.start();
    pump();

    // Test other path callbacks compile and don't panic
    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_in));
    let _ = a2.start();

    let mut a3 = Anim::new();
    a3.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_out));
    let _ = a3.start();

    let mut a4 = Anim::new();
    a4.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_in_out));
    let _ = a4.start();

    let mut a5 = Anim::new();
    a5.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_bounce));
    let _ = a5.start();
    pump();
}

#[test]
fn anim_repeat_and_playback() {
    use oxivgl::anim::{Anim, ANIM_REPEAT_INFINITE, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(200)
        .set_delay(10)
        .set_repeat_count(3)
        .set_repeat_delay(50)
        .set_reverse_duration(200)
        .set_reverse_delay(20)
        .set_exec_cb(Some(anim_set_x));
    let _ = a.start();
    pump();

    // Also test infinite repeat
    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(0, 50)
        .set_duration(100)
        .set_repeat_count(ANIM_REPEAT_INFINITE)
        .set_exec_cb(Some(anim_set_x));
    let _ = a2.start();
    pump();
}

#[test]
fn anim_custom_exec_cb() {
    use oxivgl::anim::{Anim, anim_set_width, anim_set_height};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(50, 100)
        .set_duration(200)
        .set_custom_exec_cb(Some(anim_set_width));
    let _ = a.start();

    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(50, 80)
        .set_duration(200)
        .set_custom_exec_cb(Some(anim_set_height));
    let _ = a2.start();
    pump();
}

#[test]
fn anim_exec_cb_variants() {
    use oxivgl::anim::{Anim, anim_set_size, anim_set_pad_row, anim_set_pad_column,
        anim_set_arc_value, anim_set_bar_value, anim_set_slider_value};
    let screen = fresh_screen();

    // anim_set_size
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);
    let mut a = Anim::new();
    a.set_var(&obj).set_values(50, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_size));
    let _ = a.start();

    // anim_set_pad_row / pad_column
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    let mut a2 = Anim::new();
    a2.set_var(&cont).set_values(0, 20).set_duration(200)
        .set_exec_cb(Some(anim_set_pad_row));
    let _ = a2.start();

    let mut a3 = Anim::new();
    a3.set_var(&cont).set_values(0, 20).set_duration(200)
        .set_exec_cb(Some(anim_set_pad_column));
    let _ = a3.start();

    // anim_set_arc_value
    let arc = Arc::new(&screen).unwrap();
    let mut a4 = Anim::new();
    a4.set_var(&arc).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_arc_value));
    let _ = a4.start();

    // anim_set_bar_value
    let bar = Bar::new(&screen).unwrap();
    let mut a5 = Anim::new();
    a5.set_var(&bar).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_bar_value));
    let _ = a5.start();

    // anim_set_slider_value
    let slider = Slider::new(&screen).unwrap();
    let mut a6 = Anim::new();
    a6.set_var(&slider).set_values(0, 100).set_duration(200)
        .set_custom_exec_cb(Some(anim_set_slider_value));
    let _ = a6.start();

    pump();
}

// ── Anim exec cb variants: anim_set_y, anim_set_translate_x ─────────────────

#[test]
fn anim_exec_cb_y_and_translate_x() {
    use oxivgl::anim::{Anim, anim_set_y, anim_set_translate_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_y));
    let _ = a.start();

    let mut a2 = Anim::new();
    a2.set_var(&obj).set_values(0, 50).set_duration(200)
        .set_exec_cb(Some(anim_set_translate_x));
    let _ = a2.start();

    pump();
}

// ── Anim::set_bezier3_path ────────────────────────────────────────────────────

#[test]
fn anim_set_bezier3_path_no_panic() {
    use std::sync::atomic::AtomicI32;
    use oxivgl::anim::{Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);
    static P1: AtomicI32 = AtomicI32::new(128);
    static P2: AtomicI32 = AtomicI32::new(900);
    let mut anim = Anim::new();
    anim.set_exec_cb(Some(anim_set_x));
    anim.set_var(&obj);
    anim.set_values(0, 100);
    anim.set_duration(500);
    anim.set_bezier3_path(&P1, &P2);
    let _ = anim.start();
    pump();
}

// ── anim_set_scale_rotation ─────────────────────────────────────────────────

#[test]
fn anim_set_scale_rotation_no_crash() {
    use oxivgl::anim::{anim_set_scale_rotation, Anim};
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner)
        .set_range(0, 360)
        .set_total_tick_count(9)
        .set_major_tick_every(1)
        .set_angle_range(360)
        .set_rotation(0);
    scale.size(200, 200);
    let mut a = Anim::new();
    a.set_var(&scale)
        .set_values(0, 360)
        .set_duration(1000)
        .set_exec_cb(Some(anim_set_scale_rotation));
    let _h = a.start();
    pump();
}
