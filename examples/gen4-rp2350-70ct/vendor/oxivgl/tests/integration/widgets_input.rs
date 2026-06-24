use crate::common::{fresh_screen, pump};

use oxivgl::enums::ObjState;
use oxivgl::widgets::{
    Arc, Bar, BarOrientation, Buttonmatrix, ButtonmatrixCtrl, ButtonmatrixMap, Checkbox, Dropdown, Keyboard, KeyboardMode, Label, Roller, RollerMode, Slider, SliderOrientation, Spinbox, Spinner, Switch, Textarea,
};

// ── Slider ───────────────────────────────────────────────────────────────────

#[test]
fn slider_default_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    assert_eq!(slider.get_value(), 0);
}

#[test]
fn slider_set_get_value() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_value(42);
    assert_eq!(slider.get_value(), 42);
}

#[test]
fn slider_custom_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(-20, 80);
    slider.set_value(30);
    assert_eq!(slider.get_value(), 30);
}

#[test]
fn slider_clamps_to_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_value(200);
    assert_eq!(slider.get_value(), 100);
    slider.set_value(-10);
    assert_eq!(slider.get_value(), 0);
}

// ── Bar ──────────────────────────────────────────────────────────────────────

#[test]
fn bar_set_get_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range(100.0);
    bar.set_value(42.0);
    let v = bar.get_value();
    assert!((v - 42.0).abs() < 0.2, "expected ~42.0, got {v}");
}

#[test]
fn bar_zero_max_returns_zero() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_value(50.0);
    assert_eq!(bar.get_value(), 0.0);
}

// ── Arc ──────────────────────────────────────────────────────────────────────

#[test]
fn arc_set_get_value() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range(150.0);
    arc.set_value(75.0);
    let v = arc.get_value();
    assert!((v - 75.0).abs() < 0.3, "expected ~75.0, got {v}");
}

#[test]
fn arc_zero_max_returns_zero() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_value(50.0);
    assert_eq!(arc.get_value(), 0.0);
}

#[test]
fn arc_rotation_and_angles() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_rotation(135).set_bg_angles(0, 270);
    pump();
}

#[test]
fn arc_gauge_ring() {
    let screen = fresh_screen();
    let arc = Arc::gauge_ring(&screen, 200, 15, 100.0, 0x333333, 0x00FF00, 150, 200).unwrap();
    arc.set_value(50.0);
    pump();
    let v = arc.get_value();
    assert!((v - 50.0).abs() < 0.2, "expected ~50.0, got {v}");
}

// ── Switch ───────────────────────────────────────────────────────────────────

#[test]
fn switch_toggle_state() {
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();

    assert!(!sw.has_state(ObjState::CHECKED));
    sw.add_state(ObjState::CHECKED);
    assert!(sw.has_state(ObjState::CHECKED));
}

// ── Scale ────────────────────────────────────────────────────────────────────

#[test]
fn scale_builder() {
    use oxivgl::widgets::{ScaleBuilder, ScaleMode};
    let screen = fresh_screen();
    let _scale = ScaleBuilder::new(200, ScaleMode::RoundOuter)
        .rotation(135)
        .sweep(270)
        .range_max(100)
        .total_ticks(21)
        .major_every(5)
        .show_labels(false)
        .major_len(12)
        .minor_len(6)
        .major_color(0xFFFFFF)
        .minor_color(0x808080)
        .build(&screen)
        .unwrap();
    pump();
}

// ── Dropdown ─────────────────────────────────────────────────────────────────

#[test]
fn dropdown_set_symbol_static() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC");
    dd.set_symbol(c"▼");
    pump();
}

// ── Checkbox ─────────────────────────────────────────────────────────────────

#[test]
fn checkbox_create_and_text() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Accept terms");
    pump();
    assert!(cb.get_width() > 0);
}

