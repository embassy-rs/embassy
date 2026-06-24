// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style, theme, palette, gradient, transition, and color-filter integration tests.

use crate::common::{fresh_screen, pump};

use oxivgl::style::{
    color_make, lv_pct, palette_main, props, BorderSide, GradDir, GradDsc, GradExtend, Palette,
    Selector, Style, StyleBuilder, TextDecor, TransitionDsc,
};
use oxivgl::enums::Opa;
use oxivgl::widgets::{
    Arc, Button, Label, Line, Obj, Part,
};
use oxivgl::layout::{FlexAlign, FlexFlow, Layout};
use oxivgl::anim::anim_path_linear;

// ── Palette helpers ──────────────────────────────────────────────────────────

#[test]
fn palette_colors() {
    // These return lv_color_t — just verify they don't crash
    let _ = oxivgl::style::palette_main(Palette::Blue);
    let _ = oxivgl::style::palette_lighten(Palette::Red, 2);
    let _ = oxivgl::style::palette_darken(Palette::Green, 3);
    let _ = oxivgl::style::color_black();
    let _ = oxivgl::style::color_white();
    let _ = oxivgl::style::color_make(0x12, 0x34, 0x56);
}

// ── Style object ─────────────────────────────────────────────────────────────

#[test]
fn style_create_and_apply() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x0000FF)
        .bg_opa(Opa::COVER.0)
        .radius(5)
        .border_width(2)
        .border_color_hex(0xFF0000)
        .pad_all(10);
    let style = sb.build();

    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    assert!(obj.get_width() > 0);
}

// ── Color helpers ────────────────────────────────────────────────────────────

#[test]
fn style_color_brightness_and_darken() {
    use oxivgl::style::{color_brightness, color_darken, color_make};
    let white = color_make(255, 255, 255);
    let brightness = color_brightness(white);
    assert!(brightness > 200, "white should have high brightness, got {brightness}");
    let dark = color_darken(white, 2);
    let dark_brightness = color_brightness(dark);
    assert!(dark_brightness < brightness, "darkened color should be less bright");
}

// ── StyleBuilder::bg_image_src ───────────────────────────────────────────────

oxivgl::image_declare!(img_skew_strip);

#[test]
fn style_bg_image_src_static() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_image_src(img_skew_strip())
        .bg_image_tiled(true)
        .bg_image_opa(128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(200, 50);
    pump();
}

#[test]
fn style_bg_image_recolor() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_image_src(img_skew_strip())
        .bg_image_tiled(true)
        .bg_image_opa(255)
        .bg_image_recolor_hex(0x00A000)
        .bg_image_recolor_opa(180);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(200, 50);
    pump();
}

#[test]
fn obj_bg_image_src_and_recolor_direct() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(200, 50);
    obj.style_bg_image_src(img_skew_strip(), Selector::DEFAULT)
        .style_bg_image_recolor_hex(0x0000FF, Selector::DEFAULT)
        .style_bg_image_recolor_opa(200, Selector::DEFAULT);
    pump();
}

// ── New StyleBuilder methods (shared-style equivalents of inline setters) ─────

#[test]
fn style_builder_new_methods_apply() {
    use oxivgl::widgets::{BaseDir, TextAlign};
    let screen = fresh_screen();
    let style = Style::new(|s| {
        s.pad_hor(4)
            .pad_bottom(2)
            .pad_column(6)
            .pad_row(8)
            .size(40, 20)
            .clip_corner(true)
            .text_align(TextAlign::Center)
            .base_dir(BaseDir::Rtl)
            .radial_offset(3)
            .line_opa(128)
            .arc_rounded(true)
            .blur_radius(2)
            .blur_backdrop(false)
            .radius_circle();
    });
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(80, 40);
    pump();
}

// ── Style::new + build-and-forget ────────────────────────────────────────────

#[test]
fn style_new_closure_constructor() {
    let screen = fresh_screen();
    let style = Style::new(|s| {
        s.bg_color_hex(0x123456).bg_opa(255).radius(6);
    });
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(80, 40);
    pump();
}

