#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 9 — Horizontal scale with rotated labels
//!
//! Horizontal bottom scale with 45° rotated major tick labels.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Part, Scale, ScaleMode, WidgetError},
};

#[derive(Default)]
struct WidgetScale9 {
    _scale: Option<Scale<'static>>,
}

impl View for WidgetScale9 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(200, 100).center();
        scale
            .set_mode(ScaleMode::HorizontalBottom)
            .set_label_show(true);

        scale.style_transform_rotation(450, Part::Indicator);
        scale.set_tick_length(Part::Indicator, 30);
        scale.style_translate_x(5, Part::Indicator);

        scale
            .set_total_tick_count(31)
            .set_major_tick_every(5)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10)
            .set_range(10, 40);

                self._scale = Some(scale);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale9::default());
