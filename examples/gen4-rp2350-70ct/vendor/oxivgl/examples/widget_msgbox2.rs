#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 2 — Settings dialog
//!
//! A non-modal message box styled as a settings dialog. Contains a title
//! with minimize and close header buttons, two sliders (brightness, speed)
//! in the content area, and Apply/Cancel buttons in an indigo-styled footer.

use oxivgl::{
    layout::FlexFlow,
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Child, Label, Msgbox, Slider, WidgetError},
};

#[derive(Default)]
struct WidgetMsgbox2 {
    _mbox: Option<Msgbox<'static>>,
    _lbl_bright: Option<Child<Label<'static>>>,
    _slider_bright: Option<Child<Slider<'static>>>,
    _lbl_speed: Option<Child<Label<'static>>>,
    _slider_speed: Option<Child<Slider<'static>>>,
}

impl View for WidgetMsgbox2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Non-modal msgbox (parent = screen)
        let mbox = Msgbox::new(Some(container))?;
        mbox.size(300, 200);
        mbox.center();
        let mbox_style = Style::new(|s| {
            s.clip_corner(true);
        });
        mbox.add_style(&mbox_style, Selector::DEFAULT);

        // Title + header buttons
        mbox.add_title("Settings");
        mbox.add_header_button(&oxivgl::symbols::MINUS);
        mbox.add_close_button();

        // Content area — flex column with sliders
        let content = mbox.get_content();
        content.set_flex_flow(FlexFlow::Column);
        let content_style = Style::new(|s| {
            s.pad_all(10);
        });
        content.add_style(&content_style, Selector::DEFAULT);

        let lbl_bright = Label::new(&*content)?;
        lbl_bright.text("Brightness");
        let lbl_bright = Child::new(lbl_bright);

        let slider_bright = Slider::new(&*content)?;
        slider_bright.width(250);
        slider_bright.set_value(70);
        let slider_bright = Child::new(slider_bright);

        let lbl_speed = Label::new(&*content)?;
        lbl_speed.text("Speed");
        let lbl_speed = Child::new(lbl_speed);

        let slider_speed = Slider::new(&*content)?;
        slider_speed.width(250);
        slider_speed.set_value(40);
        let slider_speed = Child::new(slider_speed);

        // Footer buttons
        let btn_apply = mbox.add_footer_button("Apply");
        btn_apply.set_flex_grow(1);

        let btn_cancel = mbox.add_footer_button("Cancel");
        btn_cancel.set_flex_grow(1);

        // Style footer with indigo background
        let footer = mbox.get_footer().expect("footer exists after add_footer_button");
        let indigo = oxivgl::style::palette_main(oxivgl::style::Palette::Indigo);
        let footer_style = Style::new(|s| {
            s.bg_color(indigo).bg_opa(255);
        });
        footer.add_style(&footer_style, Selector::DEFAULT);

        self._mbox = Some(mbox);
        self._lbl_bright = Some(lbl_bright);
        self._slider_bright = Some(slider_bright);
        self._lbl_speed = Some(lbl_speed);
        self._slider_speed = Some(slider_speed);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox2::default());
