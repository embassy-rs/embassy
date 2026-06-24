use crate::common::{fresh_screen, pump};

use oxivgl::enums::{ObjFlag, ObjState};
use oxivgl::layout::{FlexAlign, FlexFlow};
use oxivgl::style::{
    Selector, color_make,
};
use oxivgl::widgets::{
    AnimImg, ArcLabel, ArcLabelDir, Button, Checkbox, Image, Label, Led, Line, Screen, Textarea,
    ValueLabel,
};

// ── Screen ───────────────────────────────────────────────────────────────────

#[test]
fn screen_style_methods() {
    let screen = fresh_screen();
    screen
        .bg_color(0x06080f)
        .bg_opa(255)
        .pad_top(6)
        .pad_bottom(6)
        .pad_left(4)
        .pad_right(4)
        .text_color(0xFFFFFF);
    pump();
}

#[test]
fn screen_remove_scrollable() {
    let screen = fresh_screen();
    screen.remove_scrollable();
    pump();
}

#[test]
fn screen_flex_layout() {
    let screen = fresh_screen();
    screen.set_flex_flow(FlexFlow::Column);
    screen.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);
    pump();
}

// ── Label ────────────────────────────────────────────────────────────────────

#[test]
fn label_create() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("Hello");
    pump();
    assert!(label.get_width() > 0);
    assert!(label.get_height() > 0);
}

#[test]
fn label_text_chaining() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("test").center();
    pump();
}

// ── Button ───────────────────────────────────────────────────────────────────

#[test]
fn button_create_with_label() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    let lbl = Label::new(&btn).unwrap();
    lbl.text("Click me").center();
    pump();
    assert!(btn.get_width() > 0);
}

#[test]
fn button_checkable_toggle() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.add_flag(ObjFlag::CHECKABLE);

    assert!(!btn.has_state(ObjState::CHECKED));
    btn.add_state(ObjState::CHECKED);
    assert!(btn.has_state(ObjState::CHECKED));
}

// ── Led ──────────────────────────────────────────────────────────────────────

#[test]
fn led_create() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    pump();
    assert!(led.get_width() > 0);
}

// ── Line ─────────────────────────────────────────────────────────────────────

#[test]
fn line_create_and_set_points() {
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    static POINTS: [oxivgl::widgets::lv_point_precise_t; 3] = [
        oxivgl::widgets::lv_point_precise_t { x: 0.0, y: 0.0 },
        oxivgl::widgets::lv_point_precise_t { x: 50.0, y: 30.0 },
        oxivgl::widgets::lv_point_precise_t { x: 100.0, y: 0.0 },
    ];
    line.set_points(&POINTS);
    pump();
}

// ── ValueLabel ───────────────────────────────────────────────────────────────

#[test]
fn value_label_format() {
    let screen = fresh_screen();
    let mut vl = ValueLabel::new(&screen, "V").unwrap();
    vl.set_value(14.2).unwrap();
    pump();
    assert!(vl.get_width() > 0);
}

// ── Image ────────────────────────────────────────────────────────────────────

oxivgl::image_declare!(img_cogwheel_argb);

#[test]
fn image_set_src_static() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src(img_cogwheel_argb());
    pump();
    assert!(img.get_width() > 0, "image should have non-zero width");
}

// ── Led (extended) ───────────────────────────────────────────────────────────

#[test]
fn led_on_off_brightness() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.on();
    pump();
    led.set_brightness(128);
    pump();
    led.off();
    pump();
}

#[test]
fn led_color() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_color(color_make(0xFF, 0x00, 0x00));
    pump();
}

// ── Label (extended) ─────────────────────────────────────────────────────────

#[test]
fn label_long_mode() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("A very long text that might need scrolling or wrapping");
    label.set_long_mode(LabelLongMode::Wrap);
    label.width(100);
    pump();
    assert!(label.get_height() > 0);
}

// ── Image setters ────────────────────────────────────────────────────────────

#[test]
fn image_rotation_scale_pivot() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_rotation(450)   // 45 degrees
        .set_scale(512)      // 2x
        .set_pivot(16, 16);
    pump();
}

#[test]
fn image_offset_and_inner_align() {
    use oxivgl::widgets::ImageAlign;
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_offset_y(10)
        .set_inner_align(ImageAlign::Center);
    pump();

    // Test other align variants
    img.set_inner_align(ImageAlign::TopLeft);
    img.set_inner_align(ImageAlign::BottomRight);
    img.set_inner_align(ImageAlign::Stretch);
    img.set_inner_align(ImageAlign::Tile);
    pump();
}

// ── Image::set_src_symbol ─────────────────────────────────────────────────────

#[test]
fn image_set_src_symbol() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src_symbol(&oxivgl::symbols::SETTINGS);
    pump();
}

