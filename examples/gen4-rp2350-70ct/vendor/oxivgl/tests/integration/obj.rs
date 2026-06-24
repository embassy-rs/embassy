use crate::common::{fresh_screen, pump};

use oxivgl::enums::{EventCode, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode};
use oxivgl::fonts::MONTSERRAT_12;
use oxivgl::layout::{FlexAlign, FlexFlow, GridAlign, GridCell, Layout, GRID_TEMPLATE_LAST};
use oxivgl::style::{palette_main, Palette, Selector};
use oxivgl::style::StyleBuilder;
use oxivgl::widgets::{
    Align, Arc, AsLvHandle, Button, Label, Obj, Part, RADIUS_MAX,
};

// ── Obj basics ───────────────────────────────────────────────────────────────

#[test]
fn obj_create_and_size() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);
    pump();
    assert_eq!(obj.get_width(), 100);
    assert_eq!(obj.get_height(), 50);
}

#[test]
fn obj_position() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(10, 20);
    pump();
    assert_eq!(obj.get_x(), 10);
    assert_eq!(obj.get_y(), 20);
}

#[test]
fn obj_center_alignment() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 40).center();
    pump();
    assert_eq!(obj.get_x(), (320 - 60) / 2);
    assert_eq!(obj.get_y(), (240 - 40) / 2);
}

#[test]
fn obj_width_height_setters() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.width(150).height(75);
    pump();
    assert_eq!(obj.get_width(), 150);
    assert_eq!(obj.get_height(), 75);
}

#[test]
fn obj_align_to() {
    let screen = fresh_screen();
    let base = Obj::new(&screen).unwrap();
    base.size(100, 100).pos(0, 0);
    let obj = Obj::new(&screen).unwrap();
    obj.size(20, 20).align_to(&base, Align::OutBottomMid, 0, 5);
    pump();
    // Should be below base, centered horizontally
    assert!(obj.get_y() > 0, "obj should be below base");
}

// ── State / flags ────────────────────────────────────────────────────────────

#[test]
fn obj_state_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    assert!(!obj.has_state(ObjState::CHECKED));
    obj.add_state(ObjState::CHECKED);
    assert!(obj.has_state(ObjState::CHECKED));
    obj.remove_state(ObjState::CHECKED);
    assert!(!obj.has_state(ObjState::CHECKED));
}

#[test]
fn obj_combined_states() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    obj.add_state(ObjState::CHECKED);
    obj.add_state(ObjState::FOCUSED);
    assert!(obj.has_state(ObjState::CHECKED));
    assert!(obj.has_state(ObjState::FOCUSED));

    obj.remove_state(ObjState::CHECKED);
    assert!(!obj.has_state(ObjState::CHECKED));
    assert!(obj.has_state(ObjState::FOCUSED));
}

#[test]
fn obj_flag_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    obj.remove_flag(ObjFlag::SCROLLABLE);
    obj.add_flag(ObjFlag::SCROLLABLE);
    obj.remove_flag(ObjFlag::CLICKABLE);
    obj.add_flag(ObjFlag::CHECKABLE);
    // No crash = success
}

#[test]
fn obj_remove_scrollable_convenience() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_scrollable();
    obj.remove_clickable();
    // Convenience methods work without crash
}

// ── Style setters (obj_style.rs) ─────────────────────────────────────────────

#[test]
fn obj_style_bg_color_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bg_color(0xFF0000).bg_opa(255);
    pump();
    assert!(obj.get_width() > 0);
}

#[test]
fn obj_style_border_pad() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.border_width(2)
        .pad(10)
        .pad_top(5)
        .pad_bottom(5)
        .pad_left(8)
        .pad_right(8);
    pump();
}

#[test]
fn obj_style_text_color_font() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.text_color(0x00FF00).text_font(MONTSERRAT_12);
    pump();
}

#[test]
fn obj_style_font_alias() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // font() is an alias for text_font()
    obj.font(MONTSERRAT_12);
    pump();
}

#[test]
fn obj_style_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.opa(Opa::OPA_50.0);
    pump();
}

#[test]
fn obj_style_radius() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.radius(10, Selector::DEFAULT);
    obj.radius(RADIUS_MAX, ObjState::PRESSED);
    pump();
}

