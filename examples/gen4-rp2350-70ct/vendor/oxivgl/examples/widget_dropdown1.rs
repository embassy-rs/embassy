#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 1 — Simple drop-down list
//!
//! A centered dropdown with fruit options and a label showing the selected
//! item, updated via VALUE_CHANGED event.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Obj, Align, Dropdown, Label, WidgetError},
};

#[derive(Default)]
struct WidgetDropdown1 {
    dd: Option<Dropdown<'static>>,
    label: Option<Label<'static>>,
}

impl View for WidgetDropdown1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let dd = Dropdown::new(container)?;
        dd.set_options(
            "Apple\n\
             Banana\n\
             Orange\n\
             Cherry\n\
             Grape\n\
             Raspberry\n\
             Melon\n\
             Lemon\n\
             Nuts",
        );
        dd.align(Align::TopMid, 0, 20);
        dd.bubble_events();

        let label = Label::new(container)?;
        label.text("Apple");
        label.align(Align::BottomMid, 0, -20);

                self.dd = Some(dd);
        self.label = Some(label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref dd) = self.dd { register_event_on(self, dd.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::VALUE_CHANGED {
            if let (Some(dd), Some(label)) = (&self.dd, &self.label) {
                let mut buf = [0u8; 32];
                if let Some(text) = dd.get_selected_str(&mut buf) {
                    label.text(text);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown1::default());
