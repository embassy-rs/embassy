#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Menu 4 — Dynamic menu with floating add button
//!
//! A menu with one initial item and a floating "+" button. Each click
//! adds a new item with a sub-page and scrolls it into view.

use oxivgl::{
    enums::{EventCode, ObjFlag},
    event::Event,
    style::{Selector, Style},
    symbols,
    view::{NavAction, View},
    widgets::{Obj, Align, Button, Label, Menu, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct WidgetMenu4 {
    menu: Option<Menu<'static>>,
    float_btn: Option<Button<'static>>,
    btn_cnt: u32,
}

impl View for WidgetMenu4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let menu = Menu::new(container)?;
        menu.size(320, 240).center();

        // Initial sub-page
        let sub_page = menu.page_create(None);
        let cont = Menu::cont_create(&sub_page);
        let lbl = Label::new(&cont)?;
        lbl.text("Hello, I am hiding inside the first item");
        core::mem::forget(lbl);

        // Main page
        let main_page = menu.page_create(None);

        let cont1 = Menu::cont_create(&main_page);
        let lbl = Label::new(&cont1)?;
        lbl.text("Item 1");
        menu.set_load_page_event(&cont1, &sub_page);
        core::mem::forget(lbl);

        menu.set_page(&main_page);

        // Floating add button
        let float_btn = Button::new(container)?;
        float_btn.size(50, 50);
        float_btn.add_flag(ObjFlag::FLOATING);
        float_btn.align(Align::BottomRight, -10, -10);
        float_btn.bubble_events();
        let float_btn_style = Style::new(|s| {
            s.radius(RADIUS_MAX as i16);
        });
        float_btn.add_style(&float_btn_style, Selector::DEFAULT);
        float_btn.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);

                self.menu = Some(menu);
        self.float_btn = Some(float_btn);
        self.btn_cnt = 1;
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(float_btn), Some(menu)) = (&self.float_btn, &self.menu) {
            if event.matches(float_btn, EventCode::CLICKED) {
                self.btn_cnt += 1;

                // New sub-page
                let sub_page = menu.page_create(None);
                let cont = Menu::cont_create(&sub_page);
                if let Ok(lbl) = Label::new(&cont) {
                    let mut buf = heapless::String::<64>::new();
                    let _ = core::fmt::Write::write_fmt(
                        &mut buf,
                        format_args!("Hello, I am hiding inside {}", self.btn_cnt),
                    );
                    lbl.text(&buf);
                    core::mem::forget(lbl);
                }

                // New main-page item
                if let Some(main_page) = menu.get_cur_main_page() {
                    let cont = Menu::cont_create(&main_page);
                    if let Ok(lbl) = Label::new(&cont) {
                        let mut buf = heapless::String::<32>::new();
                        let _ = core::fmt::Write::write_fmt(
                            &mut buf,
                            format_args!("Item {}", self.btn_cnt),
                        );
                        lbl.text(&buf);
                        menu.set_load_page_event(&cont, &sub_page);
                        cont.scroll_to_view(true);
                        core::mem::forget(lbl);
                    }
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetMenu4::default());
