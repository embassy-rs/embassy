//! Shared LVGL style palette for the hall lighting UI.
//!
//! On master `add_style(part, style: &'a mut Style)` borrows the style for the
//! widget's whole lifetime. Since our widgets live in a `'static` `HallUi`,
//! the styles attached to them must be `&'static mut Style` and unique per
//! attachment site. We expose templates here and the UI code calls
//! [`leak`] to mint a fresh `&'static mut` clone for each widget that needs it.

extern crate alloc;

use alloc::boxed::Box;

use lvgl::Color;
use lvgl::style::Style;

pub struct Theme {
    pub card: Style,
    pub header: Style,
    pub btn: Style,
    pub btn_active: Style,
    pub text: Style,
    pub muted: Style,
    pub screen_bg: Style,
}

impl Theme {
    pub fn new() -> Self {
        let mut card = Style::default();
        card.set_bg_color(Color::from_rgb((0x1E, 0x2A, 0x3A)));
        card.set_border_color(Color::from_rgb((0x3A, 0x5A, 0x8A)));
        card.set_border_width(1);
        card.set_radius(8);

        let mut header = Style::default();
        header.set_bg_color(Color::from_rgb((0x1A, 0x4A, 0x7A)));
        header.set_border_width(0);
        header.set_radius(0);

        let mut btn = Style::default();
        btn.set_bg_color(Color::from_rgb((0x2A, 0x3A, 0x50)));
        btn.set_border_color(Color::from_rgb((0x3A, 0x5A, 0x8A)));
        btn.set_border_width(1);
        btn.set_radius(6);

        let mut btn_active = Style::default();
        btn_active.set_bg_color(Color::from_rgb((0x2E, 0x7D, 0x32)));
        btn_active.set_border_color(Color::from_rgb((0xFF, 0xFF, 0xFF)));
        btn_active.set_border_width(2);
        btn_active.set_radius(6);

        let mut text = Style::default();
        text.set_text_color(Color::from_rgb((0xE8, 0xEE, 0xF4)));

        let mut muted = Style::default();
        muted.set_text_color(Color::from_rgb((0x90, 0xA0, 0xB0)));

        let mut screen_bg = Style::default();
        screen_bg.set_bg_color(Color::from_rgb((0x10, 0x18, 0x28)));

        Self {
            card,
            header,
            btn,
            btn_active,
            text,
            muted,
            screen_bg,
        }
    }
}

/// Box-and-leak a clone so `Style` lives for `'static`. Each attachment site
/// needs its own copy because `add_style` takes `&'a mut Style` (a unique
/// borrow that lasts for the widget's lifetime).
pub fn leak(template: &Style) -> &'static mut Style {
    Box::leak(Box::new(template.clone()))
}