#[test]
fn checkbox_toggle() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Option");
    assert!(!cb.has_state(ObjState::CHECKED));
    cb.add_state(ObjState::CHECKED);
    assert!(cb.has_state(ObjState::CHECKED));
}

// ── Roller ───────────────────────────────────────────────────────────────────

#[test]
fn roller_create_and_options() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("Jan\nFeb\nMar\nApr", RollerMode::Normal);
    roller.set_visible_row_count(3);
    pump();
    assert_eq!(roller.get_selected(), 0);
}

#[test]
fn roller_set_selected() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC", RollerMode::Infinite);
    roller.set_selected(2, false);
    assert_eq!(roller.get_selected(), 2);
}

// ── Dropdown (extended) ──────────────────────────────────────────────────────

#[test]
fn dropdown_options_and_selection() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("Red\nGreen\nBlue");
    dd.set_selected(1);
    assert_eq!(dd.get_selected(), 1);
    pump();
}

#[test]
fn dropdown_direction() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY");
    dd.set_dir(DdDir::Top);
    pump();
}

// ── Bar (extended) ───────────────────────────────────────────────────────────

#[test]
fn bar_mode_range() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100);
    bar.set_mode(BarMode::Range);
    bar.set_value_raw(80, false);
    bar.set_start_value_raw(20, false);
    pump();
}

// ── Slider setters ───────────────────────────────────────────────────────────

#[test]
fn slider_mode_and_range_value() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(0, 200);
    slider.set_mode(SliderMode::Range);
    slider.set_start_value(20);
    slider.set_value(80);
    pump();
    // In range mode, set_start_value and get_left_value exercise the API.
    // Just verify no panic and value is readable.
    let _left = slider.get_left_value();
    assert_eq!(slider.get_value(), 80);
}

#[test]
fn slider_mode_symmetrical() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_mode(SliderMode::Symmetrical);
    slider.set_value(50);
    pump();
}

// ── Switch setters ───────────────────────────────────────────────────────────

#[test]
fn switch_set_orientation() {
    use oxivgl::widgets::SwitchOrientation;
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();
    sw.set_orientation(SwitchOrientation::Horizontal);
    pump();
    sw.set_orientation(SwitchOrientation::Vertical);
    pump();
    sw.set_orientation(SwitchOrientation::Auto);
    pump();
}

// ── Scale (extended) ─────────────────────────────────────────────────────────

#[test]
fn scale_builder_all_setters() {
    use oxivgl::widgets::{ScaleBuilder, ScaleMode};
    let screen = fresh_screen();
    let scale = ScaleBuilder::new(150, ScaleMode::RoundInner)
        .rotation(135)
        .sweep(270)
        .range_max(200)
        .total_ticks(21)
        .major_every(5)
        .show_labels(false)
        .major_len(12)
        .minor_len(6)
        .major_color(0xFF0000)
        .minor_color(0x888888)
        .build(&screen)
        .unwrap();
    pump();
    drop(scale);
}

#[test]
fn scale_section_with_styles() {
    use oxivgl::widgets::ScaleMode;
    use oxivgl::style::{StyleBuilder, palette_main, Palette};
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundOuter)
        .set_total_tick_count(21)
        .set_major_tick_every(5)
        .set_range(0, 100)
        .set_rotation(135)
        .set_angle_range(270)
        .set_label_show(true);
    scale.size(200, 200);

    // Add section with styles
    let mut sb = StyleBuilder::new();
    sb.line_color(palette_main(Palette::Red)).line_width(3);
    let section_style = sb.build();

    let section = scale.add_section();
    section.set_range(75, 100)
        .set_indicator_style(&section_style)
        .set_items_style(&section_style)
        .set_main_style(&section_style);
    pump();
}

#[test]
fn scale_set_text_src() {
    use oxivgl::widgets::ScaleMode;
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom)
        .set_total_tick_count(4)
        .set_major_tick_every(1)
        .set_range(0, 3);
    scale.size(200, 50);
    static LABELS: &oxivgl::widgets::ScaleLabels = oxivgl::scale_labels!(c"Low", c"Med", c"High", c"Max");
    scale.set_text_src(LABELS);
    pump();
}

