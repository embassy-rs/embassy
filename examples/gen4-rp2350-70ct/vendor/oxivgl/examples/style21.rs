#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 21 — Material-design cards with shadow, rotation & scale transforms
//!
//! Two card objects with rounded corners and drop shadow. An Arc controls
//! rotation transform; a Slider controls scale transform on both cards.

use oxivgl::{
    enums::EventCode,
    event::Event,
    layout::FlexFlow,
    style::{color_make, Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Arc, Label, Obj, Slider, WidgetError},
};

#[derive(Default)]
struct Style21 {
    card1: Option<Obj<'static>>,
    card2: Option<Obj<'static>>,
    arc: Option<Arc<'static>>,
    slider: Option<Slider<'static>>,
    _lbl1: Option<Label<'static>>,
    _lbl2: Option<Label<'static>>,
    _arc_lbl: Option<Label<'static>>,
    _slider_lbl: Option<Label<'static>>,
}

impl View for Style21 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        // Shared screen background style.
        let bg_style = Style::new(|s| {
            s.bg_color_hex(0xeeeeee).bg_opa(255);
        });
        container.add_style(&bg_style, Selector::DEFAULT);

        // Shared card style — identical for both cards (one property buffer).
        let card_style = Style::new(|s| {
            s.bg_color_hex(0xffffff)
                .bg_opa(255)
                .radius(12)
                .border_width(0)
                .shadow_width(20)
                .shadow_color(color_make(0x88, 0x88, 0x88))
                .shadow_offset_x(2)
                .shadow_offset_y(4)
                .shadow_spread(0)
                .shadow_opa(200);
        });

        // Shared transparent-row style — the two control rows.
        let row_style = Style::new(|s| {
            s.bg_opa(0).border_width(0);
        });

        // Controls container style — transparent like the rows, plus padding.
        let controls_style = Style::new(|s| {
            s.bg_opa(0).border_width(0).pad_all(4).pad_row(8);
        });

        // --- Card 1 ---
        let card1 = Obj::new(container)?;
        card1.size(120, 80);
        card1.align(Align::TopLeft, 20, 20);
        card1.add_style(&card_style, Selector::DEFAULT);
        card1.style_transform_pivot_x(60, Selector::DEFAULT);
        card1.style_transform_pivot_y(40, Selector::DEFAULT);

        let lbl1 = Label::new(&card1)?;
        lbl1.text("Card A").center();

        // --- Card 2 ---
        let card2 = Obj::new(container)?;
        card2.size(120, 80);
        card2.align(Align::TopRight, -20, 20);
        card2.add_style(&card_style, Selector::DEFAULT);
        card2.style_transform_pivot_x(60, Selector::DEFAULT);
        card2.style_transform_pivot_y(40, Selector::DEFAULT);

        let lbl2 = Label::new(&card2)?;
        lbl2.text("Card B").center();

        // --- Controls container (bottom half) ---
        let controls = Obj::new(container)?;
        controls.size(280, 100);
        controls.align(Align::BottomMid, 0, -10);
        controls.add_style(&controls_style, Selector::DEFAULT);
        controls.set_flex_flow(FlexFlow::Column);

        // Arc — controls rotation
        let arc_row = Obj::new(&controls)?;
        arc_row.size(260, 40);
        arc_row.add_style(&row_style, Selector::DEFAULT);
        arc_row.set_flex_flow(FlexFlow::Row);

        let arc_lbl = Label::new(&arc_row)?;
        arc_lbl.text("Rot:");

        let arc = Arc::new(&arc_row)?;
        arc.size(40, 40);
        arc.set_rotation(270);
        arc.set_bg_angles(0, 360);
        arc.set_range_raw(0, 3600); // 0..360 degrees in 0.1 units
        arc.set_value_raw(450); // 45 deg for screenshot
        arc.bubble_events();

        // Slider — controls scale
        let slider_row = Obj::new(&controls)?;
        slider_row.size(260, 40);
        slider_row.add_style(&row_style, Selector::DEFAULT);
        slider_row.set_flex_flow(FlexFlow::Row);

        let slider_lbl = Label::new(&slider_row)?;
        slider_lbl.text("Scl:");

        let slider = Slider::new(&slider_row)?;
        slider.size(180, 10);
        slider.center();
        slider.set_range(128, 512); // 0.5x .. 2.0x  (256 = 1.0)
        slider.set_value(256); // 1.0x for default
        slider.bubble_events();

        // Apply initial transforms for screenshot
        card1.style_transform_rotation(450, Selector::DEFAULT);
        card2.style_transform_rotation(450, Selector::DEFAULT);

                self.card1 = Some(card1);
        self.card2 = Some(card2);
        self.arc = Some(arc);
        self.slider = Some(slider);
        self._lbl1 = Some(lbl1);
        self._lbl2 = Some(lbl2);
        self._arc_lbl = Some(arc_lbl);
        self._slider_lbl = Some(slider_lbl);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref arc) = self.arc {
            if event.matches(arc, EventCode::VALUE_CHANGED) {
                let angle = arc.get_value_raw();
                if let Some(ref card1) = self.card1 { card1.style_transform_rotation(angle, Selector::DEFAULT); }
                if let Some(ref card2) = self.card2 { card2.style_transform_rotation(angle, Selector::DEFAULT); }
            }
        }
        if let Some(ref slider) = self.slider {
            if event.matches(slider, EventCode::VALUE_CHANGED) {
                let scale = slider.get_value();
                if let Some(ref card1) = self.card1 {
                    card1.style_transform_scale_x(scale, Selector::DEFAULT);
                    card1.style_transform_scale_y(scale, Selector::DEFAULT);
                }
                if let Some(ref card2) = self.card2 {
                    card2.style_transform_scale_x(scale, Selector::DEFAULT);
                    card2.style_transform_scale_y(scale, Selector::DEFAULT);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style21::default());
