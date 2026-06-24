use crate::common::{fresh_screen, pump};

use oxivgl::enums::{ScrollDir};
use oxivgl::style::{palette_main, Palette, TextDecor};
use oxivgl::widgets::{
    Calendar, CalendarDate, Chart, ChartAxis, ChartType, ChartUpdateMode, Imagebutton, ImagebuttonState, Label, Menu, MenuHeaderMode, Msgbox, Obj, SpanMode, SpanOverflow, Spangroup, Table, TableCellCtrl, Tabview, Tileview, Win,
};

// ── List ──────────────────────────────────────────────────────────────────────

#[test]
fn list_create() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    pump();
    assert!(list.get_width() > 0);
}

#[test]
fn list_add_text() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    list.add_text("Section");
    pump();
    assert!(list.get_child_count() > 0);
}

#[test]
fn list_add_button_and_get_text() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    let btn = list.add_button(Some(&oxivgl::symbols::FILE), "Open");
    pump();
    let text = list.get_button_text(&*btn);
    assert_eq!(text, Some("Open"));
}

#[test]
fn list_add_button_no_icon() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    let btn = list.add_button(None, "NoIcon");
    pump();
    assert_eq!(list.get_button_text(&*btn), Some("NoIcon"));
}

#[test]
fn list_multiple_sections() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    list.add_text("A");
    list.add_button(Some(&oxivgl::symbols::OK), "Item1");
    list.add_text("B");
    list.add_button(None, "Item2");
    pump();
    // 2 text labels + 2 buttons = 4 children
    assert_eq!(list.get_child_count(), 4);
}

// ── Menu ──────────────────────────────────────────────────────────────────────

#[test]
fn menu_create() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    pump();
    drop(menu);
    pump();
}

#[test]
fn menu_page_create_untitled() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    let cont = Menu::cont_create(&page);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("Test");
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_page_create_titled() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(Some("My Page"));
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_set_load_page_event() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let sub = menu.page_create(None);
    let cont_sub = Menu::cont_create(&sub);
    let lbl = Label::new(&cont_sub).unwrap();
    lbl.text("Sub");

    let main = menu.page_create(None);
    let cont = Menu::cont_create(&main);
    let lbl2 = Label::new(&cont).unwrap();
    lbl2.text("Click me");
    menu.set_load_page_event(&cont, &sub);
    menu.set_page(&main);
    pump();
}

#[test]
fn menu_section_and_separator() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    Menu::separator_create(&page);
    let section = Menu::section_create(&page);
    let cont = Menu::cont_create(&section);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("In section");
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_root_back_button() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.set_mode_root_back_button(true);
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    let back_btn = menu.get_main_header_back_button();
    // back_button_is_root should work
    let _is_root = menu.back_button_is_root(&back_btn);
    pump();
}

#[test]
fn menu_header_mode() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.set_mode_header(MenuHeaderMode::BottomFixed);
    pump();
}

#[test]
fn menu_sidebar() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(320, 240);
    let page = menu.page_create(Some("Root"));
    let cont = Menu::cont_create(&page);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("Item");
    menu.set_sidebar_page(&page);
    pump();
    assert!(menu.get_cur_sidebar_page().is_some());
    menu.clear_sidebar();
    pump();
}

#[test]
fn menu_clear_history() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    menu.clear_history();
    pump();
}

#[test]
fn menu_get_cur_main_page() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    assert!(menu.get_cur_main_page().is_some());
}

#[test]
fn menu_get_main_header() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let _header = menu.get_main_header();
    pump();
}

// ── Msgbox ────────────────────────────────────────────────────────────────────

#[test]
fn msgbox_create_modal() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(None::<&Obj<'_>>).unwrap();
    mbox.add_title("Test Title");
    mbox.add_text("Test body");
    mbox.add_close_button();
    pump();
    Msgbox::close(mbox);
    pump();
    let _ = screen; // keep screen alive
}

#[test]
fn msgbox_create_on_parent() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Hello");
    mbox.add_text("Text");
    pump();
}

#[test]
fn msgbox_footer_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(None::<&Obj<'_>>).unwrap();
    mbox.add_title("Confirm");
    mbox.add_text("Are you sure?");
    let _btn = mbox.add_footer_button("OK");
    pump();
    Msgbox::close(mbox);
    pump();
    let _ = screen;
}

// ── Chart ─────────────────────────────────────────────────────────────────────

#[test]
fn chart_create() {
    use oxivgl::widgets::Chart;
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    pump();
    assert!(chart.get_width() > 0);
}

#[test]
fn chart_add_series_and_set_value() {
    use oxivgl::widgets::{Chart, ChartAxis, ChartType, lv_color_t};
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Line);
    chart.set_point_count(5);
    chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
    let color = lv_color_t { blue: 0, green: 0, red: 255 };
    let series = chart.add_series(color, ChartAxis::PrimaryY);
    chart.set_next_value(&series, 42);
    chart.set_next_value(&series, 80);
    chart.refresh();
    pump();
    assert!(chart.get_width() > 0);
}