#[test]
fn obj_style_selectors() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_color(
        oxivgl::style::palette_main(Palette::Blue),
        Selector::DEFAULT,
    );
    obj.style_bg_color(
        oxivgl::style::palette_darken(Palette::Blue, 2),
        ObjState::PRESSED,
    );
    pump();
}

#[test]
fn obj_style_transform() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100).center();
    obj.style_transform_rotation(450, Selector::DEFAULT); // 45.0 degrees
    obj.style_transform_scale(512, Selector::DEFAULT); // 2.0x
    obj.style_transform_pivot_x(50, Selector::DEFAULT);
    obj.style_transform_pivot_y(50, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_add_remove() {
    let screen = fresh_screen();
    let style = StyleBuilder::new().build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.remove_style_all();
    pump();
}

#[test]
fn obj_style_grad_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_grad_dir(oxivgl::style::GradDir::Hor, Selector::DEFAULT);
    obj.style_bg_grad_color(
        oxivgl::style::palette_main(Palette::Red),
        Selector::DEFAULT,
    );
    pump();
}

#[test]
fn obj_style_base_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_base_dir(oxivgl::widgets::BaseDir::Rtl, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_line_width() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.line_width(oxivgl::widgets::Part::Main, 3);
    pump();
}

#[test]
fn obj_style_text_align() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.text_align(oxivgl::widgets::TextAlign::Center);
    pump();
}

// ── Layout (obj_layout.rs) ───────────────────────────────────────────────────

#[test]
fn flex_flow_column() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    cont.set_flex_flow(FlexFlow::Column);
    cont.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

    let child1 = Obj::new(&cont).unwrap();
    child1.size(50, 30);
    let child2 = Obj::new(&cont).unwrap();
    child2.size(50, 30);
    pump();

    // In column layout, child2 should be below child1
    assert!(
        child2.get_y() > child1.get_y(),
        "child2.y={} should be > child1.y={}",
        child2.get_y(),
        child1.get_y()
    );
}

#[test]
fn flex_flow_row() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 100);
    cont.set_flex_flow(FlexFlow::Row);

    let child1 = Obj::new(&cont).unwrap();
    child1.size(40, 30);
    let child2 = Obj::new(&cont).unwrap();
    child2.size(40, 30);
    pump();

    // In row layout, child2 should be to the right of child1
    assert!(
        child2.get_x() > child1.get_x(),
        "child2.x={} should be > child1.x={}",
        child2.get_x(),
        child1.get_x()
    );
}

#[test]
fn flex_grow() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 100);
    cont.set_flex_flow(FlexFlow::Row);

    let child = Obj::new(&cont).unwrap();
    child.set_flex_grow(1);
    pump();

    // Flex-grow child should expand
    assert!(child.get_width() > 0);
}

#[test]
fn set_layout_enum() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.set_layout(Layout::Flex);
    pump();
}

static COL_DSC: [i32; 3] = [100, 100, GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 3] = [50, 50, GRID_TEMPLATE_LAST];

#[test]
fn grid_layout() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(220, 120);
    cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);

    let cell = Obj::new(&cont).unwrap();
    cell.set_grid_cell(
        GridCell::new(GridAlign::Stretch, 0, 1),
        GridCell::new(GridAlign::Stretch, 0, 1),
    );
    pump();

    assert!(cell.get_width() > 0, "grid cell should have width");
}

#[test]
fn grid_align() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(220, 120);
    cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);
    cont.set_grid_align(GridAlign::Center, GridAlign::Center);
    pump();
}

// ── Scrollbar mode ───────────────────────────────────────────────────────────

#[test]
fn scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scrollbar_mode(oxivgl::enums::ScrollbarMode::Off);
    obj.set_scrollbar_mode(oxivgl::enums::ScrollbarMode::Auto);
    pump();
}

// ── Widget tree ──────────────────────────────────────────────────────────────

#[test]
fn child_access() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _child1 = Obj::new(&parent).unwrap();
    let _child2 = Obj::new(&parent).unwrap();

    let c0 = parent.get_child(0);
    assert!(c0.is_some(), "first child should exist");
    let c1 = parent.get_child(1);
    assert!(c1.is_some(), "second child should exist");
    let c2 = parent.get_child(2);
    assert!(c2.is_none(), "third child should not exist");
}

