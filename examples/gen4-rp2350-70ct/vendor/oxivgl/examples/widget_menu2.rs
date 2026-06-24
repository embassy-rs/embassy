#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Menu 2 — Root back button with message box
//!
//! Like menu1 but with the root back button enabled. Clicking the back button
//! at root level shows a message box.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{NavAction, View},
    widgets::{Label, Menu, Msgbox, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetMenu2 {
    menu: Option<Menu<'static>>,
    _labels: Option<[Label<'static>; 4]>,
}

impl View for WidgetMenu2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let menu = Menu::new(container)?;
        menu.set_mode_root_back_button(true);
        menu.bubble_events();
        menu.size(320, 240).center();

        // Sub-page
        let sub_page = menu.page_create(None);
        let cont = Menu::cont_create(&sub_page);
        let l0 = Label::new(&cont)?;
        l0.text("Hello, I am hiding here");

        // Main page
        let main_page = menu.page_create(None);

        let cont1 = Menu::cont_create(&main_page);
        let l1 = Label::new(&cont1)?;
        l1.text("Item 1");

        let cont2 = Menu::cont_create(&main_page);
        let l2 = Label::new(&cont2)?;
        l2.text("Item 2");

        let cont3 = Menu::cont_create(&main_page);
        let l3 = Label::new(&cont3)?;
        l3.text("Item 3 (Click me!)");
        menu.set_load_page_event(&cont3, &sub_page);

        menu.set_page(&main_page);

        self.menu = Some(menu);
        self._labels = Some([l0, l1, l2, l3]);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::CLICKED {
            if let Some(ref menu) = self.menu {
                if menu.back_button_is_root(&event.target()) {
                    let mbox = Msgbox::new(None::<&Obj<'_>>);
                    if let Ok(mbox) = mbox {
                        mbox.add_title("Hello");
                        mbox.add_text("Root back btn click.");
                        mbox.add_close_button();
                        core::mem::forget(mbox);
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

oxivgl_examples_common::example_main!(WidgetMenu2::default());
