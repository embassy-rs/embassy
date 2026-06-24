#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 8 — Recolored Stripe Pattern
//!
//! Derived from `widget_bar4`: the same tiled stripe background image on the
//! indicator, but tinted green via `bg_image_recolor` + `bg_image_recolor_opa`.
//! The same image could be set directly on the widget with
//! `bar.style_bg_image_src(img_skew_strip(), Part::Indicator)`.

use oxivgl::{
    style::{Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Bar, BarMode, Obj, Part, WidgetError},
};

oxivgl::image_declare!(img_skew_strip);

#[derive(Default)]
struct WidgetBar8 {
    _style: Option<Style>,
    _bar: Option<Bar<'static>>,
}

impl View for WidgetBar8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut style = StyleBuilder::new();
        style
            .bg_image_src(img_skew_strip())
            .bg_image_tiled(true)
            .bg_image_opa(255)
            .bg_image_recolor_hex(0x00a000)
            .bg_image_recolor_opa(180);
        let style = style.build();

        let bar = Bar::new(container)?;
        bar.add_style(&style, Part::Indicator);
        bar.size(260, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_mode(BarMode::Range);
        bar.set_value_raw(90, false);
        bar.set_start_value_raw(20, false);

        self._style = Some(style);
        self._bar = Some(bar);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar8::default());
