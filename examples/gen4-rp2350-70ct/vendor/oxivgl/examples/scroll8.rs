#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 8 — Circular Scroll
//!
//! Two scroll containers (horizontal row, vertical column) that loop
//! infinitely: when the scroll reaches either end, the last/first item
//! is moved to the opposite end and the scroll position is adjusted.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ScrollbarMode},
    event::Event,
    layout::FlexFlow,
    style::lv_pct,
    view::{register_event_on, View},
    widgets::{Align, Button, Label, Obj, Part, WidgetError},
};

/// Width/height of each button child. The circular scroll adjustment math
/// assumes all children have this exact dimension — changing child sizes
/// without updating this constant will break the wrap-around positioning.
const ITEM_SIZE: i32 = 80;

#[derive(Default)]
struct Scroll8 {
    cont_row: Option<Obj<'static>>,
    cont_col: Option<Obj<'static>>,
    adjusting_row: bool,
    adjusting_col: bool,
}

impl Scroll8 {
    fn handle_row_scroll(&mut self) {
        if self.adjusting_row {
            return;
        }
        let cont_row = match self.cont_row.as_ref() {
            Some(c) => c,
            None => return,
        };
        self.adjusting_row = true;

        let scroll_x = cont_row.get_scroll_x();
        let child_count = cont_row.get_child_count() as i32;

        if scroll_x <= 0 {
            if let Some(last) = cont_row.get_child(-1) {
                last.move_to_index(0);
            }
            cont_row.scroll_to_x(scroll_x + ITEM_SIZE, false);
        } else {
            let pad_col = cont_row.get_style_pad_column(Part::Main);
            let pad_left = cont_row.get_style_pad_left(Part::Main);
            let pad_right = cont_row.get_style_pad_right(Part::Main);
            let mut content_w = pad_left + pad_right;
            for i in 0..child_count {
                if let Some(c) = cont_row.get_child(i) {
                    content_w += c.get_width();
                    if i < child_count - 1 {
                        content_w += pad_col;
                    }
                }
            }
            let cont_w = cont_row.get_width();
            if scroll_x > content_w - cont_w {
                if let Some(first) = cont_row.get_child(0) {
                    first.move_to_index(child_count - 1);
                }
                cont_row.scroll_to_x(scroll_x - ITEM_SIZE, false);
            }
        }

        self.adjusting_row = false;
    }

    fn handle_col_scroll(&mut self) {
        if self.adjusting_col {
            return;
        }
        let cont_col = match self.cont_col.as_ref() {
            Some(c) => c,
            None => return,
        };
        self.adjusting_col = true;

        let scroll_y = cont_col.get_scroll_y();
        let child_count = cont_col.get_child_count() as i32;

        if scroll_y <= 0 {
            if let Some(last) = cont_col.get_child(-1) {
                last.move_to_index(0);
            }
            cont_col.scroll_to_y(scroll_y + ITEM_SIZE, false);
        } else {
            let pad_row = cont_col.get_style_pad_row(Part::Main);
            let pad_top = cont_col.get_style_pad_top(Part::Main);
            let pad_bottom = cont_col.get_style_pad_bottom(Part::Main);
            let mut content_h = pad_top + pad_bottom;
            for i in 0..child_count {
                if let Some(c) = cont_col.get_child(i) {
                    content_h += c.get_height();
                    if i < child_count - 1 {
                        content_h += pad_row;
                    }
                }
            }
            let cont_h = cont_col.get_height();
            if scroll_y > content_h - cont_h {
                if let Some(first) = cont_col.get_child(0) {
                    first.move_to_index(child_count - 1);
                }
                cont_col.scroll_to_y(scroll_y - ITEM_SIZE, false);
            }
        }

        self.adjusting_col = false;
    }
}

impl View for Scroll8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont_row = Obj::new(container)?;
        cont_row.size(300, 75);
        cont_row.align(Align::TopMid, 0, 5);
        cont_row.set_flex_flow(FlexFlow::Row);
        cont_row.set_scrollbar_mode(ScrollbarMode::Off);

        let cont_col = Obj::new(container)?;
        cont_col.size(200, 150);
        cont_col.align_to(&cont_row, Align::OutBottomMid, 0, 5);
        cont_col.set_flex_flow(FlexFlow::Column);
        cont_col.set_scrollbar_mode(ScrollbarMode::Off);

        for i in 0u32..10 {
            let mut s = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut s, format_args!("Item {}", i + 1));

            let btn_r = Button::new(&cont_row)?;
            btn_r.size(ITEM_SIZE, lv_pct(100));
            let lbl_r = Label::new(&btn_r)?;
            lbl_r.text(&s).center();

            let btn_c = Button::new(&cont_col)?;
            btn_c.size(lv_pct(100), ITEM_SIZE);
            let lbl_c = Label::new(&btn_c)?;
            lbl_c.text(&s).center();
        }

                self.cont_row = Some(cont_row);
        self.cont_col = Some(cont_col);
        self.adjusting_row = false;
        self.adjusting_col = false;
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref cont_row) = self.cont_row {
            register_event_on(self, cont_row.handle());
        }
        if let Some(ref cont_col) = self.cont_col {
            register_event_on(self, cont_col.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::SCROLL {
            return NavAction::None;
        }
        let row_handle = self.cont_row.as_ref().map(|c| c.handle());
        let col_handle = self.cont_col.as_ref().map(|c| c.handle());
        if row_handle == Some(event.current_target_handle()) {
            self.handle_row_scroll();
        } else if col_handle == Some(event.current_target_handle()) {
            self.handle_col_scroll();
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll8::default());