#[test]
fn scale_tick_lengths() {
    use oxivgl::widgets::{ScaleMode, Part};
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::VerticalLeft)
        .set_total_tick_count(11)
        .set_major_tick_every(5)
        .set_range(0, 100);
    scale.set_tick_length(Part::Items, 8);
    scale.set_tick_length(Part::Indicator, 15);
    pump();
}

// ── Scale::get_major_tick_every ─────────────────────────────────────────────

#[test]
fn scale_get_major_tick_every() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom);
    scale.set_total_tick_count(21);
    scale.set_major_tick_every(5);
    pump();
    assert_eq!(scale.get_major_tick_every(), 5);
}

// ── Dropdown::get_selected_str ──────────────────────────────────────────────

#[test]
fn dropdown_get_selected_str() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("Apple\nBanana\nOrange");
    pump();
    let mut buf = [0u8; 32];
    assert_eq!(dd.get_selected_str(&mut buf), Some("Apple"));
    dd.set_selected(1);
    assert_eq!(dd.get_selected_str(&mut buf), Some("Banana"));
}

// ── Textarea ──────────────────────────────────────────────────────────────────

#[test]
fn textarea_create() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    assert!(ta.get_width() > 0);
}

#[test]
fn textarea_set_get_text() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("Hello");
    assert_eq!(ta.get_text(), Some("Hello"));
}

#[test]
fn textarea_one_line() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_one_line(true);
    ta.set_text("single line");
    pump();
    assert_eq!(ta.get_text(), Some("single line"));
}

#[test]
fn textarea_password_mode() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_password_mode(true);
    ta.set_text("secret");
    pump();
    assert_eq!(ta.get_text(), Some("secret"));
}

#[test]
fn textarea_cursor_pos() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("abc");
    ta.set_cursor_pos(1);
    ta.add_text("X");
    assert_eq!(ta.get_text(), Some("aXbc"));
}

#[test]
fn textarea_max_length() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_max_length(3);
    ta.set_text("");
    ta.add_text("abcdef");
    let text = ta.get_text().unwrap_or("");
    assert!(text.len() <= 3, "max_length should limit to 3 chars, got: {text}");
}

#[test]
fn textarea_add_delete_char() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("");
    ta.add_char('A');
    ta.add_char('B');
    assert_eq!(ta.get_text(), Some("AB"));
    ta.delete_char();
    assert_eq!(ta.get_text(), Some("A"));
}

#[test]
fn textarea_placeholder() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_placeholder_text("Enter text...");
    pump();
}

#[test]
fn textarea_accepted_chars() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_accepted_chars(c"0123456789");
    ta.set_text("");
    ta.add_text("12abc34");
    let text = ta.get_text().unwrap_or("");
    assert!(!text.contains('a'), "should filter non-digit chars, got: {text}");
}

// ── Buttonmatrix ──────────────────────────────────────────────────────────────

#[test]
fn buttonmatrix_create() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    pump();
    assert!(btnm.get_width() > 0);
}

#[test]
fn buttonmatrix_get_selected() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    pump();
    // LVGL returns LV_BTNMATRIX_BTN_NONE (0xFFFF) when nothing is selected.
    let sel = btnm.get_selected_button();
    assert_eq!(sel, 0xFFFF);
}

// ── Keyboard ──────────────────────────────────────────────────────────────────

#[test]
fn keyboard_create() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    pump();
    assert!(kb.get_width() > 0);
}

#[test]
fn keyboard_set_textarea() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_textarea(&ta);
    pump();
}

#[test]
fn keyboard_set_mode() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::Number);
    kb.set_mode(KeyboardMode::TextLower);
    kb.set_mode(KeyboardMode::TextUpper);
    kb.set_mode(KeyboardMode::Special);
    pump();
}