#[test]
fn style_retained_by_widget_after_handle_dropped() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(80, 40);
    {
        // Build a shared style, apply it, then drop the handle at end of scope.
        let style = Style::new(|s| {
            s.bg_color_hex(0x00ff00).bg_opa(255);
        });
        obj.add_style(&style, Selector::DEFAULT);
    }
    // The Style handle is gone; the widget's Rc clone keeps it alive.
    pump();
    // Re-render after a clear/reload cycle to exercise the retained style.
    pump();
}

// ── Style transition ─────────────────────────────────────────────────────────

static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_COLOR, props::BG_OPA, props::LAST];

#[test]
fn style_with_transition() {
    let screen = fresh_screen();
    let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

// ── Style drop ordering (spec §4.7, §5.5) ───────────────────────────────────

#[test]
fn style_drop_before_widget() {
    // Style dropped while widget still references it — Rc clone in widget's
    // _styles keeps StyleInner alive until widget drops.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x00FF00).bg_opa(255).radius(5);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    // Drop the user's Style handle; widget's internal clone keeps it alive.
    drop(style);
    pump(); // LVGL renders with style still valid via Rc clone.
    drop(obj); // Widget drop → lv_obj_delete → lv_obj_remove_style_all; then Rc hits 0.
    pump();
}

#[test]
fn add_style_then_drop_widget() {
    // Spec §5.5: "An integration test SHALL exercise the add-style-then-drop-widget path."
    // Widget dropped while styles are still applied. lv_obj_delete internally
    // calls lv_obj_remove_style_all (lv_obj.c:521), clearing LVGL-side refs
    // before Rust drops the _styles Vec.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x0000FF)
        .bg_opa(200)
        .border_width(2)
        .border_color_hex(0xFF0000);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    drop(obj); // Widget drop cleans up LVGL refs, then Rust drops _styles.
    pump();
    // style still valid here (Rc refcount back to 1), no UAF.
    let _clone = style.clone();
}

#[test]
fn style_shared_across_widgets() {
    // Same Style (Rc) applied to multiple widgets. Dropping one widget must
    // not affect the other.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x123456).bg_opa(255);
    let style = sb.build();
    let obj1 = Obj::new(&screen).unwrap();
    let obj2 = Obj::new(&screen).unwrap();
    obj1.add_style(&style, Selector::DEFAULT);
    obj2.add_style(&style, Selector::DEFAULT);
    pump();
    drop(obj1);
    pump(); // obj2 still renders fine with the shared style.
    assert!(obj2.get_width() > 0);
}

#[test]
fn remove_style_then_drop() {
    // Explicitly remove style from widget, then drop both. Tests that
    // remove_style correctly decrements the _styles Vec entry.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xABCDEF).bg_opa(128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    obj.remove_style(Some(&style), Selector::DEFAULT);
    pump();
    drop(obj);
    drop(style);
}

// ── Sub-descriptor double-set (spec §4.4) ────────────────────────────────────

#[test]
fn style_bg_grad_double_set() {
    // Calling bg_grad twice on the same StyleBuilder must not leak the first
    // GradDsc. LVGL overwrites the property map entry in-place (lv_style.c:344-346);
    // the old Box drops after LVGL no longer references it.
    let screen = fresh_screen();
    let mut grad1 = GradDsc::new();
    grad1
        .init_stops(
            &[palette_main(Palette::Blue), palette_main(Palette::Red)],
            &[255, 255],
            &[0, 255],
        )
        .horizontal();
    let mut grad2 = GradDsc::new();
    grad2
        .init_stops(
            &[palette_main(Palette::Green), palette_main(Palette::Yellow)],
            &[255, 255],
            &[0, 255],
        )
        .horizontal();
    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad1).bg_grad(grad2);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
    drop(obj);
    drop(style);
    pump();
}

#[test]
fn style_transition_double_set() {
    // Same principle as bg_grad: calling transition() twice replaces the
    // first TransitionDsc without leaking.
    static P1: [props::lv_style_prop_t; 2] = [props::BG_COLOR, props::LAST];
    static P2: [props::lv_style_prop_t; 2] = [props::BG_OPA, props::LAST];
    let screen = fresh_screen();
    let t1 = TransitionDsc::new(&P1, Some(anim_path_linear), 200, 0);
    let t2 = TransitionDsc::new(&P2, Some(anim_path_linear), 300, 50);
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFF0000).bg_opa(255).transition(t1).transition(t2);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    drop(obj);
    drop(style);
    pump();
}

