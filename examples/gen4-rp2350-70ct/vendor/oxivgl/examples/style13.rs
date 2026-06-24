#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 13 — Parts and states

extern crate alloc;

use oxivgl::{
    style::{palette_lighten, palette_main, GradDir, Palette, Style, StyleBuilder},
    view::{NavAction, View},
    enums::ObjState,
    widgets::{Obj, Part, Slider, WidgetError},
};

#[derive(Default)]
struct Style13 {
    _slider: Option<Slider<'static>>,
    _style_indic: Option<Style>,
    _style_indic_pr: Option<Style>,
}

impl View for Style13 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder_indic = StyleBuilder::new();
        builder_indic
            .bg_color(palette_lighten(Palette::Red, 3))
            .bg_grad_color(palette_main(Palette::Red))
            .bg_grad_dir(GradDir::Hor);
        let style_indic = builder_indic.build();

        let mut builder_indic_pr = StyleBuilder::new();
        builder_indic_pr
            .shadow_color(palette_main(Palette::Red))
            .shadow_width(10)
            .shadow_spread(3);
        let style_indic_pr = builder_indic_pr.build();

        let slider = Slider::new(container)?;
        slider.add_style(&style_indic, Part::Indicator);
        slider.add_style(&style_indic_pr, Part::Indicator | ObjState::PRESSED);
        slider.set_value(70);
        slider.center();

                self._slider = Some(slider);
        self._style_indic = Some(style_indic);
        self._style_indic_pr = Some(style_indic_pr);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style13::default());
