#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 7 — Conditional style binding: light / dark theme toggle
//!
//! Two subjects drive the UI:
//! - `subject_theme`: 0 = light, 1 = dark
//! - `subject_room_temperature`: integer 20–40 °C displayed by a label and a slider
//!
//! When `subject_theme == 1` the dark-overlay styles are conditionally bound
//! to the screen, container, and all three slider parts, demonstrating
//! `lv_obj_bind_style`. A dropdown lets the user switch between Light and Dark.

use oxivgl::{
    layout::FlexFlow,
    style::{LV_SIZE_CONTENT, Palette, Selector, StyleBuilder, color_make, palette_main},
    view::{NavAction, View},
    widgets::{Align, Child, Dropdown, Label, Obj, Part, Slider, Subject, WidgetError},
};

#[derive(Default)]
struct Observer7 {
    // Widgets — dropped before subjects.
    _cont: Option<Obj<'static>>,
    _label: Option<Child<Label<'static>>>,
    _slider: Option<Child<Slider<'static>>>,
    _dropdown: Option<Dropdown<'static>>,

    // Light styles (added unconditionally).
    _style_screen: Option<oxivgl::style::Style>,
    _style_slider_main: Option<oxivgl::style::Style>,
    _style_slider_indicator: Option<oxivgl::style::Style>,
    _style_slider_knob: Option<oxivgl::style::Style>,

    // Dark overlay styles (bound conditionally when theme == 1).
    _style_screen_dark: Option<oxivgl::style::Style>,
    _style_bg_dark: Option<oxivgl::style::Style>,
    _style_yellow: Option<oxivgl::style::Style>,

    // Subjects — dropped last so observer linkage outlives widgets.
    _subject_theme: Option<Subject>,
    _subject_room_temperature: Option<Subject>,
}

impl View for Observer7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        // ── Subjects ────────────────────────────────────────────────────────
        let subject_theme = Subject::new_int(0);
        let subject_room_temperature = Subject::new_int(25);

        // ── Light styles ────────────────────────────────────────────────────
        // Screen background: #cccccc
        let mut b = StyleBuilder::new();
        b.bg_color(color_make(204, 204, 204)).bg_opa(255);
        let style_screen = b.build();

        // Slider main track: red palette
        let mut b = StyleBuilder::new();
        b.bg_color(palette_main(Palette::Red)).bg_opa(255);
        let style_slider_main = b.build();

        // Slider indicator (filled portion): red palette
        let mut b = StyleBuilder::new();
        b.bg_color(palette_main(Palette::Red)).bg_opa(255);
        let style_slider_indicator = b.build();

        // Slider knob: red palette fill, white outline 4 px
        let mut b = StyleBuilder::new();
        b.bg_color(palette_main(Palette::Red))
            .bg_opa(255)
            .outline_color(color_make(255, 255, 255))
            .outline_width(4);
        let style_slider_knob = b.build();

        // ── Dark overlay styles ──────────────────────────────────────────────
        // Screen dark: #444444 bg, #eeeeee text
        let mut b = StyleBuilder::new();
        b.bg_color(color_make(68, 68, 68))
            .bg_opa(255)
            .text_color(color_make(238, 238, 238));
        let style_screen_dark = b.build();

        // Container/dropdown dark: #222222 bg, #eeeeee text, 30% border opacity
        let mut b = StyleBuilder::new();
        b.bg_color(color_make(34, 34, 34))
            .bg_opa(255)
            .text_color(color_make(238, 238, 238))
            .border_opa(77); // ~30% of 255
        let style_bg_dark = b.build();

        // Slider dark: yellow palette fill, #222222 outline
        let mut b = StyleBuilder::new();
        b.bg_color(palette_main(Palette::Yellow))
            .bg_opa(255)
            .outline_color(color_make(34, 34, 34));
        let style_yellow = b.build();

        // ── Screen ──────────────────────────────────────────────────────────
        container.add_style(&style_screen, Selector::DEFAULT);
        container.bind_style(&style_screen_dark, Selector::DEFAULT, &subject_theme, 1);

        // ── Container ───────────────────────────────────────────────────────
        let cont = Obj::new(container)?;
        cont.bind_style(&style_bg_dark, Selector::DEFAULT, &subject_theme, 1)
            .set_flex_flow(FlexFlow::Column)
            .size(LV_SIZE_CONTENT, LV_SIZE_CONTENT)
            .align(Align::TopMid, 0, 20);

        // ── Label ────────────────────────────────────────────────────────────
        let label = Child::new(Label::new(&cont)?);
        label.bind_text(&subject_room_temperature, c"%d \u{00b0}C");

        // ── Slider ───────────────────────────────────────────────────────────
        let slider = Child::new(Slider::new(&cont)?);
        slider.set_range(20, 40);
        slider.bind_value(&subject_room_temperature);

        // Apply light styles to all three slider parts.
        slider.add_style(&style_slider_main, Part::Main);
        slider.add_style(&style_slider_indicator, Part::Indicator);
        slider.add_style(&style_slider_knob, Part::Knob);

        // Bind dark (yellow) overlay styles for all three parts when theme == 1.
        slider.bind_style(&style_yellow, Part::Main, &subject_theme, 1);
        slider.bind_style(&style_yellow, Part::Indicator, &subject_theme, 1);
        slider.bind_style(&style_yellow, Part::Knob, &subject_theme, 1);

        // ── Dropdown ─────────────────────────────────────────────────────────
        let dropdown = Dropdown::new(container)?;
        dropdown
            .set_options("Light\nDark")
            .align(Align::TopMid, 0, 120);
        dropdown.bind_value(&subject_theme);
        dropdown.bind_style(&style_bg_dark, Selector::DEFAULT, &subject_theme, 1);

        // Style the popup list as well.
        let list = dropdown.get_list();
        list.bind_style(&style_bg_dark, Selector::DEFAULT, &subject_theme, 1);

                self._cont = Some(cont);
        self._label = Some(label);
        self._slider = Some(slider);
        self._dropdown = Some(dropdown);
        self._style_screen = Some(style_screen);
        self._style_slider_main = Some(style_slider_main);
        self._style_slider_indicator = Some(style_slider_indicator);
        self._style_slider_knob = Some(style_slider_knob);
        self._style_screen_dark = Some(style_screen_dark);
        self._style_bg_dark = Some(style_bg_dark);
        self._style_yellow = Some(style_yellow);
        self._subject_theme = Some(subject_theme);
        self._subject_room_temperature = Some(subject_room_temperature);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Observer7::default());