// ── remove_style(None) — known limitation (spec §5.3) ───────────────────────

#[test]
fn remove_style_none_selector() {
    // remove_style(None, selector) tells LVGL to remove all styles for that
    // selector but does NOT update _styles Vec (spec §5.3 known limitation).
    // Styles remain pinned as Rc clones until widget drops — memory leak, not UAF.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFF0000).bg_opa(255);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    obj.remove_style(None, Selector::DEFAULT);
    pump();
    // Widget and style drop cleanly — no UAF despite stale Vec entries.
    drop(obj);
    drop(style);
    pump();
}

// ── Screen style leak-on-drop (spec §5, F2 fix) ─────────────────────────────

#[test]
fn screen_drop_leaks_styles_safely() {
    // Dropping a Screen value must leak Rc clones (via mem::forget in
    // Screen::Drop) so the LVGL screen object retains valid style pointers.
    // This test verifies no crash when Screen is dropped before child widgets.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x112233).bg_opa(255);
    let style = sb.build();
    let style_clone = style.clone();
    screen.add_style(&style, Selector::DEFAULT);
    pump();
    drop(screen);
    // style_clone keeps StyleInner alive — no UAF on next pump.
    pump();
    // Verify the Rc is still valid (Screen::Drop leaked its clone, plus we
    // hold style_clone).
    drop(style);
    drop(style_clone);
}

// ── GradDsc ──────────────────────────────────────────────────────────────────

#[test]
fn grad_linear_with_stops() {
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Blue), palette_main(Palette::Red)],
        &[255, 255],
        &[0, 255],
    )
    .linear(0, 0, 100, 0, GradExtend::Pad);

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

#[test]
fn grad_radial_and_conical_api() {
    // Radial/conical gradients can hang LVGL's SW renderer on headless SDL,
    // so we test the Rust API construction without rendering.
    let mut radial = GradDsc::new();
    radial
        .init_stops(
            &[palette_main(Palette::Green), palette_main(Palette::Yellow)],
            &[],
            &[],
        )
        .radial(50, 50, 50, 50, GradExtend::Pad)
        .radial_set_focal(25, 25, 10);

    let mut conical = GradDsc::new();
    conical
        .set_stops_count(2)
        .set_stop(0, palette_main(Palette::Red), 255, 0)
        .set_stop(1, palette_main(Palette::Blue), 255, 255)
        .conical(50, 50, 0, 3600, GradExtend::Pad);
}

#[test]
fn grad_horizontal_simple() {
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Cyan), palette_main(Palette::Purple)],
        &[],
        &[],
    )
    .horizontal();

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

#[test]
fn grad_set_dir() {
    use oxivgl::style::GradDir;
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Blue), palette_main(Palette::Red)],
        &[],
        &[],
    )
    .set_dir(GradDir::Hor);

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

// ── Theme ────────────────────────────────────────────────────────────────────

#[test]
fn theme_extend_and_drop() {
    use oxivgl::style::Theme;
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x334455).bg_opa(255);
    let style = sb.build();
    {
        let _theme = Theme::extend_current(style).unwrap();
        // Buttons created now get the theme style.
        let _btn = Button::new(&screen).unwrap();
        pump();
    }
    // Theme dropped — parent theme restored.
    pump();
}

// ── StyleBuilder setters coverage ────────────────────────────────────────────

#[test]
fn style_outline_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.outline_width(3)
        .outline_color(palette_main(Palette::Red))
        .outline_opa(200)
        .outline_pad(2);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

#[test]
fn style_shadow_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.shadow_width(10)
        .shadow_color(palette_main(Palette::Blue))
        .shadow_opa(128)
        .shadow_spread(5)
        .shadow_offset_x(3)
        .shadow_offset_y(3);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(80, 80);
    pump();
}

