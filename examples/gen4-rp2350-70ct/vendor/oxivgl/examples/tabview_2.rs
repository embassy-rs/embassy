#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tabview 2 — Left-side tab bar with 4 tabs, custom bar size, and active tab
//! changed programmatically.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, DdDir, Label, Tabview, WidgetError},
};

#[derive(Default)]
struct Tabview2 {
    _tv: Option<Tabview<'static>>,
}

impl View for Tabview2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let tv = Tabview::new(container)?;

        tv.set_tab_bar_position(DdDir::Left).set_tab_bar_size(80);

        let tab1 = tv.add_tab("Info");
        let tab2 = tv.add_tab("Settings");
        let tab3 = tv.add_tab("Stats");
        let tab4 = tv.add_tab("About");

        let label1 = Label::new(&*tab1)?;
        label1.text("Info tab\nSome information here.");

        let label2 = Label::new(&*tab2)?;
        label2.text("Settings tab\nAdjust parameters here.");

        let label3 = Label::new(&*tab3)?;
        label3.text("Stats tab\nView statistics here.");

        let label4 = Label::new(&*tab4)?;
        label4.text("About tab\nVersion 1.0");

        // Start on the second tab.
        tv.set_active(1, false);

                self._tv = Some(tv);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Tabview2::default());
