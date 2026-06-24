#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 2 — Grid navigation on two side-by-side lists
//!
//! Two scrollable lists: one with 15 "File N" entries (no rollover) and one
//! with 15 "Folder N" entries (rollover). Arrow keys navigate within a list;
//! Tab switches focus between the two list containers.

use oxivgl::{
    enums::ObjState,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    style::{Palette, Style, lv_pct, palette_lighten},
    symbols,
    view::{NavAction, View},
    widgets::{Obj, Align, List, WidgetError},
};

#[derive(Default)]
struct Gridnav2 {
    _group: Option<Group>,
    _list1: Option<List<'static>>,
    _list2: Option<List<'static>>,
    _focus_style: Option<Style>,
    _item_style: Option<Style>,
}

impl View for Gridnav2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Shared styles: list focus background and transparent list-item bg.
        let focus_style = Style::new(|s| {
            s.bg_color(palette_lighten(Palette::Blue, 5));
        });
        let item_style = Style::new(|s| {
            s.bg_opa(0);
        });

        // ── List 1: File items, no rollover ───────────────────────────────
        let list1 = List::new(container)?;
        list1
            .size(lv_pct(45), lv_pct(80))
            .align(Align::LeftMid, 5, 0)
            .add_style(&focus_style, ObjState::FOCUSED);

        gridnav_add(&list1, GridnavCtrl::NONE);

        for i in 1u32..=15 {
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("File {}", i));
            let item = list1.add_button(Some(&symbols::FILE), &buf);
            item.add_style(&item_style, ObjState::DEFAULT);
            group_remove_obj(&item);
        }

        // ── List 2: Folder items, rollover ────────────────────────────────
        let list2 = List::new(container)?;
        list2
            .size(lv_pct(45), lv_pct(80))
            .align(Align::RightMid, -5, 0)
            .add_style(&focus_style, ObjState::FOCUSED);

        gridnav_add(&list2, GridnavCtrl::ROLLOVER);

        for i in 1u32..=15 {
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Folder {}", i));
            let item = list2.add_button(Some(&symbols::DIRECTORY), &buf);
            item.add_style(&item_style, ObjState::DEFAULT);
            group_remove_obj(&item);
        }

        // ── Group: only list containers ───────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&list1);
        group.add_obj(&list2);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self._list1 = Some(list1);
        self._list2 = Some(list2);
        self._focus_style = Some(focus_style);
        self._item_style = Some(item_style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Gridnav2::default());
