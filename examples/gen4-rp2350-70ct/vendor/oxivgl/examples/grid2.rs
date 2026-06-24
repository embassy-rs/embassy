#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid 2 — Demonstrate cell placement and span

use oxivgl::{
    style::LV_SIZE_CONTENT,
    view::{NavAction, View},
    layout::{GridAlign, GridCell, GRID_TEMPLATE_LAST},
    widgets::{Label, Obj, WidgetError},
};

static COL_DSC: [i32; 4] = [70, 70, 70, GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 4] = [50, 50, 50, GRID_TEMPLATE_LAST];

#[derive(Default)]
struct Grid2 {
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 5>,
    _labels: heapless::Vec<Label<'static>, 5>,
}

impl View for Grid2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);
        cont.size(300, 220).center();

        let mut items = heapless::Vec::<Obj<'static>, 5>::new();
        let mut labels = heapless::Vec::<Label<'static>, 5>::new();

        // Cell 0,0 — START/START
        let obj = Obj::new(&cont)?;
        obj.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        obj.set_grid_cell(
            GridCell::new(GridAlign::Start, 0, 1),
            GridCell::new(GridAlign::Start, 0, 1),
        );
        let lbl = Label::new(&obj)?;
        lbl.text("c0, r0");
        let _ = items.push(obj);
        let _ = labels.push(lbl);

        // Cell 1,0 — START/CENTER
        let obj = Obj::new(&cont)?;
        obj.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        obj.set_grid_cell(
            GridCell::new(GridAlign::Start, 1, 1),
            GridCell::new(GridAlign::Center, 0, 1),
        );
        let lbl = Label::new(&obj)?;
        lbl.text("c1, r0");
        let _ = items.push(obj);
        let _ = labels.push(lbl);

        // Cell 2,0 — START/END
        let obj = Obj::new(&cont)?;
        obj.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        obj.set_grid_cell(
            GridCell::new(GridAlign::Start, 2, 1),
            GridCell::new(GridAlign::End, 0, 1),
        );
        let lbl = Label::new(&obj)?;
        lbl.text("c2, r0");
        let _ = items.push(obj);
        let _ = labels.push(lbl);

        // Cell 1-2,1 — spans 2 columns
        let obj = Obj::new(&cont)?;
        obj.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        obj.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 1, 2),
            GridCell::new(GridAlign::Stretch, 1, 1),
        );
        let lbl = Label::new(&obj)?;
        lbl.text("c1-2, r1");
        let _ = items.push(obj);
        let _ = labels.push(lbl);

        // Cell 0,1-2 — spans 2 rows
        let obj = Obj::new(&cont)?;
        obj.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        obj.set_grid_cell(
            GridCell::new(GridAlign::Stretch, 0, 1),
            GridCell::new(GridAlign::Stretch, 1, 2),
        );
        let lbl = Label::new(&obj)?;
        lbl.text("c0\nr1-2");
        let _ = items.push(obj);
        let _ = labels.push(lbl);

                self._cont = Some(cont);
        self._items = items;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Grid2::default());
