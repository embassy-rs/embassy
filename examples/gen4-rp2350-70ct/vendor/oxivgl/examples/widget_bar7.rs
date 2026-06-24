#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 7 — Reversed vertical bar
//!
//! Vertical bar filling top-to-bottom via reversed range (100→0), at 70%.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Bar, Label, WidgetError},
};

#[derive(Default)]
struct WidgetBar7 {
    _bar: Option<Bar<'static>>,
    _label: Option<Label<'static>>,
}

impl View for WidgetBar7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let bar = Bar::new(container)?;
        bar.size(20, 200);
        bar.set_range_raw(100, 0);
        bar.set_value_raw(70, false);
        bar.align(Align::Center, 0, -30);

        let label = Label::new(container)?;
        label.text("Top to bottom");
        label.align_to(&bar, Align::OutTopMid, 0, -5);

                self._bar = Some(bar);
        self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar7::default());
