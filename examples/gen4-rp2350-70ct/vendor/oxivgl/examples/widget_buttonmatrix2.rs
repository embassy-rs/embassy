#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Buttonmatrix 2 — Custom draw per button
//!
//! Default button matrix with per-button draw customization via
//! `DRAW_TASK_ADDED`. Button 2 is blue with shadow, button 3 is red
//! with circular radius, button 4 hides its label and shows a star image.

oxivgl::image_declare!(img_star);

use oxivgl::view::NavAction;
use oxivgl::{
    draw::{image_header_info, Area, DrawImageDsc, DRAW_TASK_TYPE_FILL, RADIUS_CIRCLE},
    enums::{EventCode, ObjState},
    event::Event,
    style::{color_white, palette_darken, palette_main, Palette},
    view::{register_event_on, View},
    widgets::{Obj, Align, Buttonmatrix, Part, WidgetError},
};

#[derive(Default)]
struct WidgetButtonmatrix2 {
    btnm: Option<Buttonmatrix<'static>>,
}

impl View for WidgetButtonmatrix2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btnm = Buttonmatrix::new(container)?;
        btnm.center();
        btnm.send_draw_task_events();

        self.btnm = Some(btnm);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref btnm) = self.btnm {
            register_event_on(self, btnm.handle());
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

        let pressed = if let Some(ref btnm) = self.btnm {
            btnm.get_selected_button() == base.id1 && btnm.has_state(ObjState::PRESSED)
        } else {
            false
        };

        if base.id1 == 1 {
            // Blue fill, no radius, shadow, white label.
            task.with_fill_dsc(|dsc| {
                dsc.set_radius(0);
                dsc.set_color(if pressed {
                    palette_darken(Palette::Blue, 3)
                } else {
                    palette_main(Palette::Blue)
                });
            });
            task.with_box_shadow_dsc(|dsc| {
                dsc.set_width(6);
                dsc.set_ofs_x(3);
                dsc.set_ofs_y(3);
            });
            task.with_label_dsc(|dsc| {
                dsc.set_color(color_white());
            });
        } else if base.id1 == 2 {
            // Red fill, circular radius.
            task.with_fill_dsc(|dsc| {
                dsc.set_radius(RADIUS_CIRCLE);
                dsc.set_color(if pressed {
                    palette_darken(Palette::Red, 3)
                } else {
                    palette_main(Palette::Red)
                });
            });
            task.with_box_shadow_dsc(|dsc| {
                dsc.set_radius(RADIUS_CIRCLE);
            });
        } else if base.id1 == 3 {
            // Hide label, draw star image on fill pass.
            task.with_label_dsc(|dsc| {
                dsc.set_opa(0);
            });
            // Draw star only during the fill draw pass.
            if task.task_type() == DRAW_TASK_TYPE_FILL {
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
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetButtonmatrix2::default());