// ── Buttonmatrix set_map / get_button_text ────────────────────────────────────

#[test]
fn buttonmatrix_set_map_and_get_text() {
    use oxivgl::btnmatrix_map;
    use oxivgl::widgets::ButtonmatrixMap;
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    static MAP: &ButtonmatrixMap = btnmatrix_map!(c"X", c"Y", c"Z");
    btnm.set_map(MAP);
    pump();
    assert_eq!(btnm.get_button_text(0), Some("X"));
    assert_eq!(btnm.get_button_text(1), Some("Y"));
    assert_eq!(btnm.get_button_text(2), Some("Z"));
}

#[test]
fn buttonmatrix_get_button_text_oob() {
    use oxivgl::btnmatrix_map;
    use oxivgl::widgets::ButtonmatrixMap;
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    static MAP2: &ButtonmatrixMap = btnmatrix_map!(c"P", c"Q");
    btnm.set_map(MAP2);
    pump();
    assert_eq!(btnm.get_button_text(0), Some("P"));
    assert_eq!(btnm.get_button_text(1), Some("Q"));
    // Out-of-range index — LVGL returns NULL → None.
    assert_eq!(btnm.get_button_text(99), None);
}

// ── Arc::set_mode / set_bg_start_angle / set_bg_end_angle ────────────────────

#[test]
fn arc_set_mode_variants() {
    use oxivgl::widgets::ArcMode;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_mode(ArcMode::Normal);
    arc.set_mode(ArcMode::Symmetrical);
    arc.set_mode(ArcMode::Reverse);
    pump();
}

#[test]
fn arc_set_bg_start_end_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_start_angle(45);
    arc.set_bg_end_angle(315);
    pump();
}

// ── Dropdown::set_text / set_selected_highlight ───────────────────────────────

#[test]
fn dropdown_set_text_static() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC");
    dd.set_text(c"Menu");
    pump();
}

#[test]
fn dropdown_set_selected_highlight() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.set_selected_highlight(false);
    pump();
    dd.set_selected_highlight(true);
    pump();
}

// ── Spinner ──────────────────────────────────────────────────────────────────

#[test]
fn spinner_create() {
    let screen = fresh_screen();
    let spinner = Spinner::new(&screen).unwrap();
    spinner.size(100, 100).center();
    pump();
}

#[test]
fn spinner_set_anim_params() {
    let screen = fresh_screen();
    let spinner = Spinner::new(&screen).unwrap();
    spinner.set_anim_params(2000, 90);
    pump();
}

// ── Spinbox ──────────────────────────────────────────────────────────────────

#[test]
fn spinbox_create() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.width(100).center();
    pump();
}

#[test]
fn spinbox_range_and_value() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(-100, 100).set_value(42);
    assert_eq!(sb.get_value(), 42);
    // clamped to max
    sb.set_value(200);
    assert_eq!(sb.get_value(), 100);
    pump();
}

#[test]
fn spinbox_increment_decrement() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(0, 100).set_value(50).set_step(10);
    sb.increment();
    assert_eq!(sb.get_value(), 60);
    sb.decrement();
    assert_eq!(sb.get_value(), 50);
    pump();
}

#[test]
fn spinbox_digit_format() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2).set_range(-1000, 25000);
    sb.set_value(1234);
    assert_eq!(sb.get_value(), 1234);
    pump();
}

#[test]
fn spinbox_step_navigation() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_step(1);
    assert_eq!(sb.get_step(), 1);
    sb.step_prev(); // step × 10
    assert_eq!(sb.get_step(), 10);
    sb.step_next(); // step ÷ 10
    assert_eq!(sb.get_step(), 1);
    pump();
}

// ── Arc — uncovered methods ──────────────────────────────────────────────────

