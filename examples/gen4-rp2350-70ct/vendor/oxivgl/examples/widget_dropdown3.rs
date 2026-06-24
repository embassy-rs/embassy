#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 3 — Menu-style dropdown
//!
//! Dropdown with fixed "Menu" button text and no selected-item highlight.

extern crate alloc;

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Dropdown, WidgetError},
};

#[derive(Default)]
struct WidgetDropdown3 {
    _dd: Option<Dropdown<'static>>,
}

impl View for WidgetDropdown3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let dd = Dropdown::new(container)?;
        dd.set_options("New project\nNew file\nSave\nSave as ...\nOpen project\nRecent projects\nPreferences\nExit");
        dd.set_text(c"Menu");
        dd.set_selected_highlight(false);
        dd.align(Align::TopLeft, 10, 10);

                self._dd = Some(dd);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown3::default());