// ── Long-text setters (issue #105: no silent truncation at 127 bytes) ─────────

/// A 300-byte string — well past the old fixed 128-byte stack buffer.
const LONG_TEXT: &str = "The quick brown fox jumps over the lazy dog. \
    The quick brown fox jumps over the lazy dog. \
    The quick brown fox jumps over the lazy dog. \
    The quick brown fox jumps over the lazy dog. \
    The quick brown fox jumps over the lazy dog. \
    The quick brown fox jumps over the lazy dog.";

#[test]
fn label_long_text_no_panic() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    assert!(LONG_TEXT.len() > 127);
    lbl.text(LONG_TEXT);
    pump();
    assert!(lbl.get_width() > 0);
}

#[test]
fn textarea_long_text_roundtrips_in_full() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    assert!(LONG_TEXT.len() > 127);
    ta.set_text(LONG_TEXT);
    pump();
    // The whole string must survive — not the first 127 bytes (issue #105).
    assert_eq!(ta.get_text(), Some(LONG_TEXT));
}

#[test]
fn checkbox_long_text_roundtrips_in_full() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    assert!(LONG_TEXT.len() > 127);
    cb.text(LONG_TEXT);
    pump();
    assert_eq!(cb.get_text(), Some(LONG_TEXT));
}

#[test]
fn label_long_mode_variants() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("overflow").width(30);
    lbl.set_long_mode(LabelLongMode::Dots);
    pump();
    lbl.set_long_mode(LabelLongMode::Scroll);
    pump();
    lbl.set_long_mode(LabelLongMode::ScrollCircular);
    pump();
    lbl.set_long_mode(LabelLongMode::Clip);
    pump();
}

// ── Child::Debug ──────────────────────────────────────────────────────────────

#[test]
fn child_debug_fmt() {
    let screen = fresh_screen();
    let c = Label::new(&screen).unwrap();
    let s = format!("{c:?}");
    assert!(!s.is_empty());
}

// ── ValueLabel::Debug ─────────────────────────────────────────────────────────

#[test]
fn value_label_debug_fmt() {
    let screen = fresh_screen();
    let vl = ValueLabel::new(&screen, "A").unwrap();
    let s = format!("{vl:?}");
    assert!(!s.is_empty());
}

// ── AnimImg ──────────────────────────────────────────────────────────────────

#[repr(transparent)]
struct SyncPtr(*const core::ffi::c_void);
unsafe impl Sync for SyncPtr {}

// Declare extern symbol at module scope so we can take its address in a static.
// The same symbol is also referenced by the img_cogwheel_argb() function
// generated by image_declare! above — both refer to the same linker symbol.
mod animimg_frames {
    unsafe extern "C" {
        #[allow(non_upper_case_globals)]
        pub static img_cogwheel_argb: oxivgl::widgets::lv_image_dsc_t;
    }
    pub static FRAMES: [super::SyncPtr; 2] = [
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
    ];
}

fn animimg_frame_ptrs() -> &'static [*const core::ffi::c_void] {
    // SAFETY: SyncPtr is #[repr(transparent)] over *const c_void.
    unsafe {
        core::slice::from_raw_parts(
            animimg_frames::FRAMES.as_ptr().cast(),
            animimg_frames::FRAMES.len(),
        )
    }
}

#[test]
fn animimg_create() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg.size(100, 100).center();
    pump();
}

#[test]
fn animimg_set_src_and_start() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg.center();
    animimg
        .set_src(animimg_frame_ptrs())
        .set_duration(1000)
        .set_repeat_count(oxivgl::anim::ANIM_REPEAT_INFINITE)
        .start();
    pump();
    assert_eq!(animimg.get_src_count(), 2);
    assert_eq!(animimg.get_duration(), 1000);
}

#[test]
fn animimg_getters() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg
        .set_src(animimg_frame_ptrs())
        .set_duration(500)
        .set_repeat_count(3);
    assert_eq!(animimg.get_duration(), 500);
    assert_eq!(animimg.get_repeat_count(), 3);
    assert_eq!(animimg.get_src_count(), 2);
    pump();
}

// ── Label — RTL and CJK fonts ───────────────────────────────────────────────

#[test]
fn label_bidi_rtl() {
    use oxivgl::widgets::BaseDir;
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("RTL test");
    label.style_base_dir(BaseDir::Rtl, Selector::DEFAULT);
    label.font(oxivgl::fonts::DEJAVU_16_PERSIAN_HEBREW);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn label_cjk_font() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("CJK");
    label.font(oxivgl::fonts::SOURCE_HAN_SANS_SC_16_CJK);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn fixed_width_font_label() {
    use oxivgl::fonts::{FixedWidthFont, MONTSERRAT_20};
    static MONO: FixedWidthFont = FixedWidthFont::new();
    let screen = fresh_screen();
    let mono_font = MONO.init(MONTSERRAT_20, 20);
    let label = Label::new(&screen).unwrap();
    label.text_font(mono_font);
    label.text("0123.Wabc");
    pump();
    assert!(label.get_width() > 0);
}

// ── Text letter space ───────────────────────────────────────────────────────

#[test]
fn text_letter_space() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("Spaced");
    lbl.style_text_letter_space(5, Selector::DEFAULT);
    pump();
}

