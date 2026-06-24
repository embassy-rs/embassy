#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 5 — Right-to-left scrolling
//!
//! A container with RTL base direction containing a wide label with
//! Persian text. The text scrolls from right to left.

use oxivgl::{
    fonts::DEJAVU_16_PERSIAN_HEBREW,
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{BaseDir, Label, Obj, WidgetError},
};

#[derive(Default)]
struct Scroll5 {
    _cont: Option<Obj<'static>>,
    _label: Option<Label<'static>>,
}

impl View for Scroll5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        let cont_style = Style::new(|s| {
            s.base_dir(BaseDir::Rtl);
        });
        cont.add_style(&cont_style, Selector::DEFAULT);
        cont.size(200, 100);
        cont.center();

        let font_style = Style::new(|s| {
            s.text_font(DEJAVU_16_PERSIAN_HEBREW);
        });

        let label = Label::new(&cont)?;
        label.text("به وسیله یک ماشین نوشته شده است");
        label.width(400);
        label.add_style(&font_style, Selector::DEFAULT);

        self._cont = Some(cont);
        self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll5::default());
