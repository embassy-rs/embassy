#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 3 — Stacked bar chart
//!
//! A 280x180 chart with three stacked series (red, green, blue), 10 points
//! each. Simplified from the LVGL C example (no tooltip click events).

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Chart, ChartAxis, ChartType, WidgetError},
};

#[derive(Default)]
struct WidgetChart3 {
    _chart: Option<Chart<'static>>,
    _style: Option<Style>,
}

impl View for WidgetChart3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let chart = Chart::new(container)?;
        chart.size(280, 180);
        chart.center();
        chart.set_type(ChartType::Stacked);
        chart.set_point_count(10);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        let style = Style::new(|s| {
            s.pad_column(4);
        });
        chart.add_style(&style, Selector::DEFAULT);

        let color_red = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser1 = chart.add_series(color_red, ChartAxis::PrimaryY);
        for &v in &[10, 15, 12, 20, 8, 18, 14, 22, 11, 16] {
            chart.set_next_value(&ser1, v);
        }

        let color_green = oxivgl::style::palette_main(oxivgl::style::Palette::Green);
        let ser2 = chart.add_series(color_green, ChartAxis::PrimaryY);
        for &v in &[25, 20, 30, 15, 35, 22, 28, 18, 32, 24] {
            chart.set_next_value(&ser2, v);
        }

        let color_blue = oxivgl::style::palette_main(oxivgl::style::Palette::Blue);
        let ser3 = chart.add_series(color_blue, ChartAxis::PrimaryY);
        for &v in &[15, 25, 18, 30, 12, 20, 22, 28, 17, 26] {
            chart.set_next_value(&ser3, v);
        }

        self._chart = Some(chart);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetChart3::default());
