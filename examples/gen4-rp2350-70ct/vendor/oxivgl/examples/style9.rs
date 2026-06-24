#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 9 — Line styles

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, lv_point_precise_t, Line, WidgetError},
};

static POINTS: [lv_point_precise_t; 3] = [
    lv_point_precise_t { x: 10.0, y: 30.0 },
    lv_point_precise_t { x: 30.0, y: 50.0 },
    lv_point_precise_t { x: 100.0, y: 0.0 },
];

#[derive(Default)]
struct Style9 {
    _line: Option<Line<'static>>,
    _style: Option<Style>,
}

impl View for Style9 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .line_color(palette_main(Palette::Grey))
            .line_width(6)
            .line_rounded(true);
        let style = builder.build();

        let line = Line::new(container)?;
        line.add_style(&style, Selector::DEFAULT);
        line.set_points(&POINTS);
        line.center();

                self._line = Some(line);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style9::default());
