#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 2 — Bar chart with two series
//!
//! A 200x150 bar chart with 12 monthly data points and two colored series.
//! Simplified from the LVGL C example (scale ticks omitted for clarity).

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Chart, ChartAxis, ChartType, WidgetError},
};

#[derive(Default)]
struct WidgetChart2 {
    _chart: Option<Chart<'static>>,
}

impl View for WidgetChart2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let chart = Chart::new(container)?;
        chart.size(200, 150);
        chart.center();
        chart.set_type(ChartType::Bar);
        chart.set_point_count(12);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        chart.set_div_line_count(3, 0);

        let color1 = oxivgl::style::palette_main(oxivgl::style::Palette::Blue);
        let ser1 = chart.add_series(color1, ChartAxis::PrimaryY);
        for &v in &[35, 28, 42, 15, 50, 38, 22, 45, 30, 55, 18, 40] {
            chart.set_next_value(&ser1, v);
        }

        let color2 = oxivgl::style::palette_main(oxivgl::style::Palette::Orange);
        let ser2 = chart.add_series(color2, ChartAxis::PrimaryY);
        for &v in &[60, 75, 55, 80, 65, 70, 85, 58, 72, 62, 78, 68] {
            chart.set_next_value(&ser2, v);
        }

                self._chart = Some(chart);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetChart2::default());