#[test]
fn arc_get_value_raw() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 100).set_value_raw(42);
    assert_eq!(arc.get_value_raw(), 42);
    pump();
}

#[test]
fn arc_align_obj_to_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 360).set_value_raw(90);
    let label = Label::new(&screen).unwrap();
    arc.align_obj_to_angle(&label, 10);
    pump();
}

#[test]
fn arc_rotate_obj_to_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 360).set_value_raw(45);
    let label = Label::new(&screen).unwrap();
    arc.rotate_obj_to_angle(&label, 0);
    pump();
}

// ── Bar — uncovered methods ──────────────────────────────────────────────────

#[test]
fn bar_get_value_raw() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100).set_value_raw(75, false);
    assert_eq!(bar.get_value_raw(), 75);
    pump();
}

// ── Spinbox — uncovered methods ──────────────────────────────────────────────

#[test]
fn spinbox_rollover() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(0, 10).set_value(10).set_step(1).set_rollover(true);
    sb.increment();
    assert_eq!(sb.get_value(), 0); // wrapped around
    pump();
}

#[test]
fn spinbox_cursor_pos() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_cursor_pos(2);
    pump();
}

// ── Buttonmatrix (new methods) ──────────────────────────────────────────────

#[test]
fn buttonmatrix_set_button_width() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_width(0, 2);
}

#[test]
fn buttonmatrix_set_and_clear_ctrl() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE);
    assert!(btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
    btnm.clear_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE);
    assert!(!btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
}

#[test]
fn buttonmatrix_ctrl_bitor() {
    let combined = ButtonmatrixCtrl::CHECKABLE | ButtonmatrixCtrl::CHECKED;
    assert_ne!(combined, ButtonmatrixCtrl::NONE);
}

#[test]
fn buttonmatrix_set_button_ctrl_all() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    assert!(btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
}

#[test]
fn buttonmatrix_set_one_checked() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    btnm.set_one_checked(true);
    btnm.set_button_ctrl(0, ButtonmatrixCtrl::CHECKED);
}

// ── Scale::set_rotation ─────────────────────────────────────────────────────

#[test]
fn scale_set_rotation_no_crash() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_rotation(90);
    pump();
}

// ── Keyboard (new methods) ──────────────────────────────────────────────────

#[test]
fn keyboard_set_mode_user1() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::User1);
}

#[test]
fn keyboard_set_map_custom() {
    use oxivgl::btnmatrix_map;
    static MAP: &ButtonmatrixMap = btnmatrix_map!(c"A", c"B", c"C");
    static CTRL: &[ButtonmatrixCtrl] = &[
        ButtonmatrixCtrl::NONE,
        ButtonmatrixCtrl::NONE,
        ButtonmatrixCtrl::NONE,
    ];
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_map(KeyboardMode::User1, MAP, CTRL);
    kb.set_mode(KeyboardMode::User1);
}

// ── Arc getters ──────────────────────────────────────────────────────────────

#[test]
fn arc_get_angle_start() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100).set_value_raw(0); // 0% → indicator at start
    pump();
    // Just verify the call returns without panic and is in 0–360 range.
    let start = arc.get_angle_start();
    assert!(start >= 0.0 && start <= 360.0, "start={start} not in [0,360]");
}

#[test]
fn arc_get_angle_end() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100).set_value_raw(100); // 100% → indicator at end
    pump();
    let end = arc.get_angle_end();
    assert!(end >= 0.0 && end <= 360.0, "end={end} not in [0,360]");
}

#[test]
fn arc_get_bg_angle_start() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_start_angle(30);
    pump();
    let start = arc.get_bg_angle_start();
    assert!((start - 30.0).abs() < 1.0, "expected ~30.0, got {start}");
}

#[test]
fn arc_get_bg_angle_end() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_end_angle(300);
    pump();
    let end = arc.get_bg_angle_end();
    assert!((end - 300.0).abs() < 1.0, "expected ~300.0, got {end}");
}

