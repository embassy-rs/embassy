#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 2 — Text shadow
//!
//! Fake shadow effect: identical text rendered twice, offset by 2 pixels,
//! with reduced opacity for the shadow layer.

use oxivgl::{
    style::{color_black, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Align, Label, WidgetError},
};

#[derive(Default)]
struct WidgetLabel2 {
    _shadow_label: Option<Label<'static>>,
    _main_label: Option<Label<'static>>,
    _style_shadow: Option<Style>,
}

impl View for WidgetLabel2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut style_shadow = StyleBuilder::new();
        style_shadow.text_opa(76); // ~LV_OPA_30
        style_shadow.text_color(color_black());
        let style_shadow = style_shadow.build();

        let shadow_label = Label::new(container)?;
        shadow_label.add_style(&style_shadow, Selector::DEFAULT);

        let main_label = Label::new(container)?;
        main_label.text(
            "A simple method to create\nshadows on a text.\n\
             It even works with\n\nnewlines     and spaces.",
        );

        // Copy shadow text from main label (use same string)
        shadow_label.text(
            "A simple method to create\nshadows on a text.\n\
             It even works with\n\nnewlines     and spaces.",
        );

        main_label.align(Align::Center, 0, 0);
        shadow_label.align_to(&main_label, Align::TopLeft, 2, 2);

                self._shadow_label = Some(shadow_label);
        self._main_label = Some(main_label);
        self._style_shadow = Some(style_shadow);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel2::default());
