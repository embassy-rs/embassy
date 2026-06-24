#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 6 — RTL base direction changes order of the items

use oxivgl::{
    style::{Selector, Style, LV_SIZE_CONTENT},
    view::{NavAction, View},
    layout::FlexFlow,
    widgets::{BaseDir, Label, Obj, WidgetError},
};

#[derive(Default)]
struct Flex6 {
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 20>,
    _labels: heapless::Vec<Label<'static>, 20>,
}

impl View for Flex6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        let cont_style = Style::new(|s| {
            s.base_dir(BaseDir::Rtl);
        });
        cont.add_style(&cont_style, Selector::DEFAULT);
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::RowWrap);

        let mut items = heapless::Vec::<Obj<'static>, 20>::new();
        let mut labels = heapless::Vec::<Label<'static>, 20>::new();

        for i in 0..20u32 {
            let obj = Obj::new(&cont)?;
            obj.size(70, LV_SIZE_CONTENT);

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf).center();

            let _ = items.push(obj);
            let _ = labels.push(label);
        }

                self._cont = Some(cont);
        self._items = items;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Flex6::default());
