#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Buttonmatrix 1 — Basic numpad
//!
//! A 3-row button matrix: digits 1-5, 6-0, and two action buttons.
//! Action1 is 2x wide and checkable; Action2 starts checked.

use oxivgl::{
    btnmatrix_map,
    view::{NavAction, View},
    widgets::{
        Obj, Align, Buttonmatrix, ButtonmatrixCtrl, ButtonmatrixMap, WidgetError,
    },
};

static MAP: &ButtonmatrixMap = btnmatrix_map!(
    c"1", c"2", c"3", c"4", c"5", c"\n",
    c"6", c"7", c"8", c"9", c"0", c"\n",
    c"Action1", c"Action2"
);

#[derive(Default)]
struct WidgetButtonmatrix1 {
    _btnm: Option<Buttonmatrix<'static>>,
}

impl View for WidgetButtonmatrix1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btnm = Buttonmatrix::new(container)?;
        btnm.set_map(MAP);
        btnm.size(240, 150);
        btnm.align(Align::Center, 0, 0);

        // Action1 (btn_id=10) is 2x wide and checkable
        btnm.set_button_width(10, 2);
        btnm.set_button_ctrl(10, ButtonmatrixCtrl::CHECKABLE);

        // Action2 (btn_id=11) starts checked
        btnm.set_button_ctrl(11, ButtonmatrixCtrl::CHECKABLE);
        btnm.set_button_ctrl(11, ButtonmatrixCtrl::CHECKED);

                self._btnm = Some(btnm);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetButtonmatrix1::default());
