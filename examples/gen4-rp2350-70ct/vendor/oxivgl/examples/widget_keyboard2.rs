#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Keyboard 2 — Custom AZERTY layout
//!
//! A keyboard using a custom AZERTY key map assigned to User1 mode.
//! A textarea at top-mid receives the key presses.

use oxivgl::{
    btnmatrix_map,
    view::{NavAction, View},
    widgets::{
        Obj, Align, ButtonmatrixCtrl, ButtonmatrixMap, Keyboard, KeyboardMode, Textarea,
        WidgetError,
    },
};

static AZERTY_MAP: &ButtonmatrixMap = btnmatrix_map!(
    c"a", c"z", c"e", c"r", c"t", c"y", c"u", c"i", c"o", c"p", c"\n",
    c"q", c"s", c"d", c"f", c"g", c"h", c"j", c"k", c"l", c"m", c"\n",
    c"ABC", c"w", c"x", c"c", c"v", c"b", c"n", c".", c",", c"\n",
    c"#1", c" ", c"\xEF\x95\x9A"
);

// Control flags — all default
static AZERTY_CTRL: &[ButtonmatrixCtrl] = &[
    // Row 1: a z e r t y u i o p (10 keys)
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE,
    // Row 2: q s d f g h j k l m (10 keys)
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE,
    // Row 3: ABC w x c v b n . , (9 keys)
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
    // Row 4: #1 (space) backspace (3 keys)
    ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE, ButtonmatrixCtrl::NONE,
];

#[derive(Default)]
struct WidgetKeyboard2 {
    _ta: Option<Textarea<'static>>,
    _kb: Option<Keyboard<'static>>,
}

impl View for WidgetKeyboard2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let ta = Textarea::new(container)?;
        ta.size(280, 80);
        ta.align(Align::TopMid, 0, 10);
        ta.set_placeholder_text("Type here...");

        let kb = Keyboard::new(container)?;
        kb.set_textarea(&ta);
        kb.set_map(KeyboardMode::User1, AZERTY_MAP, AZERTY_CTRL);
        kb.set_mode(KeyboardMode::User1);

                self._ta = Some(ta);
        self._kb = Some(kb);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetKeyboard2::default());