#[test]
fn arc_get_rotation() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_rotation(90);
    pump();
    assert_eq!(arc.get_rotation(), 90);
}

#[test]
fn arc_get_mode() {
    use oxivgl::widgets::ArcMode;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_mode(ArcMode::Symmetrical);
    pump();
    assert!(matches!(arc.get_mode(), ArcMode::Symmetrical));
}

#[test]
fn arc_get_change_rate() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    // change_rate is a read-only property set by LVGL internally; just verify it returns a value
    let rate = arc.get_change_rate();
    let _ = rate; // no panic
}

// ── Bar getters ──────────────────────────────────────────────────────────────

#[test]
fn bar_get_start_value_raw() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100);
    bar.set_mode(BarMode::Range);
    bar.set_value_raw(80, false);
    bar.set_start_value_raw(20, false);
    assert_eq!(bar.get_start_value_raw(), 20);
}

#[test]
fn bar_get_min_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(10, 90);
    assert_eq!(bar.get_min_value(), 10);
}

#[test]
fn bar_get_max_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(10, 90);
    assert_eq!(bar.get_max_value(), 90);
}

#[test]
fn bar_get_mode() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_mode(BarMode::Symmetrical);
    assert!(matches!(bar.get_mode(), BarMode::Symmetrical));
}

#[test]
fn bar_set_get_orientation() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    assert!(matches!(bar.get_orientation(), BarOrientation::Auto));
    bar.set_orientation(BarOrientation::Vertical);
    assert!(matches!(bar.get_orientation(), BarOrientation::Vertical));
    bar.set_orientation(BarOrientation::Horizontal);
    assert!(matches!(bar.get_orientation(), BarOrientation::Horizontal));
}

// ── Checkbox getter ──────────────────────────────────────────────────────────

#[test]
fn checkbox_get_text() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Accept");
    pump();
    assert_eq!(cb.get_text(), Some("Accept"));
}

// ── Dropdown getters ─────────────────────────────────────────────────────────

#[test]
fn dropdown_get_option_count() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC\nD");
    pump();
    assert_eq!(dd.get_option_count(), 4);
}

#[test]
fn dropdown_get_selected_highlight() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY");
    dd.set_selected_highlight(false);
    pump();
    assert!(!dd.get_selected_highlight());
    dd.set_selected_highlight(true);
    assert!(dd.get_selected_highlight());
}

#[test]
fn dropdown_get_dir() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_dir(DdDir::Top);
    pump();
    let dir = dd.get_dir();
    assert_eq!(dir, DdDir::Top as u32);
}

// ── Scale getters ─────────────────────────────────────────────────────────────

#[test]
fn scale_get_mode() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom);
    pump();
    let mode = scale.get_mode();
    assert_eq!(mode, ScaleMode::HorizontalBottom as u32);
}

#[test]
fn scale_get_total_tick_count() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_total_tick_count(11);
    pump();
    assert_eq!(scale.get_total_tick_count(), 11);
}

#[test]
fn scale_get_rotation() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_rotation(45);
    pump();
    assert_eq!(scale.get_rotation(), 45);
}

#[test]
fn scale_get_label_show() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_label_show(false);
    pump();
    assert!(!scale.get_label_show());
    scale.set_label_show(true);
    assert!(scale.get_label_show());
}

#[test]
fn scale_get_angle_range() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_angle_range(270);
    pump();
    assert_eq!(scale.get_angle_range(), 270);
}

#[test]
fn scale_get_range_min_max_value() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_range(-50, 150);
    pump();
    assert_eq!(scale.get_range_min_value(), -50);
    assert_eq!(scale.get_range_max_value(), 150);
}

// ── Slider getters ───────────────────────────────────────────────────────────

#[test]
fn slider_get_min_max_value() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(-20, 80);
    pump();
    assert_eq!(slider.get_min_value(), -20);
    assert_eq!(slider.get_max_value(), 80);
}

