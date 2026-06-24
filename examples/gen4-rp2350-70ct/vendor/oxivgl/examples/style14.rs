#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 14 — Extending the current theme

use oxivgl::{
    style::{palette_darken, palette_main, Palette, StyleBuilder, Theme},
    view::{NavAction, View},
    widgets::{Obj, Align, Button, Label, WidgetError},
};

#[derive(Default)]
struct Style14 {
    _theme: Option<Theme>,
    _label2: Option<Label<'static>>,
    _btn2: Option<Button<'static>>,
    _label1: Option<Label<'static>>,
    _btn1: Option<Button<'static>>,
}

impl View for Style14 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Button created before the theme extension — uses the default theme.
        let btn1 = Button::new(container)?;
        btn1.align(Align::TopMid, 0, 20);
        let label1 = Label::new(&btn1)?;
        label1.text("Original theme").center();

        // Install the theme extension: all buttons created from here on are green.
        let mut style = StyleBuilder::new();
        style
            .bg_color(palette_main(Palette::Green))
            .border_color(palette_darken(Palette::Green, 3))
            .border_width(3);
        let theme = Theme::extend_current(style.build())?;

        // Button created after the theme extension — receives the green style.
        let btn2 = Button::new(container)?;
        btn2.align(Align::BottomMid, 0, -20);
        let label2 = Label::new(&btn2)?;
        label2.text("New theme").center();

                self._theme = Some(theme);
        self._label2 = Some(label2);
        self._btn2 = Some(btn2);
        self._label1 = Some(label1);
        self._btn1 = Some(btn1);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style14::default());
