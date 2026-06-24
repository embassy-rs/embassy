#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 4 — List with section separators and click logging
//!
//! A single list with 5 sections of 4 "File N" buttons each, separated by
//! text headers. Clicking any item logs its label text. A standalone button
//! sits to the right. Arrow keys navigate the list; Tab moves to the button.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    style::lv_pct,
    symbols,
    view::{View, register_event_on},
    widgets::{Obj, Align, Button, Label, List, WidgetError},
};

#[derive(Default)]
struct Gridnav4 {
    _group: Option<Group>,
    list: Option<List<'static>>,
    _btn: Option<Button<'static>>,
    _btn_label: Option<Label<'static>>,
}

impl View for Gridnav4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // ── List with section separators ──────────────────────────────────
        let list = List::new(container)?;
        list.size(lv_pct(60), lv_pct(90))
            .align(Align::LeftMid, 10, 0);

        gridnav_add(&list, GridnavCtrl::ROLLOVER);

        for i in 0u32..20 {
            if i % 5 == 0 {
                let mut sec = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut sec,
                    format_args!("Section {}", i / 5 + 1),
                );
                list.add_text(&sec);
            }

            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("File {}", i + 1));
            let item = list.add_button(Some(&symbols::FILE), &buf);
            item.bubble_events();
            group_remove_obj(&item);
        }

        // ── Standalone button to the right ────────────────────────────────
        let btn = Button::new(container)?;
        btn.align(Align::RightMid, -10, 0);

        let btn_label = Label::new(&btn)?;
        btn_label.text("Button").center();

        // ── Group: only the list ──────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&list);
        group.add_obj(&btn);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self.list = Some(list);
        self._btn = Some(btn);
        self._btn_label = Some(btn_label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref list) = self.list {
            register_event_on(self, list.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::CLICKED {
            return NavAction::None;
        }
        let target = event.target();
        if let Some(ref list) = self.list {
            if target.handle() == list.handle() {
                return NavAction::None;
            }
            if let Some(text) = list.get_button_text(&target) {
                oxivgl_examples_common::log::info!("Clicked: {}", text);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Gridnav4::default());
