#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 5 — Demonstrate the effect of column and row gap style properties

use oxivgl::{
    anim::{anim_set_pad_column, anim_set_pad_row, Anim, ANIM_REPEAT_INFINITE},
    style::LV_SIZE_CONTENT,
    view::{NavAction, View},
    layout::FlexFlow,
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct Flex5 {
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 9>,
    _labels: heapless::Vec<Label<'static>, 9>,
}

impl View for Flex5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::RowWrap);

        let mut items = heapless::Vec::<Obj<'static>, 9>::new();
        let mut labels = heapless::Vec::<Label<'static>, 9>::new();

        for i in 0..9u32 {
            let obj = Obj::new(&cont)?;
            obj.size(70, LV_SIZE_CONTENT);

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf).center();

            let _ = items.push(obj);
            let _ = labels.push(label);
        }

        // Animate row gap
        let mut a = Anim::new();
        a.set_var(&cont)
            .set_values(0, 10)
            .set_repeat_count(ANIM_REPEAT_INFINITE);

        a.set_exec_cb(Some(anim_set_pad_row))
            .set_duration(500)
            .set_reverse_duration(500);
        a.start();

        a.set_exec_cb(Some(anim_set_pad_column))
            .set_duration(3000)
            .set_reverse_duration(3000);
        a.start();

                self._cont = Some(cont);
        self._items = items;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Flex5::default());
