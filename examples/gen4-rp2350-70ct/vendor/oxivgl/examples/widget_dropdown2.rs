#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 2 — Drop-down in four directions
//!
//! Four dropdowns opening in each cardinal direction.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, DdDir, Dropdown, WidgetError},
};

#[derive(Default)]
struct WidgetDropdown2 {
    _dd_top: Option<Dropdown<'static>>,
    _dd_bottom: Option<Dropdown<'static>>,
    _dd_right: Option<Dropdown<'static>>,
    _dd_left: Option<Dropdown<'static>>,
}

const OPTS: &str = "Apple\nBanana\nOrange\nMelon";

impl View for WidgetDropdown2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Default (opens downward)
        let dd_top = Dropdown::new(container)?;
        dd_top.set_options(OPTS);
        dd_top.align(Align::TopMid, 0, 10);

        // Opens upward
        let dd_bottom = Dropdown::new(container)?;
        dd_bottom.set_options(OPTS);
        dd_bottom.set_dir(DdDir::Top);
        dd_bottom.align(Align::BottomMid, 0, -10);

        // Opens to the right
        let dd_right = Dropdown::new(container)?;
        dd_right.set_options(OPTS);
        dd_right.set_dir(DdDir::Right);
        dd_right.align(Align::LeftMid, 10, 0);

        // Opens to the left
        let dd_left = Dropdown::new(container)?;
        dd_left.set_options(OPTS);
        dd_left.set_dir(DdDir::Left);
        dd_left.align(Align::RightMid, -10, 0);

                self._dd_top = Some(dd_top);
        self._dd_bottom = Some(dd_bottom);
        self._dd_right = Some(dd_right);
        self._dd_left = Some(dd_left);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown2::default());
