#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Imagebutton 1 — Image button with state switching
//!
//! Creates an image button and a label. Without image assets the button
//! is invisible, but the wrapper API is exercised: state is toggled via
//! `set_state` and a child label is added for visual feedback.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Imagebutton, ImagebuttonState, Label, WidgetError},
};

#[derive(Default)]
struct Imagebutton1 {
    _btn: Option<Imagebutton<'static>>,
    _label: Option<Label<'static>>,
}

impl View for Imagebutton1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btn = Imagebutton::new(container)?;
        btn.size(200, 50).center();
        btn.set_state(ImagebuttonState::Released);

        let label = Label::new(&btn)?;
        label.text("Imagebutton");
        label.center();

                self._btn = Some(btn);
        self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Imagebutton1::default());
