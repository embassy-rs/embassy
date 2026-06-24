#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 8 — Circular line chart with gap
//!
//! A line chart in circular update mode with 80 data points. A timer adds
//! new values every 300 ms, overwriting the oldest. Three points ahead of
//! the write cursor are blanked to create a visible gap.

use oxivgl::{
    style::{Selector, Style},
    timer::Timer,
    view::{NavAction, View},
    widgets::{Obj, Chart, ChartAxis, ChartSeries, ChartType, ChartUpdateMode, Part, WidgetError, CHART_POINT_NONE},
};

/// Simple LCG pseudo-random.
fn pseudo_rand(seed: &mut u32, min: i32, max: i32) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let range = (max - min + 1) as u32;
    min + ((*seed >> 16) % range) as i32
}

#[derive(Default)]
struct WidgetChart8 {
    chart: Option<Chart<'static>>,
    ser: Option<ChartSeries>,
    timer: Option<Timer>,
    seed: u32,
}

impl View for WidgetChart8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let chart = Chart::new(container)?;
        chart.size(280, 150);
        chart.center();
        chart.set_type(ChartType::Line);
        chart.set_update_mode(ChartUpdateMode::Circular);
        chart.set_point_count(80);
        let indicator_style = Style::new(|s| {
            s.size(0, 0);
        });
        chart.add_style(&indicator_style, Selector::from(Part::Indicator));

        let color = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color, ChartAxis::PrimaryY);

        let mut seed: u32 = 123;
        for _ in 0..80 {
            chart.set_next_value(&ser, pseudo_rand(&mut seed, 10, 90));
        }

        let timer = Timer::new(300)?;

        self.chart = Some(chart);
        self.ser = Some(ser);
        self.timer = Some(timer);
        self.seed = seed;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let Some(ref timer) = self.timer {
            if timer.triggered() {
                if let (Some(chart), Some(ser)) = (&self.chart, &self.ser) {
                    chart.set_next_value(ser, pseudo_rand(&mut self.seed, 10, 90));

                    // Blank 3 points ahead of write cursor to create gap
                    let p = chart.get_point_count();
                    let s = chart.get_x_start_point(ser);
                    for offset in 1..=3 {
                        let idx = (s + offset) % p;
                        chart.set_series_value_by_id(ser, idx, CHART_POINT_NONE as i32);
                    }
                    chart.refresh();
                }
            }
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetChart8::default());