#[test]
fn chart_scatter_series() {
    use oxivgl::widgets::{Chart, ChartAxis, ChartType, lv_color_t};
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Scatter);
    chart.set_point_count(3);
    let color = lv_color_t { blue: 255, green: 0, red: 0 };
    let series = chart.add_series(color, ChartAxis::PrimaryY);
    chart.set_next_value2(&series, 10, 20);
    chart.set_next_value2(&series, 50, 80);
    chart.refresh();
    pump();
}

// ── Table ─────────────────────────────────────────────────────────────────────

#[test]
fn table_create_and_set_cell() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_cell_value(0, 0, "Hello");
    pump();
    assert_eq!(table.get_cell_value(0, 0).as_deref(), Some("Hello"));
}

#[test]
fn table_row_col_count() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_row_count(4).set_column_count(3);
    pump();
    assert_eq!(table.get_row_count(), 4);
    assert_eq!(table.get_column_count(), 3);
}

#[test]
fn table_column_width() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_column_count(2);
    table.set_column_width(0, 100).set_column_width(1, 80);
    pump();
    assert_eq!(table.get_column_width(0), 100);
    assert_eq!(table.get_column_width(1), 80);
}

#[test]
fn table_cell_ctrl() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_cell_value(0, 0, "Item");
    table.set_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1);
    pump();
    assert!(table.has_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1));
    table.clear_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1);
    assert!(!table.has_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1));
}

#[test]
fn table_selected_cell() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_row_count(3).set_column_count(2);
    // No cell selected initially.
    pump();
    assert_eq!(table.get_selected_cell(), None);
    // Programmatic selection.
    table.set_selected_cell(1, 0);
    pump();
    assert_eq!(table.get_selected_cell(), Some((1, 0)));
}

// ── Tabview ───────────────────────────────────────────────────────────────────

#[test]
fn tabview_create_and_add_tabs() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab1 = tv.add_tab("Alpha");
    let _tab2 = tv.add_tab("Beta");
    pump();
    assert_eq!(tv.get_tab_count(), 2);
    assert_eq!(tv.get_tab_active(), 0);
}

#[test]
fn tabview_set_active() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab1 = tv.add_tab("A");
    let _tab2 = tv.add_tab("B");
    let _tab3 = tv.add_tab("C");
    tv.set_active(2, false);
    pump();
    assert_eq!(tv.get_tab_active(), 2);
}

#[test]
fn tabview_get_content_and_bar() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("Only");
    pump();
    // Just verify the calls don't panic.
    let _content = tv.get_content();
    let _bar = tv.get_tab_bar();
}

#[test]
fn tabview_set_tab_bar_position_and_size() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("A");
    tv.set_tab_bar_position(DdDir::Left);
    tv.set_tab_bar_size(80);
    pump();
}

// ── Calendar ──────────────────────────────────────────────────────────────────

#[test]
fn calendar_create() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 3, 22).set_month_shown(2024, 3);
    pump();
}

#[test]
fn calendar_highlighted_dates() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.set_today_date(2021, 2, 23).set_month_shown(2021, 2);
    cal.set_highlighted_dates(&[
        CalendarDate::new(2021, 2, 6),
        CalendarDate::new(2021, 2, 11),
    ]);
    pump();
    // today and shown dates round-trip correctly
    let today = cal.get_today_date();
    assert_eq!(today.year, 2021);
    assert_eq!(today.month, 2);
    assert_eq!(today.day, 23);
    let shown = cal.get_showed_date();
    assert_eq!(shown.year, 2021);
    assert_eq!(shown.month, 2);
}

#[test]
fn calendar_header_arrow() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    let _hdr = cal.add_header_arrow();
    pump();
}

#[test]
fn calendar_get_pressed_date_none_initially() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.set_today_date(2024, 1, 1).set_month_shown(2024, 1);
    pump();
    // No user click → None
    assert!(cal.get_pressed_date().is_none());
}

// ── Calendar — uncovered methods ─────────────────────────────────────────────

#[test]
fn calendar_header_dropdown() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    let _hdr = cal.add_header_dropdown();
    pump();
}

#[test]
fn calendar_get_btnmatrix() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    let _bm = cal.get_btnmatrix();
    pump();
}

// ── Menu — uncovered methods ─────────────────────────────────────────────────

#[test]
fn menu_clear_page() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(200, 200).center();
    let page = menu.page_create(None);
    let cont = Menu::cont_create(&page);
    let _label = Label::new(&cont).unwrap();
    menu.set_page(&page);
    pump();
    menu.clear_page();
    pump();
}

