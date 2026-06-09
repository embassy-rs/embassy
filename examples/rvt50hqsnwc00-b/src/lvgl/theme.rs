//! Shared LVGL styles for the hall lighting UI.

use lvgl::style::Style;
use lvgl::{Color, LvResult, Part, Widget};
use lvgl::widgets::Label;

pub struct Theme {
    pub card: Style,
    pub header: Style,
    pub btn: Style,
    pub btn_active: Style,
    pub text: Style,
    pub muted: Style,
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

        Self {
            card,
            header,
            btn,
            btn_active,
            text,
            muted,
        }
    }

    pub fn apply_screen(&self, screen: &mut lvgl::Obj) -> LvResult<()> {
        let mut bg = Style::default();
        bg.set_bg_color(Color::from_rgb((0x10, 0x18, 0x28)));
        screen.add_style(Part::Main, &mut bg)
    }

    pub fn label_text(&self, label: &mut Label) -> LvResult<()> {
        let mut style = self.text.clone();
        label.add_style(Part::Main, &mut style)
    }

    pub fn label_muted(&self, label: &mut Label) -> LvResult<()> {
        let mut style = self.muted.clone();
        label.add_style(Part::Main, &mut style)
    }
}
