#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Win 1 — Window with header buttons, title, and scrollable content.
//!
//! Port of `lv_example_win_1.c`. Three header buttons (left, right, close)
//! with a title, and a long label in the content area to demonstrate scrolling.

use oxivgl::{
    symbols,
    view::{NavAction, View},
    widgets::{Obj, Label, WidgetError, Win},
};

#[derive(Default)]
struct Win1 {
    _win: Option<Win<'static>>,
}

impl View for Win1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let win = Win::new(container)?;

        let _btn1 = win.add_button(&symbols::LEFT, 40);
        let _title = win.add_title("A title");
        let _btn2 = win.add_button(&symbols::RIGHT, 40);
        let _btn3 = win.add_button(&symbols::CLOSE, 60);

        let content = win.get_content();
        let label = Label::new(&content)?;
        label.text(
            "This is\n\
             a pretty\n\
             long text\n\
             to see how\n\
             the window\n\
             becomes\n\
             scrollable.\n\
             \n\
             \n\
             Some more\n\
             text to be\n\
             sure it\n\
             overflows. :)",
        );

                self._win = Some(win);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Win1::default());
