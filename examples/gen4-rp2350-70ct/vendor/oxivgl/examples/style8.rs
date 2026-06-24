#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 8 — Text styles

extern crate alloc;

use oxivgl::{
    style::{palette_lighten, palette_main, Palette, Selector, Style, StyleBuilder, TextDecor},
    view::{NavAction, View},
    widgets::{Obj, Label, WidgetError},
};

#[derive(Default)]
struct Style8 {
    _label: Option<Label<'static>>,
    _style: Option<Style>,
}

impl View for Style8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 2))
            .border_width(2)
            .border_color(palette_main(Palette::Blue))
            .pad_all(10)
            .text_color(palette_main(Palette::Blue))
            .text_letter_space(5)
            .text_line_space(20)
            .text_decor(TextDecor::UNDERLINE);
        let style = builder.build();

        let label = Label::new(container)?;
        label.add_style(&style, Selector::DEFAULT);
        label.text("Text of\na label");
        label.center();

                self._label = Some(label);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style8::default());
