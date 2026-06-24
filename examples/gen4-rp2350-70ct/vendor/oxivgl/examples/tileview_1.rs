#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tileview 1 — 2×2 tile grid with L-shaped scrolling.
//!
//! Tile (0,0) has a label, tile (0,1) has a button, and tile (1,1) has a
//! scrollable list. Scroll directions form an L shape: down from (0,0),
//! up+right from (0,1), left from (1,1).

use oxivgl::{
    enums::ScrollDir,
    style::lv_pct,
    view::{NavAction, View},
    widgets::{Obj, Button, Label, List, Tileview, WidgetError},
};

#[derive(Default)]
struct Tileview1 {
    _tv: Option<Tileview<'static>>,
}

impl View for Tileview1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let tv = Tileview::new(container)?;

        // Tile1: just a label
        let tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
        let label = Label::new(&*tile1)?;
        label.text("Scroll down");
        label.center();

        // Tile2: a button
        let tile2 = tv.add_tile(0, 1, ScrollDir::TOP | ScrollDir::RIGHT);
        let btn = Button::new(&*tile2)?;
        let btn_label = Label::new(&btn)?;
        btn_label.text("Scroll up or right");
        btn.center();

        // Tile3: a list
        let tile3 = tv.add_tile(1, 1, ScrollDir::LEFT);
        let list = List::new(&*tile3)?;
        list.size(lv_pct(100), lv_pct(100));

        for name in &["One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten"] {
            list.add_button(None, name);
        }

                self._tv = Some(tv);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Tileview1::default());
