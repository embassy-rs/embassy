#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 7 — Dynamic Widget Loading
//!
//! A scrollable column that dynamically loads and unloads items as the user
//! scrolls. Labels on the left show the current range of loaded item numbers.
//! A checkbox toggles scrollbar visibility.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ObjState},
    event::Event,
    layout::FlexFlow,
    style::{lv_pct, Style},
    view::{register_event_on, View},
    widgets::{Align, Checkbox, Label, Obj, Part, WidgetError},
};

#[derive(Default)]
struct Scroll7 {
    cont: Option<Obj<'static>>,
    high_label: Option<Label<'static>>,
    low_label: Option<Label<'static>>,
    checkbox: Option<Checkbox<'static>>,
    top_num: i32,
    bottom_num: i32,
    running: bool,
}

impl Scroll7 {
    fn load_item_back(&self, cont: &Obj<'static>, num: i32) {
        if let Ok(item) = Obj::new(cont) {
            item.size(lv_pct(100), 40);
            if let Ok(lbl) = Label::new(&item) {
                let mut s = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{}", num));
                lbl.text(&s);
            }
        }
    }

    fn load_item_front(&self, cont: &Obj<'static>, num: i32) {
        if let Ok(item) = Obj::new(cont) {
            item.size(lv_pct(100), 40);
            if let Ok(lbl) = Label::new(&item) {
                let mut s = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{}", num));
                lbl.text(&s);
            }
            item.move_to_index(0);
        }
    }

    fn update_scroll(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        let Some(ref cont) = self.cont else { self.running = false; return; };

        // Load items near the bottom
        while self.bottom_num > -30 && cont.get_scroll_bottom() < 200 {
            self.bottom_num -= 1;
            self.load_item_back(cont, self.bottom_num);
            cont.update_layout();
        }

        // Load items near the top, compensating scroll position
        while self.top_num < 30 && cont.get_scroll_top() < 200 {
            self.top_num += 1;
            let bot_before = cont.get_scroll_bottom();
            self.load_item_front(cont, self.top_num);
            cont.update_layout();
            let bot_after = cont.get_scroll_bottom();
            cont.scroll_by(0, bot_before - bot_after, false);
        }

        // Delete far-bottom items
        while cont.get_scroll_bottom() > 600 {
            self.bottom_num += 1;
            cont.delete_child(-1);
            cont.update_layout();
        }

        // Delete far-top items, compensating scroll position
        while cont.get_scroll_top() > 600 {
            self.top_num -= 1;
            let bot_before = cont.get_scroll_bottom();
            cont.delete_child(0);
            cont.update_layout();
            let bot_after = cont.get_scroll_bottom();
            cont.scroll_by(0, bot_before - bot_after, false);
        }

        let mut s = heapless::String::<48>::new();
        let _ = core::fmt::Write::write_fmt(
            &mut s,
            format_args!("current largest\nloaded value:\n{}", self.top_num),
        );
        if let Some(ref high_label) = self.high_label { high_label.text(&s); }

        let mut s = heapless::String::<48>::new();
        let _ = core::fmt::Write::write_fmt(
            &mut s,
            format_args!("current smallest\nloaded value:\n{}", self.bottom_num),
        );
        if let Some(ref low_label) = self.low_label { low_label.text(&s); }

        self.running = false;
    }
}

impl View for Scroll7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let high_label = Label::new(container)?;
        high_label.text("current largest\nloaded value:\n3");
        high_label.align(Align::TopLeft, 10, 10);

        let low_label = Label::new(container)?;
        low_label.text("current smallest\nloaded value:\n3");
        low_label.align(Align::BottomLeft, 10, -10);

        let checkbox = Checkbox::new(container)?;
        checkbox.text("show\nscrollbar");
        checkbox.align(Align::LeftMid, 10, 0);
        checkbox.bubble_events();

        let cont = Obj::new(container)?;
        cont.size(160, 220);
        cont.align(Align::RightMid, -10, 0);
        cont.set_flex_flow(FlexFlow::Column);
        // Hide the scrollbar initially via a shared style; the checkbox handler
        // overrides the opacity at runtime with a local style (last-wins).
        let scrollbar_hidden = Style::new(|s| {
            s.opa(0);
        });
        cont.add_style(&scrollbar_hidden, Part::Scrollbar);

        // Load initial item
        let item = Obj::new(&cont)?;
        item.size(lv_pct(100), 40);
        let lbl = Label::new(&item)?;
        lbl.text("3");

        self.cont = Some(cont);
        self.high_label = Some(high_label);
        self.low_label = Some(low_label);
        self.checkbox = Some(checkbox);
        self.top_num = 3;
        self.bottom_num = 3;
        self.running = false;
        if let Some(ref c) = self.cont { c.update_layout(); }
        self.update_scroll();
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref cont) = self.cont { register_event_on(self, cont.handle()); }
        if let Some(ref cb) = self.checkbox { register_event_on(self, cb.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref cont) = self.cont {
            if event.code() == EventCode::SCROLL
                && event.current_target_handle() == cont.handle()
            {
                self.update_scroll();
                return NavAction::None;
            }
        }
        if let Some(ref checkbox) = self.checkbox {
            if event.matches(checkbox, EventCode::VALUE_CHANGED) {
                let checked = checkbox.has_state(ObjState::CHECKED);
                let opa = if checked { 255u8 } else { 0u8 };
                if let Some(ref cont) = self.cont {
                    #[allow(deprecated)]
                    // runtime-varying style; must stay inline
                    cont.style_opa(opa, Part::Scrollbar);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll7::default());