#[test]
fn style_arc_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.arc_color(palette_main(Palette::Green)).arc_width(8);
    let style = sb.build();
    let arc = Arc::new(&screen).unwrap();
    arc.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_text_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.text_color_hex(0xFF00FF)
        .text_opa(200)
        .text_letter_space(2)
        .text_line_space(4)
        .text_decor(TextDecor::UNDERLINE | TextDecor::STRIKETHROUGH);
    let style = sb.build();
    let label = Label::new(&screen).unwrap();
    label.text("Styled");
    label.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_line_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.line_color(palette_main(Palette::Grey))
        .line_width(4)
        .line_rounded(true);
    let style = sb.build();
    let line = Line::new(&screen).unwrap();
    line.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_dimensions_and_padding() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.width(120)
        .height(80)
        .x(10)
        .y(20)
        .pad_ver(5)
        .pad_left(6)
        .pad_right(7)
        .pad_top(8)
        .pad_all(4)
        .length(50);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_border_side() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.border_width(2)
        .border_color_hex(0xFF0000)
        .border_opa(255)
        .border_side(BorderSide::TOP | BorderSide::BOTTOM);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_layout_and_flex() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.layout(Layout::Flex)
        .flex_flow(FlexFlow::Column)
        .flex_main_place(FlexAlign::Center);
    let style = sb.build();
    let cont = Obj::new(&screen).unwrap();
    cont.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_translate_and_anim() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.translate_y(-10).anim_duration(300);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_transform_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.transform_rotation(450)
        .transform_scale(512)
        .transform_pivot_x(50)
        .transform_pivot_y(50);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

// ── Color filter in style ────────────────────────────────────────────────────

#[test]
fn style_color_filter() {
    use oxivgl::style::{darken_filter_cb, ColorFilter};
    let screen = fresh_screen();
    let filter = ColorFilter::new(darken_filter_cb);
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFFFFFF)
        .bg_opa(255)
        .color_filter(filter, 128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

// ── Part::from_raw + PartialEq ──────────────────────────────────────────────

#[test]
fn part_from_raw() {
    use oxivgl::widgets::Part;
    assert_eq!(Part::from_raw(0x000000), Part::Main);
    assert_eq!(Part::from_raw(0x020000), Part::Indicator);
    assert_eq!(Part::from_raw(0x050000), Part::Items);
    assert_eq!(Part::from_raw(0xFFFFFF), Part::Main); // unknown → Main
}

// ── Part::Cursor ──────────────────────────────────────────────────────────────

#[test]
fn part_cursor_value() {
    assert_eq!(Part::Cursor as u32, 0x060000);
    assert_eq!(Part::from_raw(0x060000), Part::Cursor);
}

// ── Screen::add_style ─────────────────────────────────────────────────────────

#[test]
fn screen_add_style() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x111111).bg_opa(255);
    let style = sb.build();
    screen.add_style(&style, Selector::DEFAULT);
    pump();
}

// ── Style builder — uncovered setters ────────────────────────────────────────

#[test]
fn style_bg_grad_color_hex() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let mut sb = StyleBuilder::new();
    sb.bg_grad_color_hex(0xFF0000).bg_grad_dir(GradDir::Ver);
    let style = sb.build();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_transform_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let mut sb = StyleBuilder::new();
    sb.transform_width(10).transform_height(20);
    let style = sb.build();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_lv_pct() {
    assert!(lv_pct(50) != 50); // lv_pct encodes as a special value
}

// ── Shadow style methods ────────────────────────────────────────────────────

#[test]
fn shadow_style_methods() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 80);
    obj.style_shadow_width(20, Selector::DEFAULT);
    obj.style_shadow_color(color_make(255, 0, 0), Selector::DEFAULT);
    obj.style_shadow_offset_x(5, Selector::DEFAULT);
    obj.style_shadow_offset_y(5, Selector::DEFAULT);
    obj.style_shadow_spread(10, Selector::DEFAULT);
    obj.style_shadow_opa(200, Selector::DEFAULT);
    pump();
}

// ── Blur style ──────────────────────────────────────────────────────────────

#[test]
fn blur_style_methods() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.style_blur_radius(10, Selector::DEFAULT);
    obj.style_blur_backdrop(true, Selector::DEFAULT);
    pump();
}