// ── Spangroup ────────────────────────────────────────────────────────────────

#[test]
fn span_create() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.size(200, 100).center();
    pump();
}

#[test]
fn span_add_and_set_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text(c"Hello spans");
    sg.refresh();
    assert_eq!(sg.get_span_count(), 1);
    pump();
}

#[test]
fn span_add_multiple() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let _s1 = sg.add_span().unwrap();
    let _s2 = sg.add_span().unwrap();
    let _s3 = sg.add_span().unwrap();
    assert_eq!(sg.get_span_count(), 3);
    sg.refresh();
    pump();
}

#[test]
fn span_delete() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    let span = sg.add_span().unwrap();
    span.set_text(c"temp");
    assert_eq!(sg.get_span_count(), 1);
    sg.delete_span(&span);
    assert_eq!(sg.get_span_count(), 0);
    sg.refresh();
    pump();
}

#[test]
fn span_overflow_and_indent() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_overflow(SpanOverflow::Ellipsis);
    sg.set_indent(20);
    assert_eq!(sg.get_indent(), 20);
    let span = sg.add_span().unwrap();
    span.set_text(c"text");
    sg.refresh();
    pump();
}

#[test]
fn span_mode() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_mode(SpanMode::Break);
    let span = sg.add_span().unwrap();
    span.set_text(c"break mode");
    sg.refresh();
    pump();
}

#[test]
fn span_max_lines() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_max_lines(3);
    assert_eq!(sg.get_max_lines(), 3);
    sg.refresh();
    pump();
}

#[test]
fn span_text_color_and_decor() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text(c"styled")
        .set_text_color(palette_main(Palette::Red))
        .set_text_opa(128)
        .set_text_decor(TextDecor::UNDERLINE);
    sg.refresh();
    pump();
}

#[test]
fn span_static_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text_static(c"static text");
    sg.refresh();
    pump();
}

// ── Tileview ──────────────────────────────────────────────────────────────────

#[test]
fn tileview_create_and_add_tiles() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP | ScrollDir::RIGHT);
    let _tile3 = tv.add_tile(1, 1, ScrollDir::LEFT);
    pump();
}

#[test]
fn tileview_set_tile_by_index() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile_by_index(0, 1, false);
    pump();
}

#[test]
fn tileview_set_tile_by_obj() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile(&*tile2, false);
    pump();
    // Switch back
    tv.set_tile(&*tile1, false);
    pump();
}

#[test]
fn tileview_get_active_tile() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile_by_index(0, 1, false);
    pump();
    let active = tv.get_tile_active();
    assert!(active.is_some());
}

// ── Win — window widget ──────────────────────────────────────────────────────

#[test]
fn win_create() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    win.size(300, 200).center();
    pump();
}

#[test]
fn win_add_title() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _title = win.add_title("Hello");
    pump();
}

#[test]
fn win_add_button() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _btn = win.add_button(&oxivgl::symbols::CLOSE, 40);
    pump();
}

#[test]
fn win_get_header() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _hdr = win.get_header();
    pump();
}

#[test]
fn win_get_content() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let content = win.get_content();
    let _lbl = Label::new(&content).unwrap();
    pump();
}

#[test]
fn win_full_example() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _btn1 = win.add_button(&oxivgl::symbols::LEFT, 40);
    let _title = win.add_title("Test Win");
    let _btn2 = win.add_button(&oxivgl::symbols::RIGHT, 40);
    let _btn3 = win.add_button(&oxivgl::symbols::CLOSE, 60);
    let content = win.get_content();
    let lbl = Label::new(&content).unwrap();
    lbl.text("Content text");
    pump();
}

// ── Imagebutton ─────────────────────────────────────────────────────────────

#[test]
fn imagebutton_create() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.size(200, 50).center();
    pump();
}

#[test]
fn imagebutton_set_state() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.set_state(ImagebuttonState::Released);
    btn.set_state(ImagebuttonState::Pressed);
    btn.set_state(ImagebuttonState::Disabled);
    btn.set_state(ImagebuttonState::CheckedReleased);
    btn.set_state(ImagebuttonState::CheckedPressed);
    btn.set_state(ImagebuttonState::CheckedDisabled);
    pump();
}

#[test]
fn imagebutton_set_src_none() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.set_src(ImagebuttonState::Released, None, None, None);
    pump();
}

#[test]
fn imagebutton_with_child_label() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.size(200, 50).center();
    let label = Label::new(&btn).unwrap();
    label.text("Click me");
    label.center();
    pump();
}

// ── Span — uncovered methods ─────────────────────────────────────────────────

#[test]
fn spangroup_letter_and_line_space() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    let span = sg.add_span().unwrap();
    span.set_text(c"Spacing test");
    span.set_text_letter_space(2);
    span.set_text_line_space(4);
    pump();
}

