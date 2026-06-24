#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 1 — Simple progress bar
//!
//! A 200×20 bar centered on screen, set to 70%.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Bar, WidgetError},
};

#[derive(Default)]
struct WidgetBar1 {
    _bar: Option<Bar<'static>>,
}

impl View for WidgetBar1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let bar = Bar::new(container)?;
        bar.size(200, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_value_raw(70, false);

                self._bar = Some(bar);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar1::default());
