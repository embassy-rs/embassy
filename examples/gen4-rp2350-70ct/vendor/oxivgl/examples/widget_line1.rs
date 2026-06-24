#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Line 1 — Simple line with styled points
//!
//! A line drawn through 5 points with rounded ends and blue color.

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, lv_point_precise_t, Line, WidgetError},
};

static LINE_POINTS: [lv_point_precise_t; 5] = [
    lv_point_precise_t { x: 5.0, y: 5.0 },
    lv_point_precise_t {
        x: 70.0,
        y: 70.0,
    },
    lv_point_precise_t {
        x: 120.0,
        y: 10.0,
    },
    lv_point_precise_t {
        x: 180.0,
        y: 60.0,
    },
    lv_point_precise_t {
        x: 240.0,
        y: 10.0,
    },
];

#[derive(Default)]
struct WidgetLine1 {
    _line: Option<Line<'static>>,
    _style: Option<Style>,
}

impl View for WidgetLine1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut sb = StyleBuilder::new();
        sb.line_width(8)
            .line_color(palette_main(Palette::Blue))
            .line_rounded(true);
        let style = sb.build();

        let line = Line::new(container)?;
        line.set_points(&LINE_POINTS);
        line.add_style(&style, Selector::DEFAULT);
        line.center();

                self._line = Some(line);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLine1::default());