// ── ArcLabel ─────────────────────────────────────────────────────────────────

#[test]
fn arclabel_create() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"Test");
    al.set_radius(50);
    al.set_angle_start(0.0);
    al.set_angle_size(180.0);
    pump();
}

#[test]
fn arclabel_direction() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"CCW");
    al.set_dir(ArcLabelDir::CounterClockwise);
    pump();
}

#[test]
fn arclabel_set_text_static() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"Static text");
    pump();
}

#[test]
fn arclabel_set_radius() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_radius(80);
    pump();
}

#[test]
fn arclabel_set_angle_start() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_angle_start(45.0);
    pump();
}

#[test]
fn arclabel_set_angle_size() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_angle_size(270.0);
    pump();
}

// ── Screen::layer_top ───────────────────────────────────────────────────────

#[test]
fn screen_layer_top() {
    let _screen = fresh_screen();
    let top = Screen::layer_top();
    assert!(!top.handle().is_null());
}

// ── Image getters ────────────────────────────────────────────────────────────

#[test]
fn image_get_rotation() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_rotation(450); // 45.0 degrees
    pump();
    assert_eq!(img.get_rotation(), 450);
}

#[test]
fn image_get_scale() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_scale(512); // 2.0x
    pump();
    assert_eq!(img.get_scale(), 512);
}

#[test]
fn image_get_scale_x() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // set_scale sets both scale_x and scale_y to the same value
    img.set_scale(384);
    pump();
    assert_eq!(img.get_scale_x(), 384);
}

#[test]
fn image_get_offset_x() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // No set_offset_x in the wrapper; just verify the getter returns a value
    pump();
    let _x = img.get_offset_x(); // no panic
}

#[test]
fn image_get_offset_y() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_offset_y(20);
    pump();
    assert_eq!(img.get_offset_y(), 20);
}

#[test]
fn image_get_inner_align() {
    use oxivgl::widgets::ImageAlign;
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_inner_align(ImageAlign::Center);
    pump();
    let align = img.get_inner_align();
    // ImageAlign::Center is variant 5 in lv_image_align_t
    assert!(align > 0);
}

#[test]
fn image_get_antialias() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // No set_antialias wrapper; just verify the getter returns without panic.
    pump();
    let _aa = img.get_antialias();
}

#[test]
fn image_get_src_width_height() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src(img_cogwheel_argb());
    pump();
    // With a real image loaded, src_width/height should be > 0
    assert!(img.get_src_width() > 0);
    assert!(img.get_src_height() > 0);
}

// ── Label getters ────────────────────────────────────────────────────────────

#[test]
fn label_get_long_mode() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.set_long_mode(LabelLongMode::Dots);
    pump();
    assert!(matches!(lbl.get_long_mode(), LabelLongMode::Dots));
}

#[test]
fn label_get_recolor() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("test");
    pump();
    // No set_recolor wrapper; just verify the getter returns without panic.
    let _recolor = lbl.get_recolor();
}

#[test]
fn label_get_text_selection_start() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("Hello");
    pump();
    // No selection initially; LVGL returns LV_DRAW_LABEL_NO_TXT_SEL (0xFFFF) when nothing selected
    let start = lbl.get_text_selection_start();
    let _ = start; // just verify no panic
}

// ── Led getters ──────────────────────────────────────────────────────────────

#[test]
fn led_get_brightness() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_brightness(200);
    pump();
    assert_eq!(led.get_brightness(), 200);
}

#[test]
fn led_get_color() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_color(color_make(255, 0, 0));
    pump();
    let _c = led.get_color(); // just verify no panic
}

// ── Line getters ─────────────────────────────────────────────────────────────

#[test]
fn line_get_point_count() {
    use oxivgl::widgets::lv_point_precise_t;
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    static PTS: [lv_point_precise_t; 3] = [
        lv_point_precise_t { x: 0.0, y: 0.0 },
        lv_point_precise_t { x: 50.0, y: 50.0 },
        lv_point_precise_t { x: 100.0, y: 0.0 },
    ];
    line.set_points(&PTS);
    pump();
    assert_eq!(line.get_point_count(), 3);
}

#[test]
fn line_get_y_invert() {
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    // No set_y_invert wrapper; default is false
    pump();
    assert!(!line.get_y_invert());
}

