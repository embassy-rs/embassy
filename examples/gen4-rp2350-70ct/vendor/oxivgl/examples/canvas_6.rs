#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canvas 6 — Draw an image onto a canvas

use oxivgl::{
    draw::{Area, DrawImageDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct Canvas6 {
    _canvas: Option<Canvas<'static>>,
}

impl View for Canvas6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Canvas large enough for the 100x100 cogwheel image
        let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(container, buf)?;
        canvas.fill_bg(color_make(0xcf, 0xcf, 0xcf), 255);
        canvas.align(Align::Center, 0, 0);

        {
            let mut layer = canvas.init_layer();
            let dsc = DrawImageDsc::from_static_dsc(img_cogwheel_argb());
            layer.draw_image(&dsc, Area { x1: 0, y1: 0, x2: 99, y2: 99 });
        }

                self._canvas = Some(canvas);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {}
    fn on_event(&mut self, _: &oxivgl::event::Event) -> NavAction { NavAction::None }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas6::default());
