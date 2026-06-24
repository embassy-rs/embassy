#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid 3 — Demonstrate grid's "free unit" (FR)

use oxivgl::{
    layout::{grid_fr, GridAlign, GridCell, GRID_TEMPLATE_LAST},
    view::{NavAction, View},
    widgets::{Label, Obj, WidgetError},
};

static COL_DSC: [i32; 4] = [60, grid_fr(1), grid_fr(2), GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 4] = [50, grid_fr(1), 50, GRID_TEMPLATE_LAST];

#[derive(Default)]
struct Grid3 {
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 9>,
    _labels: heapless::Vec<Label<'static>, 9>,
}

impl View for Grid3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(300, 220).center();
        cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);

        let mut items = heapless::Vec::<Obj<'static>, 9>::new();
        let mut labels = heapless::Vec::<Label<'static>, 9>::new();

        for i in 0..9u32 {
            let col = (i % 3) as i32;
            let row = (i / 3) as i32;

            let obj = Obj::new(&cont)?;
            obj.set_grid_cell(
                GridCell::new(GridAlign::Stretch, col, 1),
                GridCell::new(GridAlign::Stretch, row, 1),
            );

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<8>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{},{}", col, row));
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

oxivgl_examples_common::example_main!(Grid3::default());
