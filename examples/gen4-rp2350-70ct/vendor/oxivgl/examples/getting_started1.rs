#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 1 — Hello World

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, Label, WidgetError},
};

#[derive(Default)]
struct GettingStarted1 {
    _label: Option<Label<'static>>,
}

impl View for GettingStarted1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let container_style = Style::new(|s| {
            s.bg_color_hex(0x003a57).bg_opa(255).text_color_hex(0xffffff);
        });
        container.add_style(&container_style, Selector::DEFAULT);

        let label = Label::new(container)?;
        label.text("Hello world").align(Align::Center, 0, 0);

                self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(GettingStarted1::default());
