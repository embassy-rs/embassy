#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 2 — Runtime image recoloring
//!
//! Cogwheel image with RGB + intensity sliders controlling recolor tint.

use oxivgl::{
    style::{color_make, palette_main, Palette, Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, Image, Part, Slider, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct WidgetImage2 {
    slider_r: Option<Slider<'static>>,
    slider_g: Option<Slider<'static>>,
    slider_b: Option<Slider<'static>>,
    slider_i: Option<Slider<'static>>,
    img: Option<Image<'static>>,
}

impl View for WidgetImage2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Create image on the right side
        let img = Image::new(container)?;
        img.set_src(img_cogwheel_argb());
        img.align(Align::RightMid, -80, 0);

        // Helper to create a vertical slider
        let make_slider =
            |parent: &Obj<'static>, x: i32, initial: i32| -> Result<Slider<'static>, WidgetError> {
                let s = Slider::new(parent)?;
                s.set_range(0, 255);
                s.set_value(initial);
                s.size(10, 200);
                s.align(Align::LeftMid, x, 0);
                Ok(s)
            };

        let knob_r = Style::new(|s| {
            s.bg_color(palette_main(Palette::Red));
        });
        let knob_g = Style::new(|s| {
            s.bg_color(palette_main(Palette::Green));
        });
        let knob_b = Style::new(|s| {
            s.bg_color(palette_main(Palette::Blue));
        });

        let slider_r = make_slider(container, 20, 51)?;
        slider_r.add_style(&knob_r, Part::Knob);

        let slider_g = make_slider(container, 50, 230)?;
        slider_g.add_style(&knob_g, Part::Knob);

        let slider_b = make_slider(container, 80, 153)?;
        slider_b.add_style(&knob_b, Part::Knob);

        let slider_i = make_slider(container, 110, 128)?;

                self.slider_r = Some(slider_r);
        self.slider_g = Some(slider_g);
        self.slider_b = Some(slider_b);
        self.slider_i = Some(slider_i);
        self.img = Some(img);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let (Some(sr), Some(sg), Some(sb), Some(si), Some(img)) = (
            &self.slider_r, &self.slider_g, &self.slider_b, &self.slider_i, &self.img,
        ) {
            let r = sr.get_value() as u8;
            let g = sg.get_value() as u8;
            let b = sb.get_value() as u8;
            let intense = si.get_value() as u8;

            let color = color_make(r, g, b);
            // runtime-varying style; must stay inline
            #[allow(deprecated)]
            img.style_image_recolor(color, Selector::DEFAULT)
                .style_image_recolor_opa(intense, Selector::DEFAULT);
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetImage2::default());
