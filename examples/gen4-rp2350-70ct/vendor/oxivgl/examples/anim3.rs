#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 3 — Cubic bezier with chart
//!
//! Two sliders (P1, P2) adjust bezier control points. A scatter chart shows
//! the curve in real-time. Click the play button to animate a red square along
//! the current bezier curve.

use core::sync::atomic::{AtomicI32, Ordering::Relaxed};

use oxivgl::{
    anim::{Anim, anim_set_translate_x},
    enums::EventCode,
    event::Event,
    layout::{GridAlign, GridCell, GRID_TEMPLATE_LAST, grid_fr},
    math::bezier3,
    style::{Palette, Selector, Style, palette_main},
    view::{NavAction, View},
    widgets::{Button, Chart, ChartAxis, ChartSeries, ChartType, Label, Obj, Part,
        Slider, WidgetError,
    },
};

const CHART_POINTS: u32 = 256;

static COL_DSC: [i32; 4] = [grid_fr(1), 200, grid_fr(1), GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 5] = [30, 10, 10, grid_fr(1), GRID_TEMPLATE_LAST];

// Bezier control points — read by Anim::set_bezier3_path each frame.
static P1: AtomicI32 = AtomicI32::new(0);
static P2: AtomicI32 = AtomicI32::new(0);

#[derive(Default)]
struct Anim3 {
    _cont: Option<Obj<'static>>,
    anim_obj: Option<Obj<'static>>,
    chart: Option<Chart<'static>>,
    series: Option<ChartSeries>,
    p1_slider: Option<Slider<'static>>,
    p1_label: Option<Label<'static>>,
    p2_slider: Option<Slider<'static>>,
    p2_label: Option<Label<'static>>,
    run_btn: Option<Button<'static>>,
    _btn_label: Option<Label<'static>>,
    anim_end: i32,
}

impl Anim3 {
    fn update_chart(&self) {
        if let (Some(chart), Some(series)) = (&self.chart, &self.series) {
            for i in 0..CHART_POINTS {
                let t = (i * (1024 / CHART_POINTS)) as i32;
                let step = bezier3(t, 0, P1.load(Relaxed) as u32, P2.load(Relaxed), 1024);
                chart.set_series_value_by_id2(series, i, t, step);
            }
            chart.refresh();
        }
    }
}

impl View for Anim3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        let cont_style = Style::new(|s| {
            s.pad_all(2).pad_column(10).pad_row(10);
        });
        cont.add_style(&cont_style, Selector::DEFAULT);
        cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);
        cont.size(320, 240).center();
        cont.remove_scrollable();
        cont.bubble_events();

        // Red animated square
        let anim_obj = Obj::new(&cont)?;
        anim_obj.size(30, 30);
        anim_obj.remove_scrollable();
        let anim_obj_style = Style::new(|s| {
            s.bg_color(palette_main(Palette::Red)).bg_opa(255);
        });
        anim_obj.add_style(&anim_obj_style, Selector::DEFAULT);
        anim_obj.set_grid_cell(
            GridCell::new(GridAlign::Start, 0, 1),
            GridCell::new(GridAlign::Start, 0, 1),
        );

        // Shared knob padding style for both sliders.
        let knob_pad_style = Style::new(|s| {
            s.pad_all(2);
        });

        // P1 label + slider
        let p1_label = Label::new(&cont)?;
        p1_label.text("p1:0");
        p1_label.set_grid_cell(GridCell::at(0), GridCell::at(1));

        let p1_slider = Slider::new(&cont)?;
        p1_slider.set_range(0, 1024);
        p1_slider.add_style(&knob_pad_style, Part::Knob);
        p1_slider.bubble_events();
        p1_slider.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 1, 1),
            GridCell::new(GridAlign::Start, 1, 1),
        );

        // P2 label + slider
        let p2_label = Label::new(&cont)?;
        p2_label.text("p2:0");
        p2_label.set_grid_cell(GridCell::at(0), GridCell::new(GridAlign::Start, 2, 1));

        let p2_slider = Slider::new(&cont)?;
        p2_slider.set_range(0, 1024);
        p2_slider.add_style(&knob_pad_style, Part::Knob);
        p2_slider.bubble_events();
        p2_slider.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 1, 1),
            GridCell::new(GridAlign::Start, 2, 1),
        );

        // Play button
        let run_btn = Button::new(&cont)?;
        run_btn.bubble_events();
        run_btn.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 2, 1),
            GridCell::new(GridAlign::Stretch, 1, 2),
        );
        let btn_label = Label::new(&run_btn)?;
        btn_label.text(">").center();

        // Chart
        let chart = Chart::new(&cont)?;
        let chart_style = Style::new(|s| {
            s.pad_all(0);
        });
        chart.add_style(&chart_style, Selector::DEFAULT);
        let chart_indicator_style = Style::new(|s| {
            s.size(2, 2);
        });
        chart.add_style(&chart_indicator_style, Part::Indicator);
        chart.set_type(ChartType::Scatter);
        let series = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 1024);
        chart.set_axis_range(ChartAxis::PrimaryX, 0, 1024);
        chart.set_point_count(CHART_POINTS);
        chart.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 0, 3),
            GridCell::new(GridAlign::Stretch, 3, 1),
        );

        // Compute animation end value (container width - obj width - margin).
        let anim_end = 320 - 30 - 10;

        self._cont = Some(cont);
        self.anim_obj = Some(anim_obj);
        self.chart = Some(chart);
        self.series = Some(series);
        self.p1_slider = Some(p1_slider);
        self.p1_label = Some(p1_label);
        self.p2_slider = Some(p2_slider);
        self.p2_label = Some(p2_label);
        self.run_btn = Some(run_btn);
        self._btn_label = Some(btn_label);
        self.anim_end = anim_end;
        self.update_chart();
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref p1_slider) = self.p1_slider {
            if event.matches(p1_slider, EventCode::VALUE_CHANGED) {
                let val = p1_slider.get_value();
                P1.store(val, Relaxed);
                let mut buf = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("p1:{}", val));
                if let Some(ref p1_label) = self.p1_label {
                    p1_label.text(&buf);
                }
                self.update_chart();
                return NavAction::None;
            }
        }
        if let Some(ref p2_slider) = self.p2_slider {
            if event.matches(p2_slider, EventCode::VALUE_CHANGED) {
                let val = p2_slider.get_value();
                P2.store(val, Relaxed);
                let mut buf = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("p2:{}", val));
                if let Some(ref p2_label) = self.p2_label {
                    p2_label.text(&buf);
                }
                self.update_chart();
                return NavAction::None;
            }
        }
        if let Some(ref run_btn) = self.run_btn {
            if event.matches(run_btn, EventCode::CLICKED) {
                if let Some(ref anim_obj) = self.anim_obj {
                    let mut a = Anim::new();
                    a.set_var(anim_obj)
                        .set_values(5, self.anim_end)
                        .set_duration(2000)
                        .set_exec_cb(Some(anim_set_translate_x))
                        .set_bezier3_path(&P1, &P2);
                    a.start();
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Anim3::default());
