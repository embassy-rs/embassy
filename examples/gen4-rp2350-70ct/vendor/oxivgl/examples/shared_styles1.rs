#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared Styles 1 — one style guide applied to many widgets
//!
//! The memory-efficient way to style at scale (see `docs/memory-tuning.md`):
//! build each visual treatment **once** as a shared [`Style`] and apply it to
//! every widget with `add_style`. LVGL then keeps a single property buffer per
//! style instead of a per-object local style.
//!
//! Note there is not a single inline style setter here — only shared styles
//! plus geometry/content helpers (`size`, `align`, `text`), which are not
//! styles. The three styles are dropped at the end of `create()`; the widgets
//! retain them via `Rc`.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Label, Obj, WidgetError},
};

#[derive(Default)]
struct SharedStyles1 {
    _cards: Option<heapless::Vec<Obj<'static>, 3>>,
    _labels: Option<heapless::Vec<Label<'static>, 6>>,
}

impl View for SharedStyles1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        // ── The app's style guide — built once, shared everywhere ──────────
        let screen_bg = Style::new(|s| {
            s.bg_color_hex(0x10101a).bg_opa(255);
        });
        let card = Style::new(|s| {
            s.bg_color_hex(0x222244)
                .bg_opa(255)
                .radius(8)
                .border_width(0)
                .pad_all(10);
        });
        let heading = Style::new(|s| {
            s.text_color_hex(0xffffff);
        });
        let muted = Style::new(|s| {
            s.text_color_hex(0x8888aa);
        });

        container.add_style(&screen_bg, Selector::DEFAULT);

        let titles = ["Voltage", "Current", "Temperature"];
        let values = ["13.8 V", "42.1 A", "57 °C"];

        let mut cards = heapless::Vec::<Obj<'static>, 3>::new();
        let mut labels = heapless::Vec::<Label<'static>, 6>::new();

        for i in 0..3usize {
            let cardw = Obj::new(container)?;
            cardw.add_style(&card, Selector::DEFAULT);
            cardw.size(220, 56);
            cardw.align(Align::TopMid, 0, 12 + i as i32 * 68);

            let title = Label::new(&cardw)?;
            title.add_style(&heading, Selector::DEFAULT);
            title.text(titles[i]);
            title.align(Align::TopLeft, 0, 0);

            let value = Label::new(&cardw)?;
            value.add_style(&muted, Selector::DEFAULT);
            value.text(values[i]);
            value.align(Align::BottomLeft, 0, 0);

            let _ = cards.push(cardw);
            let _ = labels.push(title);
            let _ = labels.push(value);
        }

        // screen_bg / card / heading / muted go out of scope here — the styled
        // widgets retain them via Rc, so the styling persists.
        self._cards = Some(cards);
        self._labels = Some(labels);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(SharedStyles1::default());