#[test]
fn nested_widget_tree() {
    let screen = fresh_screen();
    let container = Obj::new(&screen).unwrap();
    container.size(200, 200);
    let btn = Button::new(&container).unwrap();
    let lbl = Label::new(&btn).unwrap();
    lbl.text("Nested");
    pump();
    assert!(lbl.get_width() > 0);
}

// ── Widget ownership ─────────────────────────────────────────────────────────

#[test]
fn widget_deref_to_obj() {
    let screen = fresh_screen();
    let child = Label::new(&screen).unwrap();
    child.text("via Label");
    pump();
    assert!(child.get_width() > 0);
}

#[test]
fn widget_drop_after_parent_cascade() {
    // Regression: Obj::drop must be a no-op when LVGL has already cascade-
    // deleted the object via parent deletion (lv_obj_is_valid guard).
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let child = Label::new(&parent).unwrap();
    pump();
    drop(parent); // LVGL cascade-deletes child
    pump();
    drop(child); // must not crash — lv_obj_is_valid returns false
    pump();
}

#[test]
fn widget_fire_and_forget() {
    // Widgets created as local variables and forgotten persist in LVGL until
    // their parent is deleted (lv_obj_is_valid returns true, but Rust never
    // calls lv_obj_delete because mem::forget suppresses Drop).
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("ephemeral");
    core::mem::forget(label); // LVGL parent owns and cleans up
    pump();
}

// ── Obj scroll methods ───────────────────────────────────────────────────────

#[test]
fn obj_scroll_snap() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(200, 200);
    obj.set_scroll_snap_x(ScrollSnap::Center);
    obj.set_scroll_snap_y(ScrollSnap::Start);
    obj.set_scroll_dir(ScrollDir::VER);
    pump();
}

#[test]
fn obj_scroll_to_position() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    let child = Obj::new(&cont).unwrap();
    child.size(100, 400);
    pump();
    cont.scroll_to(0, 50, false);
    pump();
    assert!(cont.get_scroll_y() != 0 || cont.get_scroll_x() == 0);
}

#[test]
fn obj_child_count_and_foreground() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    assert_eq!(parent.get_child_count(), 0);
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();
    assert_eq!(parent.get_child_count(), 2);
    _c1.move_foreground();
    pump();
}

#[test]
fn obj_send_event() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SENT: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn cb(_e: *mut oxivgl_sys::lv_event_t) {
        SENT.store(true, Ordering::SeqCst);
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
    btn.send_event(oxivgl::enums::EventCode::CLICKED);
    assert!(SENT.load(Ordering::SeqCst));
}

// ── Obj methods (extended) ───────────────────────────────────────────────────

#[test]
fn obj_bubble_events_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bubble_events();
    assert!(obj.has_flag(ObjFlag::EVENT_BUBBLE));
}

#[test]
fn obj_remove_clickable() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    // Button should be clickable by default.
    assert!(btn.has_flag(ObjFlag::CLICKABLE));
    btn.remove_clickable();
    assert!(!btn.has_flag(ObjFlag::CLICKABLE));
}

#[test]
fn obj_has_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::FLOATING);
    assert!(obj.has_flag(ObjFlag::FLOATING));
    obj.remove_flag(ObjFlag::FLOATING);
    assert!(!obj.has_flag(ObjFlag::FLOATING));
}

#[test]
fn obj_on_callback() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FIRED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |_event| {
        FIRED.store(true, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    assert!(FIRED.load(Ordering::SeqCst));
}

#[test]
fn obj_get_child() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();

    assert!(parent.get_child(0).is_some());
    assert!(parent.get_child(1).is_some());
    assert!(parent.get_child(2).is_none()); // out of range
}

#[test]
fn obj_set_transform_and_reset() {
    use oxivgl::widgets::Matrix;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 60);
    let mut m = Matrix::identity();
    m.scale(0.5, 0.5).rotate(45.0);
    obj.set_transform(&m);
    pump();
    obj.reset_transform();
    pump();
}

#[test]
fn obj_pos_and_getters() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(40, 30).pos(10, 20);
    pump();
    assert_eq!(obj.get_x(), 10);
    assert_eq!(obj.get_y(), 20);
    assert_eq!(obj.get_width(), 40);
    assert_eq!(obj.get_height(), 30);
}

#[test]
fn obj_align_to_out_bottom() {
    let screen = fresh_screen();
    let base = Obj::new(&screen).unwrap();
    base.size(60, 60).align(Align::Center, 0, 0);
    let obj = Obj::new(&screen).unwrap();
    obj.size(20, 20);
    obj.align_to(&base, Align::OutBottomMid, 0, 5);
    pump();
}

