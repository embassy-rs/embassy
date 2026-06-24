#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget List 2 — Reorderable list
//!
//! Left panel: 15 items that can be selected (checked state). Right panel:
//! Top/Up/Center/Down/Bottom/Shuffle buttons to reorder the selected item.
//! Shuffle uses a deterministic permutation (no `lv_rand` in safe API).

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    layout::FlexFlow,
    style::{lv_pct, Selector, Style},
    symbols,
    view::{register_event_on, View},
    widgets::{Align, AsLvHandle, Button, Child, List, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetList2 {
    list1: Option<List<'static>>,
    _list2: Option<List<'static>>,
    _item_btns: heapless::Vec<Button<'static>, 15>,
    btn_top: Option<Child<Button<'static>>>,
    btn_up: Option<Child<Button<'static>>>,
    btn_center: Option<Child<Button<'static>>>,
    btn_down: Option<Child<Button<'static>>>,
    btn_bottom: Option<Child<Button<'static>>>,
    btn_shuffle: Option<Child<Button<'static>>>,
    current: Option<*mut oxivgl_sys::lv_obj_t>,
}

impl View for WidgetList2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Left list: items
        let list1 = List::new(container)?;
        list1.size(lv_pct(60), lv_pct(100));
        let list1_style = Style::new(|s| {
            s.pad_row(5);
        });
        list1.add_style(&list1_style, Selector::DEFAULT);

        let mut item_btns = heapless::Vec::<Button<'static>, 15>::new();
        for i in 0..15 {
            let btn = Button::new(&list1)?;
            btn.width(lv_pct(50));
            btn.add_flag(ObjFlag::CHECKABLE);
            btn.bubble_events();
            let lab = oxivgl::widgets::Label::new(&btn)?;
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Item {}", i));
            lab.text(&buf).center();
            let _ = item_btns.push(btn);
        }

        // Select first button
        let first_ptr = item_btns.first().map(|b| b.lv_handle());
        if let Some(btn) = item_btns.first() {
            btn.add_state(ObjState::CHECKED);
        }

        // Right list: control buttons
        let list2 = List::new(container)?;
        list2.size(lv_pct(40), lv_pct(100));
        list2.align(Align::TopRight, 0, 0);
        list2.set_flex_flow(FlexFlow::Column);

        let btn_top = list2.add_button(None, "Top");
        btn_top.bubble_events();
        let btn_up = list2.add_button(Some(&symbols::UP), "Up");
        btn_up.bubble_events();
        let btn_center = list2.add_button(Some(&symbols::LEFT), "Center");
        btn_center.bubble_events();
        let btn_down = list2.add_button(Some(&symbols::DOWN), "Down");
        btn_down.bubble_events();
        let btn_bottom = list2.add_button(None, "Bottom");
        btn_bottom.bubble_events();
        let btn_shuffle = list2.add_button(Some(&symbols::SHUFFLE), "Shuffle");
        btn_shuffle.bubble_events();

                self.list1 = Some(list1);
        self._list2 = Some(list2);
        self._item_btns = item_btns;
        self.btn_top = Some(btn_top);
        self.btn_up = Some(btn_up);
        self.btn_center = Some(btn_center);
        self.btn_down = Some(btn_down);
        self.btn_bottom = Some(btn_bottom);
        self.btn_shuffle = Some(btn_shuffle);
        self.current = first_ptr;
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref list1) = self.list1 { register_event_on(self, list1.lv_handle()); }
        if let Some(ref list2) = self._list2 { register_event_on(self, list2.lv_handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let code = event.code();
        let target = event.target_handle();

        let Some(ref list1) = self.list1 else { return NavAction::None };

        // Item click in list1 — toggle selection
        if code == EventCode::CLICKED && event.current_target_handle() == list1.lv_handle() {
            if self.current == Some(target) {
                self.current = None;
            } else {
                self.current = Some(target);
            }
            // Update checked state on all item children
            let count = list1.get_child_count();
            for i in 0..count as i32 {
                if let Some(child) = list1.get_child(i) {
                    if Some(child.lv_handle()) == self.current {
                        child.add_state(ObjState::CHECKED);
                    } else {
                        child.remove_state(ObjState::CHECKED);
                    }
                }
            }
            return NavAction::None;
        }

        let Some(cur) = self.current else { return NavAction::None };
        let cur_obj = Obj::from_raw_non_owning(cur);

        // Control buttons check code for CLICKED or LONG_PRESSED_REPEAT
        if code != EventCode::CLICKED && code != EventCode::LONG_PRESSED_REPEAT {
            return NavAction::None;
        }

        let btn_top_h = self.btn_top.as_ref().map(|b| b.lv_handle());
        let btn_up_h = self.btn_up.as_ref().map(|b| b.lv_handle());
        let btn_center_h = self.btn_center.as_ref().map(|b| b.lv_handle());
        let btn_down_h = self.btn_down.as_ref().map(|b| b.lv_handle());
        let btn_bottom_h = self.btn_bottom.as_ref().map(|b| b.lv_handle());
        let btn_shuffle_h = self.btn_shuffle.as_ref().map(|b| b.lv_handle());

        if Some(target) == btn_top_h && code == EventCode::CLICKED {
            cur_obj.move_background();
            cur_obj.scroll_to_view(true);
        } else if Some(target) == btn_up_h {
            let idx = cur_obj.get_index();
            if idx > 0 {
                cur_obj.move_to_index(idx - 1);
                cur_obj.scroll_to_view(true);
            }
        } else if Some(target) == btn_center_h {
            let count = list1.get_child_count();
            cur_obj.move_to_index(count as i32 / 2);
            cur_obj.scroll_to_view(true);
        } else if Some(target) == btn_down_h {
            let idx = cur_obj.get_index();
            cur_obj.move_to_index(idx + 1);
            cur_obj.scroll_to_view(true);
        } else if Some(target) == btn_bottom_h && code == EventCode::CLICKED {
            cur_obj.move_foreground();
            cur_obj.scroll_to_view(true);
        } else if Some(target) == btn_shuffle_h {
            // Deterministic shuffle: rotate each child by 7 positions
            let count = list1.get_child_count();
            if count > 1 {
                let idx = cur_obj.get_index();
                let new_idx = (idx + 7) % count as i32;
                cur_obj.move_to_index(new_idx);
                cur_obj.scroll_to_view(true);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetList2::default());
