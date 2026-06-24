#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! List 2 — Reorderable item list with control buttons
//!
//! A left list contains 15 clickable items. Clicking an item selects it
//! (shown via CHECKED state). A right panel has six control buttons:
//! Top, Up, Center, Down, Bottom, and Shuffle. The control buttons move
//! the selected item using `move_to_index` and `scroll_to_view`.
//!
//! Shuffle uses a deterministic counter-based swap to avoid `lv_rand`.
//! Both lists are added to a group; the item list uses gridnav so arrow
//! keys navigate items without individual items entering the group.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ObjState},
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::FlexFlow,
    style::{lv_pct, Style},
    symbols,
    view::{View, register_event_on},
    widgets::{Obj, Align, Button, Label, List, Part, WidgetError},
};
#[derive(Default)]
struct List2 {
    _group: Option<Group>,
    list1: Option<List<'static>>,
    list2: Option<List<'static>>,
    /// Child index in list1 of the currently selected item, or None.
    current_idx: Option<i32>,
    /// Counter used to drive deterministic shuffle.
    shuffle_ctr: u32,
    /// Labels for the left-list items (kept alive; owned by their buttons).
    _item_labels: Option<heapless::Vec<Label<'static>, 15>>,
}

impl View for List2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // ── Left list: 15 plain buttons ───────────────────────────────────
        let list1 = List::new(container)?;
        list1.size(lv_pct(60), lv_pct(100));
        let list1_style = Style::new(|s| {
            s.pad_row(5);
        });
        list1.add_style(&list1_style, Part::Main);
        gridnav_add(&list1, GridnavCtrl::ROLLOVER);

        let mut item_labels: heapless::Vec<Label<'static>, 15> = heapless::Vec::new();
        for i in 0u32..15 {
            let btn = Button::new(&list1)?;
            btn.width(lv_pct(100));
            btn.add_flag(oxivgl::enums::ObjFlag::CHECKABLE);
            btn.bubble_events();
            group_remove_obj(&btn);

            let lbl = Label::new(&btn)?;
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Item {}", i));
            lbl.text(&buf);

            // btn is a child of list1; suppress drop via ManuallyDrop wrapper.
            // We don't need to keep btn alive — LVGL owns it via the list.
            core::mem::forget(btn);
            let _ = item_labels.push(lbl);
        }

        // Select the first item by default.
        if let Some(first) = list1.get_child(0) {
            first.add_state(ObjState::CHECKED);
        }

        // ── Right list: control buttons ───────────────────────────────────
        let list2 = List::new(container)?;
        list2
            .size(lv_pct(40), lv_pct(100))
            .align(Align::TopRight, 0, 0)
            .set_flex_flow(FlexFlow::Column);

        let add_ctrl = |list: &List<'static>, icon, text: &str| -> oxivgl::widgets::Child<Button<'static>> {
            let btn = list.add_button(icon, text);
            btn.bubble_events();
            group_remove_obj(&*btn);
            btn
        };

        let _btn_top    = add_ctrl(&list2, None,                    "Top");
        let _btn_up     = add_ctrl(&list2, Some(&symbols::UP),      "Up");
        let _btn_center = add_ctrl(&list2, Some(&symbols::LEFT),    "Center");
        let _btn_dn     = add_ctrl(&list2, Some(&symbols::DOWN),    "Down");
        let _btn_bot    = add_ctrl(&list2, None,                    "Bottom");
        let _btn_shuf   = add_ctrl(&list2, Some(&symbols::SHUFFLE), "Shuffle");

        // Control buttons are children of list2; drop the Child handles now —
        // their lifetimes are managed by list2.
        drop(_btn_top);
        drop(_btn_up);
        drop(_btn_center);
        drop(_btn_dn);
        drop(_btn_bot);
        drop(_btn_shuf);

        // ── Group ─────────────────────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&list1);
        group.add_obj(&list2);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self.list1 = Some(list1);
        self.list2 = Some(list2);
        self.current_idx = Some(0);
        self.shuffle_ctr = 0;
        self._item_labels = Some(item_labels);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref list1) = self.list1 {
            register_event_on(self, list1.handle());
        }
        if let Some(ref list2) = self.list2 {
            register_event_on(self, list2.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let code = event.code();
        let is_action = code == EventCode::CLICKED || code == EventCode::LONG_PRESSED_REPEAT;
        if !is_action {
            return NavAction::None;
        }

        let target_handle = event.target().handle();

        let list1_handle = match self.list1 {
            Some(ref l) => l.handle(),
            None => return NavAction::None,
        };
        let list2_handle = match self.list2 {
            Some(ref l) => l.handle(),
            None => return NavAction::None,
        };

        if target_handle == list1_handle || target_handle == list2_handle {
            // Event on the container itself — ignore.
            return NavAction::None;
        }

        let list1 = self.list1.as_ref().unwrap();
        let list2 = self.list2.as_ref().unwrap();

        // ── Determine if the click is in list1 (item select) or list2 (control) ──
        // Check whether target is a direct child of list1.
        let cnt1 = list1.get_child_count();
        let mut in_list1 = false;
        for i in 0..cnt1 as i32 {
            if let Some(child) = list1.get_child(i) {
                if child.handle() == target_handle {
                    in_list1 = true;
                    break;
                }
            }
        }

        if in_list1 {
            // Find the clicked item's index.
            let clicked_idx = (0..cnt1 as i32).find(|&i| {
                list1
                    .get_child(i)
                    .map(|c| c.handle() == target_handle)
                    .unwrap_or(false)
            });
            // Toggle selection: clicking the current item deselects it.
            if self.current_idx == clicked_idx {
                self.current_idx = None;
            } else {
                self.current_idx = clicked_idx;
            }
            // Update CHECKED state on all list1 children.
            for i in 0..cnt1 as i32 {
                if let Some(child) = list1.get_child(i) {
                    if self.current_idx == Some(i) {
                        child.add_state(ObjState::CHECKED);
                    } else {
                        child.remove_state(ObjState::CHECKED);
                    }
                }
            }
            return NavAction::None;
        }

        // ── Control button: identify by text ──────────────────────────────
        let cur_idx = match self.current_idx {
            Some(i) => i,
            None => return NavAction::None,
        };
        let cur = match list1.get_child(cur_idx) {
            Some(c) => c,
            None => return NavAction::None,
        };
        let btn_text = list2.get_button_text(&event.target());

        match btn_text {
            Some("Top") => {
                cur.move_to_index(0);
                cur.scroll_to_view(true);
                self.current_idx = Some(cur.get_index());
            }
            Some("Up") => {
                let idx = cur.get_index();
                if idx > 0 {
                    cur.move_to_index(idx - 1);
                    cur.scroll_to_view(true);
                    self.current_idx = Some(cur.get_index());
                }
            }
            Some("Center") => {
                let cnt = list1.get_child_count() as i32;
                cur.move_to_index(cnt / 2);
                cur.scroll_to_view(true);
                self.current_idx = Some(cur.get_index());
            }
            Some("Down") => {
                let idx = cur.get_index();
                cur.move_to_index(idx + 1);
                cur.scroll_to_view(true);
                self.current_idx = Some(cur.get_index());
            }
            Some("Bottom") => {
                cur.move_to_index(-1);
                cur.scroll_to_view(true);
                self.current_idx = Some(cur.get_index());
            }
            Some("Shuffle") => {
                let cnt = list1.get_child_count();
                if cnt > 1 {
                    // Deterministic shuffle: swap pairs using a counter.
                    for _ in 0..20u32 {
                        self.shuffle_ctr = self.shuffle_ctr.wrapping_mul(1664525u32).wrapping_add(1013904223u32);
                        let a = (self.shuffle_ctr >> 16) % cnt;
                        self.shuffle_ctr = self.shuffle_ctr.wrapping_mul(1664525u32).wrapping_add(1013904223u32);
                        let b = (self.shuffle_ctr >> 16) % cnt;
                        if a != b {
                            if let (Some(ca), Some(cb)) = (
                                list1.get_child(a as i32),
                                list1.get_child(b as i32),
                            ) {
                                ca.swap(&*cb);
                            }
                        }
                    }
                    cur.scroll_to_view(true);
                    self.current_idx = Some(cur.get_index());
                }
            }
            _ => {}
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(List2::default());
