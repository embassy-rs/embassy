#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Spinbox 1 — Numeric input with +/− buttons
//!
//! Spinbox with range −1000..25000, 5 digits and 2 decimal places.
//! Plus and minus buttons on either side increment/decrement the value.

use oxivgl::{
    enums::EventCode,
    event::Event,
    style::Selector,
    symbols,
    view::{NavAction, View},
    widgets::{Obj, Align, Button, Spinbox, WidgetError},
};

#[derive(Default)]
struct Spinbox1 {
    spinbox: Option<Spinbox<'static>>,
    btn_plus: Option<Button<'static>>,
    btn_minus: Option<Button<'static>>,
}

impl View for Spinbox1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let spinbox = Spinbox::new(container)?;
        spinbox
            .set_range(-1000, 25000)
            .set_digit_format(5, 2)
            .step_prev();
        spinbox.width(100).center();

        let h = spinbox.get_height();

        let btn_plus = Button::new(container)?;
        btn_plus.size(h, h).align_to(&spinbox, Align::OutRightMid, 5, 0);
        btn_plus.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);
        btn_plus.bubble_events();

        let btn_minus = Button::new(container)?;
        btn_minus.size(h, h).align_to(&spinbox, Align::OutLeftMid, -5, 0);
        btn_minus.style_bg_image_src_symbol(&symbols::MINUS, Selector::DEFAULT);
        btn_minus.bubble_events();

                self.spinbox = Some(spinbox);
        self.btn_plus = Some(btn_plus);
        self.btn_minus = Some(btn_minus);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(btn_plus), Some(spinbox)) = (&self.btn_plus, &self.spinbox) {
            if event.matches(btn_plus, EventCode::SHORT_CLICKED) {
                spinbox.increment();
            }
        }
        if let (Some(btn_minus), Some(spinbox)) = (&self.btn_minus, &self.spinbox) {
            if event.matches(btn_minus, EventCode::SHORT_CLICKED) {
                spinbox.decrement();
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Spinbox1::default());
