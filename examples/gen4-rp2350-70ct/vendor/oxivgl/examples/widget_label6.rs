#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 6 — Fixed-width glyph override
//!
//! Clones Montserrat 20 and overrides its glyph descriptor callback to force
//! a fixed advance width, producing a monospaced appearance. Two labels show
//! the same text: proportional (top) vs fixed-width (bottom).

use oxivgl::{
    fonts::{FixedWidthFont, MONTSERRAT_20},
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Label, WidgetError},
};

/// Static storage for the cloned fixed-width font. LVGL stores a pointer to
/// the font, so it must live for `'static`.
static MONO_FONT: FixedWidthFont = FixedWidthFont::new();

#[derive(Default)]
struct WidgetLabel6 {
    _label1: Option<Label<'static>>,
    _label2: Option<Label<'static>>,
}

impl View for WidgetLabel6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Label with normal proportional font
        let style_prop = Style::new(|s| {
            s.text_font(MONTSERRAT_20);
        });
        let label1 = Label::new(container)?;
        label1.add_style(&style_prop, Selector::DEFAULT);
        label1.text("0123.Wabc");

        // Label with fixed-width glyph override
        let mono = MONO_FONT.init(MONTSERRAT_20, 20);
        let style_mono = Style::new(|s| {
            s.text_font(mono);
        });
        let label2 = Label::new(container)?;
        label2.y(30);
        label2.add_style(&style_mono, Selector::DEFAULT);
        label2.text("0123.Wabc");

                self._label1 = Some(label1);
        self._label2 = Some(label2);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel6::default());
