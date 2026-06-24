#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Spinner 1 — Centered loading spinner
//!
//! 100×100 spinner with a 10 s animation cycle and 200° arc.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Spinner, WidgetError},
};

#[derive(Default)]
struct Spinner1 {
    _spinner: Option<Spinner<'static>>,
}

impl View for Spinner1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let spinner = Spinner::new(container)?;
        spinner.size(100, 100).center();
        spinner.set_anim_params(10000, 200);

                self._spinner = Some(spinner);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Spinner1::default());
