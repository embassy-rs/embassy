#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 3 — 3D matrix transform
//!
//! Centered object with animated scale + rotation via matrix transform.
//! Scale grows from 0.1 to 2.0, rotation follows `value * 360°`.
//! Resets and repeats at 2.0.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Matrix, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetObj3 {
    obj: Option<Obj<'static>>,
    value: f32,
}

impl View for WidgetObj3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let obj = Obj::new(container)?;
        obj.center();

                self.obj = Some(obj);
        self.value = 0.1;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        self.value += 0.01;

        if let Some(ref obj) = self.obj {
            if self.value > 2.0 {
                obj.reset_transform();
                self.value = 0.1;
            } else {
                let mut m = Matrix::identity();
                m.scale(self.value, 1.0).rotate(self.value * 360.0);
                obj.set_transform(&m);
            }
        }

        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetObj3::default());
