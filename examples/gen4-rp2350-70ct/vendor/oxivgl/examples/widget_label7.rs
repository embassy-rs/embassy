#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 7 — Translation / i18n
//!
//! Uses LVGL's `lv_translation_*` API to switch between English, German,
//! and Spanish. Labels use translation tags and auto-update when the
//! language changes via the dropdown.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    translation::{self, StaticCStr as S},
    view::{register_event_on, View},
    widgets::{Obj, Align, Dropdown, Label, WidgetError},
};

// NULL-terminated static arrays — LVGL stores these pointers directly.
static LANGUAGES: [S; 4] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::from_cstr(c"es"), S::NULL];
static TAGS: [S; 5] = [
    S::from_cstr(c"hello"), S::from_cstr(c"welcome"),
    S::from_cstr(c"button"), S::from_cstr(c"goodbye"), S::NULL,
];
// Translations flattened: [en_hello, en_welcome, en_button, en_goodbye, de_..., es_...]
static TRANSLATIONS: [S; 12] = [
    S::from_cstr(c"Hello!"), S::from_cstr(c"Welcome to LVGL"),
    S::from_cstr(c"Press a button"), S::from_cstr(c"Goodbye"),
    S::from_cstr(c"Hallo!"), S::from_cstr(c"Willkommen bei LVGL"),
    S::from_cstr(c"Taste druecken"), S::from_cstr(c"Auf Wiedersehen"),
    S::from_cstr(c"Hola!"), S::from_cstr(c"Bienvenido a LVGL"),
    S::from_cstr(c"Pulsa un boton"), S::from_cstr(c"Adios"),
];

static LANG_CSTR: [&core::ffi::CStr; 3] = [c"en", c"de", c"es"];

#[derive(Default)]
struct WidgetLabel7 {
    dd: Option<Dropdown<'static>>,
    _lbl_hello: Option<Label<'static>>,
    _lbl_welcome: Option<Label<'static>>,
    _lbl_button: Option<Label<'static>>,
    _lbl_goodbye: Option<Label<'static>>,
}

impl View for WidgetLabel7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Register translation pack and set initial language
        translation::add_static(&LANGUAGES, &TAGS, &TRANSLATIONS);
        translation::set_language(c"en");

        // Language selector dropdown
        let dd = Dropdown::new(container)?;
        dd.set_options("English\nDeutsch\nEspanol");
        dd.align(Align::TopMid, 0, 10);
        dd.bubble_events();

        // Labels with translation tags — auto-update on language change
        let lbl_hello = Label::new(container)?;
        lbl_hello.set_translation_tag("hello");
        lbl_hello.align(Align::Center, 0, -40);

        let lbl_welcome = Label::new(container)?;
        lbl_welcome.set_translation_tag("welcome");
        lbl_welcome.align(Align::Center, 0, -10);

        let lbl_button = Label::new(container)?;
        lbl_button.set_translation_tag("button");
        lbl_button.align(Align::Center, 0, 20);

        let lbl_goodbye = Label::new(container)?;
        lbl_goodbye.set_translation_tag("goodbye");
        lbl_goodbye.align(Align::Center, 0, 50);

                self.dd = Some(dd);
        self._lbl_hello = Some(lbl_hello);
        self._lbl_welcome = Some(lbl_welcome);
        self._lbl_button = Some(lbl_button);
        self._lbl_goodbye = Some(lbl_goodbye);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref dd) = self.dd { register_event_on(self, dd.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::VALUE_CHANGED {
            if let Some(ref dd) = self.dd {
                let idx = dd.get_selected() as usize;
                if idx < LANG_CSTR.len() {
                    translation::set_language(LANG_CSTR[idx]);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel7::default());