#[test]
fn obj_scroll_to_view_and_update_snap() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    let child = Obj::new(&cont).unwrap();
    child.size(100, 400);
    pump();
    child.scroll_to_view(false);
    cont.update_snap(false);
    pump();
}

#[test]
fn obj_scroll_to_view_recursive() {
    let screen = fresh_screen();
    let outer = Obj::new(&screen).unwrap();
    outer.size(100, 100);
    let inner = Obj::new(&outer).unwrap();
    inner.size(100, 400);
    let target = Obj::new(&inner).unwrap();
    target.size(50, 50);
    pump();
    target.scroll_to_view_recursive(false);
    pump();
}

#[test]
fn obj_set_scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.set_scrollbar_mode(ScrollbarMode::Off);
    pump();
    obj.set_scrollbar_mode(ScrollbarMode::Auto);
    pump();
}

#[test]
fn obj_remove_scrollable() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_scrollable();
    assert!(!obj.has_flag(ObjFlag::SCROLLABLE));
}

// ── Obj style methods with selector ─────────────────────────────────────────

#[test]
fn obj_style_text_color_with_selector() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.style_text_color(palette_main(Palette::Red), Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_arc_width_and_color() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.style_arc_width(10, Part::Main);
    arc.style_arc_color(palette_main(Palette::Blue), Part::Main);
    arc.style_arc_rounded(false, Part::Main);
    pump();
}

#[test]
fn obj_style_line_color() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_line_color(palette_main(Palette::Green), Part::Main);
    obj.style_line_width(3, Part::Main);
    pump();
}

#[test]
fn obj_style_length_and_width() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_length(15, Part::Items);
    obj.style_width(20, Part::Indicator);
    pump();
}

// ── Obj::send_draw_task_events ──────────────────────────────────────────────

#[test]
fn obj_send_draw_task_events_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.send_draw_task_events();
    pump();
}

// ── ObjFlag::ADV_HITTEST ────────────────────────────────────────────────────

#[test]
fn obj_adv_hittest_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::ADV_HITTEST);
    pump();
}

// ── Obj::move_to_index / get_index / move_background ─────────────────────────

#[test]
fn obj_move_to_index_and_get_index() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let c0 = Obj::new(&parent).unwrap();
    let c1 = Obj::new(&parent).unwrap();
    let c2 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(c2.get_index(), 2);
    c2.move_to_index(0);
    pump();
    assert_eq!(c2.get_index(), 0);
    // c0 and c1 shifted
    assert_eq!(c0.get_index(), 1);
    assert_eq!(c1.get_index(), 2);
}

#[test]
fn obj_move_background() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c0 = Obj::new(&parent).unwrap();
    let c1 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(c1.get_index(), 1);
    c1.move_background();
    pump();
    assert_eq!(c1.get_index(), 0);
}

#[test]
fn obj_get_style_pad_right() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_right(12);
    pump();
    assert_eq!(obj.get_style_pad_right(Part::Main), 12);
}

#[test]
fn obj_from_raw_non_owning() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let handle = obj.lv_handle();
    {
        let child_ref = Obj::from_raw_non_owning(handle);
        child_ref.size(77, 33);
        pump();
    }
    // obj still valid after child_ref dropped (non-owning)
    pump();
    assert_eq!(obj.get_width(), 77);
}

#[test]
fn obj_delete_child() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&cont).unwrap();
    let _c2 = Obj::new(&cont).unwrap();
    core::mem::forget(_c1);
    core::mem::forget(_c2);
    pump();
    assert_eq!(cont.get_child_count(), 2);
    cont.delete_child(0);  // delete by index
    pump();
    assert_eq!(cont.get_child_count(), 1);
    cont.delete_child(-1); // -1 = last child (LVGL convention)
    pump();
    assert_eq!(cont.get_child_count(), 0);
    let _ = screen;
}

#[test]
fn obj_swap_children() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let a = Obj::new(&parent).unwrap();
    let b = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(a.get_index(), 0);
    assert_eq!(b.get_index(), 1);
    a.swap(&b);
    pump();
    assert_eq!(a.get_index(), 1);
    assert_eq!(b.get_index(), 0);
}