#[test]
fn slider_get_mode() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_mode(SliderMode::Symmetrical);
    pump();
    assert!(matches!(slider.get_mode(), SliderMode::Symmetrical));
}

#[test]
fn slider_set_get_orientation() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    assert!(matches!(slider.get_orientation(), SliderOrientation::Auto));
    slider.set_orientation(SliderOrientation::Vertical);
    assert!(matches!(slider.get_orientation(), SliderOrientation::Vertical));
}

// ── Spinner getters ──────────────────────────────────────────────────────────

#[test]
fn spinner_get_anim_duration() {
    let screen = fresh_screen();
    let sp = Spinner::new(&screen).unwrap();
    sp.set_anim_params(2000, 90);
    pump();
    assert_eq!(sp.get_anim_duration(), 2000);
}

#[test]
fn spinner_get_arc_sweep() {
    let screen = fresh_screen();
    let sp = Spinner::new(&screen).unwrap();
    sp.set_anim_params(1000, 120);
    pump();
    assert_eq!(sp.get_arc_sweep(), 120);
}

// ── Spinbox getters ──────────────────────────────────────────────────────────

#[test]
fn spinbox_get_digit_count() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2);
    pump();
    assert_eq!(sb.get_digit_count(), 5);
}

#[test]
fn spinbox_get_dec_point_pos() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2);
    pump();
    assert_eq!(sb.get_dec_point_pos(), 2);
}

#[test]
fn spinbox_get_min_max_value() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(-100, 500);
    pump();
    assert_eq!(sb.get_min_value(), -100);
    assert_eq!(sb.get_max_value(), 500);
}

// ── Switch getter ────────────────────────────────────────────────────────────

#[test]
fn switch_get_orientation() {
    use oxivgl::widgets::SwitchOrientation;
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();
    sw.set_orientation(SwitchOrientation::Vertical);
    pump();
    assert!(matches!(sw.get_orientation(), SwitchOrientation::Vertical));
}

// ── Textarea getters ─────────────────────────────────────────────────────────

#[test]
fn textarea_get_cursor_click_pos() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_cursor_click_pos wrapper; just verify the getter returns without panic.
    let _click_pos = ta.get_cursor_click_pos();
}

#[test]
fn textarea_get_password_mode() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_password_mode(true);
    pump();
    assert!(ta.get_password_mode());
}

#[test]
fn textarea_get_one_line() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_one_line(true);
    pump();
    assert!(ta.get_one_line());
}

#[test]
fn textarea_get_max_length() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_max_length(64);
    pump();
    assert_eq!(ta.get_max_length(), 64);
}

#[test]
fn textarea_get_text_selection() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_text_selection wrapper; just verify the getter returns without panic.
    let _sel = ta.get_text_selection();
}

#[test]
fn textarea_get_password_show_time() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_password_show_time wrapper; just verify the getter returns without panic.
    let _time = ta.get_password_show_time();
}

// ── Roller getters ───────────────────────────────────────────────────────────

#[test]
fn roller_get_option_count() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC\nD\nE", RollerMode::Normal);
    pump();
    assert_eq!(roller.get_option_count(), 5);
}

#[test]
fn roller_get_selected_str() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("Alpha\nBeta\nGamma", RollerMode::Normal);
    roller.set_selected(1, false);
    pump();
    let mut buf = [0u8; 32];
    let result = roller.get_selected_str(&mut buf);
    assert_eq!(result, Some("Beta"));
}

// ── Keyboard getters ─────────────────────────────────────────────────────────

#[test]
fn keyboard_get_mode() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::Number);
    pump();
    assert!(matches!(kb.get_mode(), KeyboardMode::Number));
}

#[test]
fn keyboard_get_popovers() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    pump();
    // No set_popovers wrapper; just verify the getter returns without panic.
    let _pop = kb.get_popovers();
}

