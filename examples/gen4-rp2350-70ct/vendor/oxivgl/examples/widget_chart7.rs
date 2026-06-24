#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 7 — Scatter chart with live data
//!
//! A scatter plot with 50 data points. A timer adds new random points every
//! 100 ms. Simplified from LVGL original (no custom draw coloring).

use oxivgl::{
    style::{Selector, Style},
    timer::Timer,
    view::{NavAction, View},
    widgets::{Obj, Chart, ChartAxis, ChartSeries, ChartType, Part, WidgetError},
};

/// Simple LCG pseudo-random for deterministic scatter data.
fn pseudo_rand(seed: &mut u32, min: i32, max: i32) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let range = (max - min + 1) as u32;
    min + ((*seed >> 16) % range) as i32
}

#[derive(Default)]
struct WidgetChart7 {
    chart: Option<Chart<'static>>,
    ser: Option<ChartSeries>,
    timer: Option<Timer>,
    seed: u32,
    _style_items: Option<Style>,
    _style_indicator: Option<Style>,
}

impl View for WidgetChart7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let chart = Chart::new(container)?;
        chart.size(200, 150);
        chart.center();
        chart.set_type(ChartType::Scatter);
        chart.set_point_count(50);
        chart.set_axis_range(ChartAxis::PrimaryX, 0, 200);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 1000);
        // Hide connecting lines — show points only
        let style_items = Style::new(|s| {
            s.line_width(0);
        });
        chart.add_style(&style_items, Selector::from(Part::Items));
        let style_indicator = Style::new(|s| {
            s.size(4, 4);
        });
        chart.add_style(&style_indicator, Selector::from(Part::Indicator));

        let color = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color, ChartAxis::PrimaryY);

        let mut seed: u32 = 42;
        for _ in 0..50 {
            let x = pseudo_rand(&mut seed, 0, 200);
            let y = pseudo_rand(&mut seed, 0, 1000);
            chart.set_next_value2(&ser, x, y);
        }

        let timer = Timer::new(100)?;

        self.chart = Some(chart);
        self.ser = Some(ser);
        self.timer = Some(timer);
        self.seed = seed;
        self._style_items = Some(style_items);
        self._style_indicator = Some(style_indicator);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let Some(ref timer) = self.timer {
            if timer.triggered() {
                let x = pseudo_rand(&mut self.seed, 0, 200);
                let y = pseudo_rand(&mut self.seed, 0, 1000);
                if let (Some(chart), Some(ser)) = (&self.chart, &self.ser) {
                    chart.set_next_value2(ser, x, y);
                }
            }
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetChart7::default());
