#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 3 — Temperature meter
//!
//! Vertical bar with red-to-blue gradient indicator, animated between
//! -20 and 40 (3 s each direction, infinite repeat).

use oxivgl::{
    anim::{anim_set_bar_value, Anim, ANIM_REPEAT_INFINITE},
    style::{palette_main, GradDir, Palette, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Bar, Part, WidgetError},
};

#[derive(Default)]
struct WidgetBar3 {
    _bar: Option<Bar<'static>>,
    _style_indic: Option<Style>,
}

impl View for WidgetBar3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut style_indic = StyleBuilder::new();
        style_indic
            .bg_opa(255)
            .bg_color(palette_main(Palette::Red))
            .bg_grad_color(palette_main(Palette::Blue))
            .bg_grad_dir(GradDir::Ver);
        let style_indic = style_indic.build();

        let bar = Bar::new(container)?;
        bar.add_style(&style_indic, Part::Indicator);
        bar.size(20, 200).center();
        bar.set_range_raw(-20, 40);

        let mut anim = Anim::new();
        anim.set_var(&bar)
            .set_exec_cb(Some(anim_set_bar_value))
            .set_duration(3000)
            .set_reverse_duration(3000)
            .set_values(-20, 40)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .start();

                self._bar = Some(bar);
        self._style_indic = Some(style_indic);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar3::default());