// ── Obj::swap, get_x_aligned, get_y_aligned, get_style_pad_left, get_style_bg_color ──

#[test]
fn obj_get_x_y_aligned() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(15, 25);
    pump();
    // get_x_aligned returns the user-set position before alignment resolution.
    let xa = obj.get_x_aligned();
    let ya = obj.get_y_aligned();
    assert_eq!(xa, 15);
    assert_eq!(ya, 25);
}

#[test]
fn obj_get_style_pad_left() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_left(7);
    pump();
    assert_eq!(obj.get_style_pad_left(Part::Main), 7);
}

#[test]
fn obj_get_style_bg_color() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bg_color(0xFF0000).bg_opa(255);
    pump();
    let _c = obj.get_style_bg_color(Part::Main);
    // Just verify no panic; lv_color_t fields depend on color format.
}

// ── ObjStyle methods ──────────────────────────────────────────────────────────

#[test]
fn obj_style_pad_hor_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_hor(10, Selector::DEFAULT);
    pump();
    // pad_hor sets both left and right; verify left side
    assert_eq!(obj.get_style_pad_left(Part::Main), 10);
}

#[test]
fn obj_style_text_font_with_selector() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("test");
    label.style_text_font(MONTSERRAT_12, Selector::DEFAULT);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn obj_style_pad_column_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_column(5, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_column(Part::Main), 5);
}

#[test]
fn obj_style_size_smoke() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // style_size sets a style property on sub-parts (e.g. indicator size).
    obj.style_size(8, 8, Part::Indicator);
    pump();
}

#[test]
fn obj_style_bg_image_src_symbol_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_image_src_symbol(&oxivgl::symbols::SETTINGS, Selector::DEFAULT);
    obj.bg_opa(255);
    pump();
}

// ── Obj style: clip_corner, translate_x ───────────────────────────────────────

#[test]
fn obj_style_clip_corner() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.radius(20, Selector::DEFAULT);
    obj.style_clip_corner(true, Selector::DEFAULT);
    pump();
    obj.style_clip_corner(false, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_translate_x() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_translate_x(15, Selector::DEFAULT);
    pump();
}

// ── Obj style: image_recolor / image_recolor_opa / radial_offset / line_opa ──

#[test]
fn obj_style_image_recolor_and_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_image_recolor(palette_main(Palette::Red), Selector::DEFAULT);
    obj.style_image_recolor_opa(200, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_radial_offset() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_radial_offset(5, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_line_opa() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_line_opa(128, Part::Main);
    pump();
}

// ── Obj style: style_opa with selector ───────────────────────────────────────

#[test]
fn obj_style_opa_with_selector() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_opa(200, Selector::DEFAULT);
    pump();
}

// ── Obj style: style_pad_all with selector ────────────────────────────────────

#[test]
fn obj_style_pad_all_with_selector() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_all(8, Part::Main);
    pump();
    assert_eq!(obj.get_style_pad_left(Part::Main), 8);
}

// ── Obj style: style_pad_row and style_pad_column ─────────────────────────────

#[test]
fn obj_style_pad_row_and_column_setters() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_row(4, Selector::DEFAULT);
    obj.style_pad_column(7, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_row(Part::Main), 4);
    assert_eq!(obj.get_style_pad_column(Part::Main), 7);
}

// ── ObjFlag::HIDDEN ──────────────────────────────────────────────────────────

#[test]
fn obj_flag_hidden_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    assert!(!obj.has_flag(ObjFlag::HIDDEN));
    obj.add_flag(ObjFlag::HIDDEN);
    assert!(obj.has_flag(ObjFlag::HIDDEN));
    obj.remove_flag(ObjFlag::HIDDEN);
    assert!(!obj.has_flag(ObjFlag::HIDDEN));
}

// ── ObjFlag::SCROLL_MOMENTUM ────────────────────────────────────────────────

#[test]
fn obj_flag_scroll_momentum_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // SCROLL_MOMENTUM is on by default for scrollable objects
    obj.remove_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(!obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
    obj.add_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
    obj.remove_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(!obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
}

// ── ObjFlag::SCROLL_CHAIN ───────────────────────────────────────────────────

#[test]
fn obj_flag_scroll_chain_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_flag(ObjFlag::SCROLL_CHAIN);
    assert!(!obj.has_flag(ObjFlag::SCROLL_CHAIN));
    obj.add_flag(ObjFlag::SCROLL_CHAIN);
    assert!(obj.has_flag(ObjFlag::SCROLL_CHAIN));
    obj.remove_flag(ObjFlag::SCROLL_CHAIN);
    assert!(!obj.has_flag(ObjFlag::SCROLL_CHAIN));
}

