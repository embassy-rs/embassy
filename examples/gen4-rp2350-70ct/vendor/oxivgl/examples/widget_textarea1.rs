#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 1 — Numeric keypad
//!
//! One-line textarea with a custom numeric button matrix keypad.
//! Pressing a digit appends it; backspace deletes; newline sends READY.

use oxivgl::view::NavAction;
use oxivgl::{
    btnmatrix_map,
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{register_event_on, View},
    widgets::{Obj, Align, Buttonmatrix, ButtonmatrixMap, Textarea, WidgetError,
    },
};

/// LV_SYMBOL_BACKSPACE (U+F55A)
const SYMBOL_BACKSPACE: &str = "\u{F55A}";
/// LV_SYMBOL_NEW_LINE (U+F8A2)
const SYMBOL_NEW_LINE: &str = "\u{F8A2}";

static BTNM_MAP: &ButtonmatrixMap = btnmatrix_map!(
    c"1", c"2", c"3", c"\n",
    c"4", c"5", c"6", c"\n",
    c"7", c"8", c"9", c"\n",
    c"\u{F55A}", c"0", c"\u{F8A2}"
);

#[derive(Default)]
struct WidgetTextarea1 {
    ta: Option<Textarea<'static>>,
    btnm: Option<Buttonmatrix<'static>>,
}

impl View for WidgetTextarea1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let ta = Textarea::new(container)?;
        ta.set_one_line(true);
        ta.align(Align::TopMid, 0, 10);
        ta.add_state(ObjState::FOCUSED);
        ta.bubble_events();

        let btnm = Buttonmatrix::new(container)?;
        btnm.set_map(BTNM_MAP);
        btnm.size(200, 150);
        btnm.align(Align::BottomMid, 0, -10);
        btnm.remove_flag(ObjFlag::CLICK_FOCUSABLE); // Keep textarea focused on button clicks
        btnm.bubble_events();

                self.ta = Some(ta);
        self.btnm = Some(btnm);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref btnm) = self.btnm {
            register_event_on(self, btnm.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref btnm) = self.btnm {
            if event.matches(btnm, EventCode::VALUE_CHANGED) {
                let btn_id = btnm.get_selected_button();
                if let Some(txt) = btnm.get_button_text(btn_id) {
                    if let Some(ref ta) = self.ta {
                        if txt == SYMBOL_BACKSPACE {
                            ta.delete_char();
                        } else if txt == SYMBOL_NEW_LINE {
                            ta.send_event(EventCode::READY);
                        } else {
                            ta.add_text(txt);
                        }
                    }
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea1::default());
