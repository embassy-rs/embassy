#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget LED 1 — LED brightness and color
//!
//! Three LEDs: off, dim red (brightness 150), and full on.

use oxivgl::{
    style::{palette_main, Palette},
    view::{NavAction, View},
    widgets::{Obj, Align, Led, WidgetError},
};

#[derive(Default)]
struct WidgetLed1 {
    _led1: Option<Led<'static>>,
    _led2: Option<Led<'static>>,
    _led3: Option<Led<'static>>,
}

impl View for WidgetLed1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let led1 = Led::new(container)?;
        led1.align(Align::Center, -80, 0);
        led1.off();

        let led2 = Led::new(container)?;
        led2.align(Align::Center, 0, 0);
        led2.set_brightness(150);
        led2.set_color(palette_main(Palette::Red));

        let led3 = Led::new(container)?;
        led3.align(Align::Center, 80, 0);
        led3.on();

                self._led1 = Some(led1);
        self._led2 = Some(led2);
        self._led3 = Some(led3);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLed1::default());
