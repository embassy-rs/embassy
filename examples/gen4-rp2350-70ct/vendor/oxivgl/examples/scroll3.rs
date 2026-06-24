#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 3 — List with floating add button
//!
//! A list with initial tracks and a floating "+" button in the bottom-right
//! corner. Clicking the button adds a new track and scrolls it into view.

use oxivgl::{
    enums::{EventCode, ObjFlag},
    event::Event,
    style::{Selector, Style},
    symbols,
    view::{NavAction, View},
    widgets::{Obj, Align, Button, List, Part, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct Scroll3 {
    list: Option<List<'static>>,
    float_btn: Option<Button<'static>>,
    btn_cnt: u32,
    _float_btn_style: Option<Style>,
}

impl View for Scroll3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let list = List::new(container)?;
        list.size(280, 220).center();

        // Add initial tracks
        for i in 1..=2u32 {
            let mut buf = heapless::String::<32>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Track {}", i));
            list.add_button(Some(&symbols::AUDIO), &buf);
        }

        // Floating add button
        let float_btn = Button::new(&list)?;
        float_btn.size(50, 50);
        float_btn.add_flag(ObjFlag::FLOATING);
        let pad = list.get_style_pad_right(Part::Main);
        float_btn.align(Align::BottomRight, 0, -pad);
        float_btn.bubble_events();
        let float_btn_style = Style::new(|s| {
            s.radius(RADIUS_MAX as i16);
        });
        float_btn.add_style(&float_btn_style, Selector::DEFAULT);
        float_btn.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);

                self.list = Some(list);
        self.float_btn = Some(float_btn);
        self.btn_cnt = 2;
        self._float_btn_style = Some(float_btn_style);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref float_btn) = self.float_btn {
            if event.matches(float_btn, EventCode::CLICKED) {
                self.btn_cnt += 1;
                let mut buf = heapless::String::<32>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Track {}", self.btn_cnt));
                if let Some(ref list) = self.list {
                    let new_btn = list.add_button(Some(&symbols::AUDIO), &buf);
                    new_btn.scroll_to_view(true);
                }
                if let Some(ref float_btn) = self.float_btn {
                    float_btn.move_foreground();
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll3::default());
