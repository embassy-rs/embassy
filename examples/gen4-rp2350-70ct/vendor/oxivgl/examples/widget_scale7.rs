#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 7 — Custom major tick label color and text
//!
//! Horizontal scale where a `DRAW_TASK_ADDED` handler recolors each major
//! tick label with a rainbow palette and reformats the numeric text as a
//! one-decimal float.

use oxivgl::view::NavAction;
use oxivgl::{
    draw::DrawTask,
    enums::EventCode,
    event::Event,
    style::{lv_pct, palette_main, Palette, Style},
    view::{register_event_on, View},
    widgets::{Obj, Part, Scale, ScaleMode, WidgetError},
};

const RAINBOW: [Palette; 7] = [
    Palette::Red,
    Palette::Orange,
    Palette::Yellow,
    Palette::Green,
    Palette::Cyan,
    Palette::Blue,
    Palette::Purple,
];

#[derive(Default)]
struct WidgetScale7 {
    scale: Option<Scale<'static>>,
}

impl WidgetScale7 {
    fn handle_draw_task(&self, draw_task: &DrawTask) {
        let base = draw_task.base();
        if base.part != Part::Indicator {
            return;
        }
        let Some(label_dsc) = draw_task.label_dsc() else { return };
        let Some(ref scale) = self.scale else { return };

        // Color each major tick label by index.
        let major_every = scale.get_major_tick_every() as u32;
        if major_every > 0 {
            let color_idx = (base.id1 / major_every) as usize;
            if color_idx < RAINBOW.len() {
                label_dsc.set_color(palette_main(RAINBOW[color_idx]));
            }
        }

        // Reformat the label text as a one-decimal float.
        let mut buf = heapless::String::<20>::new();
        let val = base.id2 as f32;
        // Format as X.Y (one decimal).
        let int_part = val as i32;
        let frac_part = ((val - int_part as f32) * 10.0) as i32;
        let _ = core::fmt::Write::write_fmt(
            &mut buf,
            format_args!("{}.{}", int_part, frac_part.abs()),
        );
        label_dsc.set_text(&buf);

        // Adjust the draw area to fit the new (potentially wider) text.
        let (new_w, _) = label_dsc.text_size(&buf);
        let mut area = draw_task.area();
        area.set_width_centered(new_w);
        draw_task.set_area(area);
    }
}

impl View for WidgetScale7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(lv_pct(80), 100);
        scale.set_mode(ScaleMode::HorizontalBottom);
        scale.center();

        scale.set_label_show(true);
        scale.set_total_tick_count(31);
        scale.set_major_tick_every(5);

        let minor_len = Style::new(|s| {
            s.length(5);
        });
        let major_len = Style::new(|s| {
            s.length(10);
        });
        scale.add_style(&minor_len, Part::Items);
        scale.add_style(&major_len, Part::Indicator);
        scale.set_range(10, 40);

        scale.send_draw_task_events();

                self.scale = Some(scale);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref scale) = self.scale { register_event_on(self, scale.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::DRAW_TASK_ADDED {
            if let Some(draw_task) = event.draw_task() {
                self.handle_draw_task(&draw_task);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale7::default());
