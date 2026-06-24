#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Keyboard 3 — Per-key coloring with star icon on OK key
//!
//! A keyboard where each key gets a unique palette color via
//! `DRAW_TASK_ADDED`. The OK key (checkmark symbol) has its label hidden
//! and a star image drawn in its place.

oxivgl::image_declare!(img_star);

use oxivgl::view::NavAction;
use oxivgl::{
    draw::{Area, DrawImageDsc, image_header_info},
    enums::EventCode,
    event::Event,
    style::{palette_lighten, palette_main, Palette},
    view::{register_event_on, View},
    widgets::{Obj, Align, Keyboard, Part, WidgetError},
};

/// Map a button index to a palette cycling through 19 palettes.
fn palette_for(id: u32) -> Palette {
    match id % 19 {
        0 => Palette::Red,
        1 => Palette::Pink,
        2 => Palette::Purple,
        3 => Palette::DeepPurple,
        4 => Palette::Indigo,
        5 => Palette::Blue,
        6 => Palette::LightBlue,
        7 => Palette::Cyan,
        8 => Palette::Teal,
        9 => Palette::Green,
        10 => Palette::LightGreen,
        11 => Palette::Lime,
        12 => Palette::Yellow,
        13 => Palette::Amber,
        14 => Palette::Orange,
        15 => Palette::DeepOrange,
        16 => Palette::Brown,
        17 => Palette::BlueGrey,
        _ => Palette::Grey,
    }
}

// LV_SYMBOL_OK is U+F00C encoded as UTF-8: \xEF\x80\x8C
const SYMBOL_OK_STR: &str = "\u{F00C}";

#[derive(Default)]
struct WidgetKeyboard3 {
    kb: Option<Keyboard<'static>>,
}

impl View for WidgetKeyboard3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let kb = Keyboard::new(container)?;
        kb.center();
        kb.send_draw_task_events();

        self.kb = Some(kb);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref kb) = self.kb {
            register_event_on(self, kb.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::DRAW_TASK_ADDED {
            return NavAction::None;
        }
        let Some(task) = event.draw_task() else { return NavAction::None };
        let base = task.base();
        if base.part != Part::Items {
            return NavAction::None;
        }

        let palette = palette_for(base.id1);

        // Color the key background.
        task.with_fill_dsc(|dsc| {
            dsc.set_color(palette_main(palette));
        });

        // Color the key label, or replace the OK key with a star image.
        task.with_label_dsc(|dsc| {
            let text = dsc.text();
            let is_ok = text.map(|t| t == SYMBOL_OK_STR).unwrap_or(false);

            if is_ok {
                // Hide the label text — draw star image instead.
                dsc.set_opa(0);

                if let Some((w, h)) = image_header_info(img_star()) {
                    if let Some(layer) = task.layer() {
                        let task_area = task.area();
                        let mut img_area = Area {
                            x1: 0,
                            y1: 0,
                            x2: w - 1,
                            y2: h - 1,
                        };
                        img_area.align_to_area(task_area, Align::Center, 0, 0);
                        let img_dsc = DrawImageDsc::from_static_dsc(img_star());
                        layer.draw_image(&img_dsc, img_area);
                    }
                }
            } else {
                dsc.set_color(palette_lighten(palette, 4));
            }
        });
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetKeyboard3::default());
