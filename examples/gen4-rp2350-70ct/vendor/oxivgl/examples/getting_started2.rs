#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 2 — Button

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Button, Label, WidgetError},
};

#[derive(Default)]
struct GettingStarted2 {
    _btn: Option<Button<'static>>,
    _label: Option<Label<'static>>,
}

impl View for GettingStarted2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btn = Button::new(container)?;
        btn.pos(10, 10).size(120, 50);

        let label = Label::new(&btn)?;
        label.text("Button").center();

                self._btn = Some(btn);
        self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(GettingStarted2::default());
