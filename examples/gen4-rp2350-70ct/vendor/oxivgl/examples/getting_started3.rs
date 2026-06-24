#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 3 — Custom Styles

use oxivgl::{
    style::{
        darken_filter_cb, palette_lighten, palette_main, ColorFilter, GradDir, Palette, Selector,
        Style, StyleBuilder,
    },
    view::{NavAction, View},
    enums::{ObjState, Opa},
    widgets::{Obj, Button, Label, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct GettingStarted3 {
    _lbl2: Option<Label<'static>>,
    _btn2: Option<Button<'static>>,
    _lbl1: Option<Label<'static>>,
    _btn1: Option<Button<'static>>,
    _style_red: Option<Style>,
    _style_pressed: Option<Style>,
    _style_btn: Option<Style>,
    _style_pill: Option<Style>,
}

impl View for GettingStarted3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let color_filter = ColorFilter::new(darken_filter_cb);

        let mut style_btn = StyleBuilder::new();
        style_btn
            .radius(10)
            .bg_opa(Opa::COVER.0)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .bg_grad_color(palette_main(Palette::Grey))
            .bg_grad_dir(GradDir::Ver)
            .border_color_hex(0x000000)
            .border_opa(Opa::OPA_20.0)
            .border_width(2)
            .text_color_hex(0x000000);
        let style_btn = style_btn.build();

        let mut style_pressed = StyleBuilder::new();
        style_pressed.color_filter(color_filter, Opa::OPA_20.0);
        let style_pressed = style_pressed.build();

        let mut style_red = StyleBuilder::new();
        style_red
            .bg_color(palette_main(Palette::Red))
            .bg_grad_color(palette_lighten(Palette::Red, 3));
        let style_red = style_red.build();

        let mut style_pill = StyleBuilder::new();
        style_pill.radius(RADIUS_MAX as i16);
        let style_pill = style_pill.build();

        let btn1 = Button::new(container)?;
        btn1.remove_style_all().pos(10, 10).size(120, 50);
        btn1.add_style(&style_btn, Selector::DEFAULT);
        btn1.add_style(&style_pressed, ObjState::PRESSED);

        let lbl1 = Label::new(&btn1)?;
        lbl1.text("Button").center();

        let btn2 = Button::new(container)?;
        btn2.remove_style_all().pos(10, 80).size(120, 50);
        btn2.add_style(&style_btn, Selector::DEFAULT);
        btn2.add_style(&style_red, Selector::DEFAULT);
        btn2.add_style(&style_pressed, ObjState::PRESSED);
        btn2.add_style(&style_pill, Selector::DEFAULT);

        let lbl2 = Label::new(&btn2)?;
        lbl2.text("Button 2").center();

                self._lbl2 = Some(lbl2);
        self._btn2 = Some(btn2);
        self._lbl1 = Some(lbl1);
        self._btn1 = Some(btn1);
        self._style_red = Some(style_red);
        self._style_pressed = Some(style_pressed);
        self._style_btn = Some(style_btn);
        self._style_pill = Some(style_pill);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(GettingStarted3::default());