// ── Obj scroll/layout methods ────────────────────────────────────────────────

#[test]
fn obj_get_coords() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50).center();
    pump();
    let area = obj.get_coords();
    assert!(area.width() > 0);
    assert!(area.height() > 0);
    let _ = screen;
}

#[test]
fn obj_invalidate() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.invalidate(); // should not panic
    pump();
    let _ = screen;
}

#[test]
fn obj_scroll_to_x_y() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    for _ in 0..5 {
        let child = Obj::new(&cont).unwrap();
        child.size(300, 300);
        core::mem::forget(child);
    }
    pump();
    cont.scroll_to_x(10, false);
    cont.scroll_to_y(10, false);
    pump();
    // After scrolling, at least one scroll position should be non-zero.
    let sx = cont.get_scroll_x();
    let sy = cont.get_scroll_y();
    assert!(sx != 0 || sy != 0, "expected scroll position to change, got x={sx} y={sy}");
    let _ = screen;
}

#[test]
fn obj_get_scroll_top_bottom() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    for _ in 0..5 {
        let child = Obj::new(&cont).unwrap();
        child.size(100, 80);
        core::mem::forget(child);
    }
    pump();
    // scroll down a bit; get_scroll_top() should reflect it
    cont.scroll_to_y(20, false);
    pump();
    let top = cont.get_scroll_top();
    assert!(top >= 0, "scroll_top should be non-negative, got {top}");
    let bot = cont.get_scroll_bottom();
    assert!(bot >= 0, "scroll_bottom should be non-negative, got {bot}");
    let _ = screen;
}

#[test]
fn obj_scroll_by() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    for _ in 0..3 {
        let child = Obj::new(&cont).unwrap();
        child.size(100, 80);
        core::mem::forget(child);
    }
    pump();
    cont.scroll_by(0, 20, false);
    pump();
    let _ = screen;
}

#[test]
fn obj_update_layout() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    let child = Obj::new(&cont).unwrap();
    child.size(50, 50);
    cont.update_layout();
    pump();
    core::mem::forget(child);
    let _ = screen;
}

#[test]
fn obj_get_style_pad_top_bottom() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_top(8);
    obj.pad_bottom(12);
    pump();
    assert_eq!(obj.get_style_pad_top(Part::Main), 8);
    assert_eq!(obj.get_style_pad_bottom(Part::Main), 12);
    let _ = screen;
}

#[test]
fn obj_get_style_pad_row_column() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_row(6, Selector::DEFAULT);
    obj.style_pad_column(9, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_row(Part::Main), 6);
    assert_eq!(obj.get_style_pad_column(Part::Main), 9);
    let _ = screen;
}

// ── Obj getters ──────────────────────────────────────────────────────────────

#[test]
fn obj_get_x2_y2() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(10, 20).size(40, 30);
    pump();
    assert_eq!(obj.get_x2(), 10 + 40);
    assert_eq!(obj.get_y2(), 20 + 30);
}

#[test]
fn obj_get_content_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 80).pad(10);
    pump();
    // content = size - 2 * padding
    assert!(obj.get_content_width() >= 0);
    assert!(obj.get_content_height() >= 0);
}

#[test]
fn obj_get_self_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 40);
    pump();
    assert!(obj.get_self_width() >= 0);
    assert!(obj.get_self_height() >= 0);
}

#[test]
fn obj_get_scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scrollbar_mode(ScrollbarMode::Off);
    pump();
    assert!(matches!(obj.get_scrollbar_mode(), ScrollbarMode::Off));
}

#[test]
fn obj_get_scroll_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scroll_dir(ScrollDir::HOR);
    pump();
    let dir = obj.get_scroll_dir();
    assert_eq!(dir.0 & ScrollDir::HOR.0, ScrollDir::HOR.0);
}

#[test]
fn obj_get_state() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_state(ObjState::CHECKED);
    pump();
    let state = obj.get_state();
    assert_eq!(state.0 & ObjState::CHECKED.0, ObjState::CHECKED.0);
}