#[test]
fn spangroup_getters() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.set_overflow(SpanOverflow::Ellipsis);
    assert_eq!(sg.get_overflow(), 1); // LV_SPAN_OVERFLOW_ELLIPSIS
    sg.set_mode(SpanMode::Break);
    assert_eq!(sg.get_mode(), 2); // LV_SPAN_MODE_BREAK
    let _h = sg.get_max_line_height();
    sg.size(200, 100);
    let _w = sg.get_expand_width(200);
    let _h2 = sg.get_expand_height(200);
    pump();
}

#[test]
fn spangroup_align_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.set_align_text(1); // LV_TEXT_ALIGN_CENTER
    let span = sg.add_span().unwrap();
    span.set_text(c"Centered");
    pump();
}

// ── Calendar — Chinese mode ─────────────────────────────────────────────────

#[test]
fn calendar_chinese_mode() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(300, 300).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    cal.set_chinese_mode(true, oxivgl::fonts::SOURCE_HAN_SANS_SC_14_CJK);
    cal.font(oxivgl::fonts::SOURCE_HAN_SANS_SC_14_CJK);
    pump();
    // Works on ESP32 hardware.
}

// ── Chart (new methods) ─────────────────────────────────────────────────────

#[test]
fn chart_set_series_value_by_id() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(5);
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    chart.set_series_value_by_id(&ser, 0, 42);
    chart.set_series_value_by_id(&ser, 4, 99);
}

#[test]
fn chart_get_first_point_center_offset() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(10);
    let _ = chart.get_first_point_center_offset();
}

#[test]
fn chart_get_point_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(7);
    assert_eq!(chart.get_point_count(), 7);
}

#[test]
fn chart_set_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
}

// ── Msgbox (new methods) ────────────────────────────────────────────────────

#[test]
fn msgbox_add_header_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Test");
    let _btn = mbox.add_header_button(&oxivgl::symbols::CLOSE);
}

#[test]
fn msgbox_get_content_non_null() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    let content = mbox.get_content();
    // Content should always exist
    drop(content);
}

#[test]
fn msgbox_get_footer_after_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_footer_button("OK");
    assert!(mbox.get_footer().is_some());
}

#[test]
fn msgbox_get_header_after_title() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Test");
    assert!(mbox.get_header().is_some());
}

// ── Chart new APIs ───────────────────────────────────────────────────────────

#[test]
fn chart_set_update_mode() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
}

#[test]
fn chart_add_cursor_and_set_point() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    for i in 0..10 { chart.set_next_value(&ser, i * 10); }
    let color = palette_main(Palette::Blue);
    let cursor = chart.add_cursor(color, 0x01 | 0x08);
    chart.set_cursor_point(&cursor, Some(&ser), 5);
}

#[test]
fn chart_get_pressed_point_none() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    assert!(chart.get_pressed_point().is_none());
}

#[test]
fn chart_get_series_next() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let _ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let first = chart.get_series_next(None);
    assert!(first.is_some());
    let second = chart.get_series_next(first.as_ref());
    assert!(second.is_none());
}

#[test]
fn chart_get_x_start_point() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let _ = chart.get_x_start_point(&ser);
}

// ── Chart getters ────────────────────────────────────────────────────────────

#[test]
fn chart_get_type() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Bar);
    pump();
    assert!(matches!(chart.get_type(), ChartType::Bar));
}

#[test]
fn chart_get_update_mode() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
    pump();
    assert!(matches!(chart.get_update_mode(), ChartUpdateMode::Circular));
}

#[test]
fn chart_get_hor_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
    pump();
    assert_eq!(chart.get_hor_div_line_count(), 3);
}

#[test]
fn chart_get_ver_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
    pump();
    assert_eq!(chart.get_ver_div_line_count(), 5);
}

// ── Tabview getter ───────────────────────────────────────────────────────────

#[test]
fn tabview_get_tab_bar_position() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("A");
    tv.set_tab_bar_position(DdDir::Bottom);
    pump();
    assert!(matches!(tv.get_tab_bar_position(), DdDir::Bottom));
}

// ── Menu getters ─────────────────────────────────────────────────────────────

#[test]
fn menu_get_mode_header_explicit() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(200, 200).center();
    menu.set_mode_header(MenuHeaderMode::BottomFixed);
    pump();
    assert!(matches!(menu.get_mode_header(), MenuHeaderMode::BottomFixed));
}

#[test]
fn menu_get_sidebar_header() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(320, 240);
    // Set a sidebar page so lv_menu_get_sidebar_header() returns non-null.
    let page = menu.page_create(Some("SideRoot"));
    menu.set_sidebar_page(&page);
    pump();
    let _hdr = menu.get_sidebar_header(); // just verify no panic
}

