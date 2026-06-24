use crate::common::{fresh_screen, pump};

use oxivgl::style::Selector;
use oxivgl::widgets::{Label, Obj, WidgetError};

#[test]
fn timer_handler_callable() {
    // pump() calls timer_handler internally — just verify it doesn't panic.
    let _screen = fresh_screen();
    pump();
}

// ── Error handling ───────────────────────────────────────────────────────────

#[test]
fn widget_error_display() {
    let err = WidgetError::LvglNullPointer;
    let msg = format!("{err}");
    assert!(msg.contains("NULL"), "error msg: {msg}");
}

#[test]
fn widget_error_format_error() {
    let err = WidgetError::FormatError(core::fmt::Error);
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

// ── SDL builder ──────────────────────────────────────────────────────────────

#[test]
fn sdl_builder_api() {
    // Verify builder API compiles and chains. Can't call build() since
    // LVGL is already initialised by ensure_init().
    let _builder = oxivgl::driver::LvglDriver::sdl(320, 240)
        .title(c"test")
        .mouse(false);
}

// ── Indev ───────────────────────────────────────────────────────────────────

#[test]
fn indev_active_returns_none_without_input() {
    use oxivgl::indev::Indev;
    let _screen = fresh_screen();
    // With SDL dummy driver, there may or may not be an input device.
    // Just verify the call doesn't crash.
    let _ = Indev::active();
}

#[test]
fn indev_get_vect_without_input() {
    use oxivgl::indev::Indev;
    let _screen = fresh_screen();
    pump();
    if let Some(indev) = Indev::active() {
        let _vect = indev.get_vect();
        let _streak = indev.short_click_streak();
    }
    // No input device in headless mode — just verify no panic.
}

// ── Transform scale_x / scale_y ─────────────────────────────────────────────

#[test]
fn transform_scale_xy() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 60);
    obj.style_transform_scale_x(256, Selector::DEFAULT);
    obj.style_transform_scale_y(128, Selector::DEFAULT);
    pump();
}

// ── Translation ─────────────────────────────────────────────────────────────

use oxivgl::translation::{self, StaticCStr as S};

static TRANS_LANGS: [S; 3] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL];
static TRANS_TAGS: [S; 2] = [S::from_cstr(c"hello"), S::NULL];
static TRANS_VALUES: [S; 2] = [S::from_cstr(c"Hello"), S::from_cstr(c"Hallo")];

#[test]
fn translation_add_and_set_language() {
    let _screen = fresh_screen();
    translation::add_static(&TRANS_LANGS, &TRANS_TAGS, &TRANS_VALUES);
    translation::set_language(c"en");
    let lang = translation::get_language();
    assert_eq!(lang, Some(c"en".as_ref()));
}

#[test]
fn translation_tag_on_label() {
    let screen = fresh_screen();
    translation::add_static(&TRANS_LANGS, &TRANS_TAGS, &TRANS_VALUES);
    translation::set_language(c"en");
    let lbl = Label::new(&screen).unwrap();
    lbl.set_translation_tag("hello");
    pump();
}

// ── Label set_translation_tag ───────────────────────────────────────────────

#[test]
fn label_set_translation_tag_no_crash() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.set_translation_tag("test");
    pump();
}
