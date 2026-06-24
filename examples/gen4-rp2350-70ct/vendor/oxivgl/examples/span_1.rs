#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Span 1 — Mixed-style text with colored spans
//!
//! Spangroup with multiple spans in different colors, fonts and decorations.
//! First-line indent of 20 px, clip overflow, left-aligned.

use oxivgl::{
    style::{palette_main, LV_SIZE_CONTENT, Palette, Selector, StyleBuilder, TextDecor},
    view::{NavAction, View},
    widgets::{Obj, Align, Spangroup, SpanOverflow, WidgetError},
};

#[derive(Default)]
struct Span1 {
    _spans: Option<Spangroup<'static>>,
}

impl View for Span1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut sb = StyleBuilder::new();
        sb.border_width(1)
            .border_color(palette_main(Palette::Orange))
            .pad_all(2);
        let style = sb.build();

        let spans = Spangroup::new(container)?;
        spans.width(300);
        spans.height(LV_SIZE_CONTENT);
        spans.align(Align::Center, 0, 0);
        spans.add_style(&style, Selector::DEFAULT);
        spans.set_overflow(SpanOverflow::Clip);
        spans.set_indent(20);

        let span = spans.add_span()?;
        span.set_text(c"China is a beautiful country.");
        span.set_text_color(palette_main(Palette::Red));
        span.set_text_opa(128);
        span.set_text_decor(TextDecor::UNDERLINE);

        let span = spans.add_span()?;
        span.set_text(c"good good study, day day up.");
        span.set_text_color(palette_main(Palette::Green));

        let span = spans.add_span()?;
        span.set_text(c"LVGL is an open-source graphics library.");
        span.set_text_color(palette_main(Palette::Blue));

        let span = spans.add_span()?;
        span.set_text(c"the boy no name.");
        span.set_text_color(palette_main(Palette::Green));
        span.set_text_decor(TextDecor::UNDERLINE);

        let span = spans.add_span()?;
        span.set_text(c"I have a dream that hope to come true.");
        span.set_text_decor(TextDecor::STRIKETHROUGH);

        spans.refresh();

                self._spans = Some(spans);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Span1::default());
