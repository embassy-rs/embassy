#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 3 — Interactive pie chart
//!
//! Five colored arc segments forming a pie chart. Clicking a slice animates
//! it outward; clicking again or clicking another slice animates it back.

extern crate alloc;
use alloc::vec::Vec;

use oxivgl::view::NavAction;
use oxivgl::{
    anim::{anim_set_x, anim_set_y, Anim},
    enums::{EventCode, ObjFlag},
    event::Event,
    math::{trigo_cos, trigo_sin, TRIGO_SHIFT},
    style::{palette_main, Palette, Selector, Style},
    view::{register_event_on, View},
    widgets::{Align, Arc, ArcMode, Label, Obj, Part, WidgetError},
};

const CHART_SIZE: i32 = 150;
const SLICE_OFFSET: i32 = 20;
const NUM_SLICES: usize = 5;

#[derive(Default)]
struct SliceInfo {
    mid_angle: i32,
    home_x: i32,
    home_y: i32,
    out: bool,
}

#[derive(Default)]
struct WidgetArc3 {
    _container: Option<Obj<'static>>,
    arcs: Option<[Arc<'static>; NUM_SLICES]>,
    _labels: Option<[Label<'static>; NUM_SLICES]>,
    slices: Option<[SliceInfo; NUM_SLICES]>,
    active: Option<usize>,
}

const PERCENTAGES: [i32; NUM_SLICES] = [12, 18, 26, 24, 20];
const COLORS: [Palette; NUM_SLICES] = [
    Palette::Red,
    Palette::Blue,
    Palette::Green,
    Palette::Orange,
    Palette::BlueGrey,
];

impl WidgetArc3 {
    fn animate_slice(&self, idx: usize, to_x: i32, to_y: i32) {
        let Some(ref arcs) = self.arcs else { return };
        let arc = &arcs[idx];
        let cur_x = arc.get_x();
        let cur_y = arc.get_y();

        let mut ax = Anim::new();
        ax.set_var(arc)
            .set_exec_cb(Some(anim_set_x))
            .set_values(cur_x, to_x)
            .set_duration(200);
        ax.start();

        let mut ay = Anim::new();
        ay.set_var(arc)
            .set_exec_cb(Some(anim_set_y))
            .set_values(cur_y, to_y)
            .set_duration(200);
        ay.start();
    }
}

impl View for WidgetArc3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Slices container — transparent, non-scrollable
        let cont = Obj::new(container)?;
        let cont_size = CHART_SIZE + 2 * SLICE_OFFSET;
        cont.size(cont_size, cont_size).center();
        let cont_style = Style::new(|s| {
            s.pad_all(0).border_width(0).bg_opa(0);
        });
        cont.add_style(&cont_style, Selector::DEFAULT);
        cont.remove_flag(ObjFlag::SCROLLABLE);

        // Shared arc-width styles, applied to every slice (identical values).
        let arc_main_style = Style::new(|s| {
            s.arc_width(CHART_SIZE / 2);
        });
        let arc_indicator_style = Style::new(|s| {
            s.arc_width(0);
        });

        let mut angle_accum: f32 = 0.0;
        let mut arcs_vec = Vec::with_capacity(NUM_SLICES);
        let mut labels_vec = Vec::with_capacity(NUM_SLICES);
        let mut slices_vec = Vec::with_capacity(NUM_SLICES);

        for i in 0..NUM_SLICES {
            let pct = PERCENTAGES[i];
            let slice_angle = (pct as f32 * 360.0) / 100.0;
            let start = (angle_accum + 0.5) as i32;
            angle_accum += slice_angle;
            let mut end = (angle_accum + 0.5) as i32;
            if end > 360 {
                end = 360;
            }

            let arc = Arc::new(&cont)?;
            arc.size(CHART_SIZE, CHART_SIZE).center();
            arc.set_mode(ArcMode::Normal);
            arc.set_bg_start_angle(start);
            arc.set_bg_end_angle(end);

            // Main part = filled pie, indicator = invisible
            arc.add_style(&arc_main_style, Part::Main);
            arc.add_style(&arc_indicator_style, Part::Indicator);
            // Per-slice color (differs per arc, so not shareable across slices).
            let arc_color_style = Style::new(|s| {
                s.arc_color(palette_main(COLORS[i])).arc_rounded(false);
            });
            arc.add_style(&arc_color_style, Part::Main);
            arc.remove_style(None, Part::Knob);
            arc.add_flag(ObjFlag::ADV_HITTEST);
            arc.bubble_events();

            // Percentage label at midpoint
            let label = Label::new(&arc)?;
            let mut buf = heapless::String::<8>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", pct));
            label.text(&buf);

            let mid_angle = start + (end - start) / 2;
            let radius = CHART_SIZE / 4;
            let x_off = (radius * trigo_cos(mid_angle)) >> TRIGO_SHIFT;
            let y_off = (radius * trigo_sin(mid_angle)) >> TRIGO_SHIFT;
            label.align(Align::Center, x_off, y_off);

            let home_x = arc.get_x();
            let home_y = arc.get_y();

            arcs_vec.push(arc);
            labels_vec.push(label);
            slices_vec.push(SliceInfo { mid_angle, home_x, home_y, out: false });
        }

        let arcs: [Arc<'static>; NUM_SLICES] = arcs_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;
        let labels: [Label<'static>; NUM_SLICES] = labels_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;
        let slices: [SliceInfo; NUM_SLICES] = slices_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;

        self._container = Some(cont);
        self.arcs = Some(arcs);
        self._labels = Some(labels);
        self.slices = Some(slices);
        self.active = None;
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref arcs) = self.arcs {
            let handles: [_; NUM_SLICES] = core::array::from_fn(|i| arcs[i].handle());
            for h in handles {
                register_event_on(self, h);
            }
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::CLICKED {
            return NavAction::None;
        }
        let target = event.target_handle();

        let Some(ref arcs) = self.arcs else { return NavAction::None };

        // Find which slice was clicked
        let Some(idx) = arcs.iter().position(|a| a.handle() == target) else {
            return NavAction::None;
        };

        let Some(ref mut slices) = self.slices else { return NavAction::None };

        // If another slice is currently out, animate it back
        if let Some(prev) = self.active {
            if prev != idx && slices[prev].out {
                let home_x = slices[prev].home_x;
                let home_y = slices[prev].home_y;
                self.animate_slice(prev, home_x, home_y);
                if let Some(ref mut slices) = self.slices { slices[prev].out = false; }
            }
        }

        let Some(ref mut slices) = self.slices else { return NavAction::None };
        let info = &slices[idx];
        if info.out {
            let home_x = info.home_x;
            let home_y = info.home_y;
            self.animate_slice(idx, home_x, home_y);
            if let Some(ref mut slices) = self.slices { slices[idx].out = false; }
            self.active = None;
        } else {
            let mid_angle = info.mid_angle;
            let home_x = info.home_x;
            let home_y = info.home_y;
            let x_off = (SLICE_OFFSET * trigo_cos(mid_angle)) >> TRIGO_SHIFT;
            let y_off = (SLICE_OFFSET * trigo_sin(mid_angle)) >> TRIGO_SHIFT;
            self.animate_slice(idx, home_x + x_off, home_y + y_off);
            if let Some(ref mut slices) = self.slices { slices[idx].out = true; }
            self.active = Some(idx);
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetArc3::default());
