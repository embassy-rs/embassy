#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Trickle — Demonstrate event trickle-down
//!
//! Container with `EVENT_TRICKLE` flag and 9 sub-objects. Pressing the
//! container trickles the PRESSED state to children (black background).
//! Focusing a child applies FOCUSED state (also black background).

use oxivgl::{
    style::{color_black, color_white, Style, StyleBuilder},
    view::{NavAction, View},
    enums::{ObjFlag, ObjState},
    layout::FlexFlow,
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct EventTrickle {
    _cont: Option<Obj<'static>>,
    _style_black: Option<Style>,
    _subconts: heapless::Vec<Obj<'static>, 9>,
    _labels: heapless::Vec<Label<'static>, 9>,
}

impl View for EventTrickle {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut style_black = StyleBuilder::new();
        style_black
            .text_color(color_white())
            .bg_color(color_black());
        let style_black = style_black.build();

        let cont = Obj::new(container)?;
        cont.size(290, 200).center();
        cont.set_flex_flow(FlexFlow::RowWrap);
        cont.add_flag(ObjFlag::EVENT_TRICKLE);
        cont.add_style(&style_black, ObjState::PRESSED);

        let mut subconts = heapless::Vec::<Obj<'static>, 9>::new();
        let mut labels = heapless::Vec::<Label<'static>, 9>::new();

        for i in 0..9u32 {
            let subcont = Obj::new(&cont)?;
            subcont.size(70, 50);
            subcont.add_style(&style_black, ObjState::FOCUSED);

            let label = Label::new(&subcont)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf);

            let _ = subconts.push(subcont);
            let _ = labels.push(label);
        }

                self._cont = Some(cont);
        self._style_black = Some(style_black);
        self._subconts = subconts;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(EventTrickle::default());
