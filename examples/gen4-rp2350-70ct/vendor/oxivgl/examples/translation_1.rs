#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Translation 1 — Static and dynamic translation packs
//!
//! Registers one static translation pack (animals, three languages) and one
//! dynamic translation pack (furniture, two languages), then sets the active
//! language to German ("de") and displays two translated labels.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.

use oxivgl::{
    translation::{self, StaticCStr as S},
    view::{NavAction, View},
    widgets::{Obj, Align, Label, WidgetError},
};

// Static pack: languages and animal tags (NULL-terminated).
static LANGUAGES: [S; 4] = [
    S::from_cstr(c"en"),
    S::from_cstr(c"de"),
    S::from_cstr(c"es"),
    S::NULL,
];
static TAGS: [S; 5] = [
    S::from_cstr(c"tiger"),
    S::from_cstr(c"lion"),
    S::from_cstr(c"rabbit"),
    S::from_cstr(c"elephant"),
    S::NULL,
];
// Translations flattened row-major: [en_tiger, de_tiger, es_tiger, en_lion, ...]
static TRANSLATIONS: [S; 12] = [
    S::from_cstr(c"The Tiger"),
    S::from_cstr(c"Der Tiger"),
    S::from_cstr(c"El Tigre"),
    S::from_cstr(c"The Lion"),
    S::from_cstr(c"Der Loewe"),
    S::from_cstr(c"El Leon"),
    S::from_cstr(c"The Rabbit"),
    S::from_cstr(c"Das Kaninchen"),
    S::from_cstr(c"El Conejo"),
    S::from_cstr(c"The Elephant"),
    S::from_cstr(c"Der Elefant"),
    S::from_cstr(c"El Elefante"),
];

#[derive(Default)]
struct Translation1 {
    _lbl_tiger: Option<Label<'static>>,
    _lbl_chair: Option<Label<'static>>,
}

impl View for Translation1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Register the static pack (animals, three languages).
        translation::add_static(&LANGUAGES, &TAGS, &TRANSLATIONS);

        // Register the dynamic pack (furniture, two languages).
        let dyn_pack = translation::add_dynamic();
        dyn_pack.add_language(c"en");
        dyn_pack.add_language(c"de");

        let table = dyn_pack.add_tag(c"table");
        dyn_pack.set_translation(&table, 0, c"It's a table");
        dyn_pack.set_translation(&table, 1, c"Das ist ein Tisch");

        let chair = dyn_pack.add_tag(c"chair");
        dyn_pack.set_translation(&chair, 0, c"It's a chair");
        dyn_pack.set_translation(&chair, 1, c"Das ist ein Stuhl");

        // Activate German.
        translation::set_language(c"de");

        // Label showing the translated animal name.
        let lbl_tiger = Label::new(container)?;
        lbl_tiger
            .text(translation::translate(c"tiger").to_str().unwrap_or("tiger"))
            .align(Align::Center, 0, -25);

        // Label showing the translated furniture name from the dynamic pack.
        let lbl_chair = Label::new(container)?;
        lbl_chair
            .text(translation::translate(c"chair").to_str().unwrap_or("chair"))
            .align(Align::Center, 0, 25);

                self._lbl_tiger = Some(lbl_tiger);
        self._lbl_chair = Some(lbl_chair);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Translation1::default());
