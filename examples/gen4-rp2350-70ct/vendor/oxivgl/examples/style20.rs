#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 20 — Modal Overlay Dimming
//!
//! A screen with two buttons ("BG Dim" and "OPA Dim"). A full-screen dark
//! overlay is shown on top. For the screenshot the overlay is visible.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Button, Label, Obj, WidgetError},
};

#[derive(Default)]
struct Style20 {
    _btn1: Option<Button<'static>>,
    _btn2: Option<Button<'static>>,
    _lbl1: Option<Label<'static>>,
    _lbl2: Option<Label<'static>>,
    _overlay: Option<Obj<'static>>,
    _overlay_label: Option<Label<'static>>,
}

impl View for Style20 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let container_style = Style::new(|s| {
            s.bg_color_hex(0xffffff).bg_opa(255);
        });
        container.add_style(&container_style, Selector::DEFAULT);

        // Two buttons
        let btn1 = Button::new(container)?;
        btn1.size(120, 50);
        btn1.align(Align::Center, -70, 0);
        let lbl1 = Label::new(&btn1)?;
        lbl1.text("BG Dim");
        lbl1.center();

        let btn2 = Button::new(container)?;
        btn2.size(120, 50);
        btn2.align(Align::Center, 70, 0);
        let lbl2 = Label::new(&btn2)?;
        lbl2.text("OPA Dim");
        lbl2.center();

        // Full-screen overlay (visible for screenshot)
        let overlay = Obj::new(container)?;
        overlay.size(320, 240);
        overlay.align(Align::Center, 0, 0);
        let overlay_style = Style::new(|s| {
            s.bg_color_hex(0x000000).bg_opa(180).border_width(0).radius(0);
        });
        overlay.add_style(&overlay_style, Selector::DEFAULT);

        let overlay_label = Label::new(&overlay)?;
        overlay_label.text("Modal overlay\nclick to dismiss");
        overlay_label.center();
        let overlay_label_style = Style::new(|s| {
            s.text_color_hex(0xffffff);
        });
        overlay_label.add_style(&overlay_label_style, Selector::DEFAULT);

                self._btn1 = Some(btn1);
        self._btn2 = Some(btn2);
        self._lbl1 = Some(lbl1);
        self._lbl2 = Some(lbl2);
        self._overlay = Some(overlay);
        self._overlay_label = Some(overlay_label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style20::default());
