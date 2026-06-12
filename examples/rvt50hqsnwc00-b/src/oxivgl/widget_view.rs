//! OxivGL widget showcase view for the Riverdi RVT50 (LVGL v9.5).

extern crate alloc;

use alloc::vec::Vec;

use defmt::info;
use oxivgl::enums::{EventCode, ObjFlag};
use oxivgl::event::Event;
use crate::oxivgl::fonts::{MONTSERRAT_14, MONTSERRAT_16};
use oxivgl::style::{GradDir, Selector};
use oxivgl::view::{NavAction, View, register_event_on};
use oxivgl::widgets::{AsLvHandle, Button, Label, Obj, RADIUS_MAX, Screen, TextAlign, WidgetError};

fn on_demo_button_click(_event: &Event) {
    info!("oxivgl light scene direct button CLICKED");
}

const SCREEN_BG: u32 = 0xE7DCC8;
const SURFACE: u32 = 0xFFFDF8;
const CARD_BG: u32 = 0xFFFDF7;
const CARD_BG_HIGHLIGHT: u32 = 0xFFF7E7;
const BUTTON_BG: u32 = 0xFCF7EC;
const BUTTON_BG_ACTIVE: u32 = 0xEEE8DB;
const BUTTON_BG_PRESSED: u32 = 0xE8DDC8;
const BORDER: u32 = 0xE4D8C3;
const BORDER_ACTIVE: u32 = 0xC5BAA8;
const TEXT: u32 = 0x151515;
const MUTED: u32 = 0x665F54;
const ACCENT: u32 = 0xA37418;
const LOGO: u32 = 0x6F6A62;

/// Panel layout tuned for 800×480 — maximize touch targets.
const SHELL_X: i32 = 16;
const SHELL_Y: i32 = 12;
const SHELL_W: i32 = 768;
const SHELL_H: i32 = 456;

const CARD_X0: i32 = 10;
const CARD_COL_PITCH: i32 = 148;
const CARD_W: i32 = 140;
const CARD_Y: i32 = 56;
const CARD_H: i32 = 388;
const CARD_PAD_X: i32 = 10;
const CARD_LABEL_W: i32 = CARD_W - CARD_PAD_X * 2;

const BUTTON_W: i32 = 120;
const BUTTON_H: i32 = 96;
const BUTTON_Y0: i32 = 58;
const BUTTON_Y_STEP: i32 = 106;
const BUTTON_LABEL_W: i32 = BUTTON_W - 8;

struct ColumnSpec {
    eyebrow: &'static str,
    title: &'static str,
    buttons: [&'static str; 3],
    highlight: bool,
}

const COLUMNS: [ColumnSpec; 5] = [
    ColumnSpec {
        eyebrow: "HALLE",
        title: "Tribüne",
        buttons: ["500 Lux", "300 Lux", "Aus"],
        highlight: false,
    },
    ColumnSpec {
        eyebrow: "FELD",
        title: "Links",
        buttons: ["500 Lux", "300 Lux", "Aus"],
        highlight: false,
    },
    ColumnSpec {
        eyebrow: "FELD",
        title: "Mitte",
        buttons: ["500 Lux", "300 Lux", "Aus"],
        highlight: false,
    },
    ColumnSpec {
        eyebrow: "FELD",
        title: "Rechts",
        buttons: ["500 Lux", "300 Lux", "Aus"],
        highlight: false,
    },
    ColumnSpec {
        eyebrow: "ZENTRAL",
        title: "Alle Felder",
        buttons: ["Alle\n500 Lux", "Alle\n300 Lux", "Zentral\nAus"],
        highlight: true,
    },
];

/// Stadium lighting scene demo styled after the protronic control mock-up.
#[derive(Default)]
pub struct WidgetView {
    labels: Vec<Label<'static>>,
    buttons: Vec<Button<'static>>,
    objects: Vec<Obj<'static>>,
    clicks: u32,
}

impl View for WidgetView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        self.labels.clear();
        self.buttons.clear();
        self.objects.clear();

        container
            .bg_color(SCREEN_BG)
            .bg_opa(255)
            .style_bg_grad_dir(GradDir::None, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);

        let shell = Obj::new(container)?;
        shell
            .size(SHELL_W, SHELL_H)
            .pos(SHELL_X, SHELL_Y)
            .bg_color(SURFACE)
            .bg_opa(255)
            .style_bg_grad_dir(GradDir::None, Selector::DEFAULT)
            .border_width(0)
            .radius(18, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);

        self.labels.push(make_label(
            &shell,
            "LICHTSZENENMODUL",
            10,
            8,
            250,
            ACCENT,
            LabelKind::Eyebrow,
        )?);

        let badge = Obj::new(&shell)?;
        badge
            .size(110, 28)
            .pos(SHELL_W / 2 - 55, 16)
            .bg_color(SURFACE)
            .bg_opa(255)
            .border_width(1)
            .radius(RADIUS_MAX, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);
        set_border_color(&badge, BORDER, 255);
        self.labels
            .push(make_label(&badge, "Demo Halle", 0, 6, 110, MUTED, LabelKind::Body)?);
        self.objects.push(badge);

        self.labels
            .push(make_label(&shell, "protronic", SHELL_W - 128, 16, 105, LOGO, LabelKind::Logo)?);
        let logo_dot = Obj::new(&shell)?;
        logo_dot
            .size(9, 9)
            .pos(SHELL_W - 24, 12)
            .bg_color(LOGO)
            .bg_opa(255)
            .border_width(0)
            .radius(RADIUS_MAX, Selector::DEFAULT)
            .remove_scrollable();
        self.objects.push(logo_dot);

        for (idx, column) in COLUMNS.iter().enumerate() {
            let x = CARD_X0 + idx as i32 * CARD_COL_PITCH;
            self.create_column(&shell, column, x)?;
        }

        self.objects.push(shell);
        container.update_layout();
        Ok(())
    }

    fn register_events(&mut self) {
        // Screen-level handler catches bubbled events from children.
        if let Some(screen) = Screen::active() {
            register_event_on(self, screen.handle());
        }

        for idx in 0..self.buttons.len() {
            register_event_on(self, self.buttons[idx].handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        match event.code() {
            EventCode::CLICKED | EventCode::SHORT_CLICKED | EventCode::SINGLE_CLICKED => {
                self.clicks += 1;
            }
            _ => {}
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

impl WidgetView {
    fn create_column(
        &mut self,
        parent: &impl oxivgl::widgets::AsLvHandle,
        column: &ColumnSpec,
        x: i32,
    ) -> Result<(), WidgetError> {
        let card = Obj::new(parent)?;
        card.size(CARD_W, CARD_H)
            .pos(x, CARD_Y)
            .bg_color(if column.highlight { CARD_BG_HIGHLIGHT } else { CARD_BG })
            .bg_opa(255)
            .style_bg_grad_dir(GradDir::None, Selector::DEFAULT)
            .border_width(1)
            .radius(14, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);
        set_border_color(&card, BORDER, 255);

        self.labels.push(make_label(
            &card,
            column.eyebrow,
            CARD_PAD_X,
            12,
            CARD_LABEL_W,
            ACCENT,
            LabelKind::Eyebrow,
        )?);
        self.labels.push(make_label(
            &card,
            column.title,
            CARD_PAD_X,
            28,
            CARD_LABEL_W,
            TEXT,
            LabelKind::Title,
        )?);

        for (idx, text) in column.buttons.iter().enumerate() {
            let active = column.highlight && idx == 2;
            let button = make_scene_button(
                &card,
                text,
                CARD_PAD_X,
                BUTTON_Y0 + idx as i32 * BUTTON_Y_STEP,
                active,
                &mut self.labels,
            )?;
            if self.buttons.is_empty() {
                button.on(EventCode::CLICKED, on_demo_button_click);
            }
            self.buttons.push(button);
        }

        self.objects.push(card);
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum LabelKind {
    Eyebrow,
    Title,
    Body,
    Logo,
}

fn make_label(
    parent: &impl oxivgl::widgets::AsLvHandle,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    color: u32,
    kind: LabelKind,
) -> Result<Label<'static>, WidgetError> {
    let label = Label::new(parent)?;
    label.remove_style_all();
    label
        .text(text)
        .pos(x, y)
        .width(w)
        .text_color(color)
        .remove_scrollable();
    set_text_opa(&label, 255);

    match kind {
        LabelKind::Eyebrow => {
            label
                .text_font(MONTSERRAT_14)
                .style_text_letter_space(3, Selector::DEFAULT)
                .text_align(TextAlign::Left);
        }
        LabelKind::Title => {
            label.text_font(MONTSERRAT_16).text_align(TextAlign::Left);
        }
        LabelKind::Body => {
            label.text_font(MONTSERRAT_14).text_align(TextAlign::Center);
        }
        LabelKind::Logo => {
            label
                .text_font(MONTSERRAT_16)
                .style_text_letter_space(1, Selector::DEFAULT)
                .text_align(TextAlign::Right);
        }
    }

    Ok(label)
}

fn make_scene_button(
    parent: &impl oxivgl::widgets::AsLvHandle,
    text: &str,
    x: i32,
    y: i32,
    active: bool,
    labels: &mut Vec<Label<'static>>,
) -> Result<Button<'static>, WidgetError> {
    let button = Button::new(parent)?;
    button
        .remove_style_all()
        .size(BUTTON_W, BUTTON_H)
        .pos(x, y)
        .bg_color(if active { BUTTON_BG_ACTIVE } else { BUTTON_BG })
        .bg_opa(255)
        .style_bg_grad_dir(GradDir::None, Selector::DEFAULT)
        .border_width(1)
        .radius(10, Selector::DEFAULT)
        .style_bg_color(
            unsafe { oxivgl_sys::lv_color_hex(BUTTON_BG_PRESSED) },
            oxivgl::enums::ObjState::PRESSED,
        )
        .remove_scrollable()
        .add_flag(ObjFlag::CLICKABLE)
        .bubble_events()
        .pad(0);
    set_border_color(&button, if active { BORDER_ACTIVE } else { BORDER }, 255);

    let label = Label::new(&button)?;
    label.remove_style_all();
    label
        .text(text)
        .width(BUTTON_LABEL_W)
        .text_color(TEXT)
        .text_font(MONTSERRAT_16)
        .text_align(TextAlign::Center)
        .center()
        .remove_scrollable();
    set_text_opa(&label, 255);
    labels.push(label);

    Ok(button)
}

fn set_text_opa(obj: &impl AsLvHandle, opa: u8) {
    unsafe {
        oxivgl_sys::lv_obj_set_style_text_opa(obj.lv_handle(), opa as oxivgl_sys::lv_opa_t, 0);
    }
}

fn set_border_color(obj: &impl AsLvHandle, color: u32, opa: u8) {
    unsafe {
        oxivgl_sys::lv_obj_set_style_border_color(obj.lv_handle(), oxivgl_sys::lv_color_hex(color), 0);
        oxivgl_sys::lv_obj_set_style_border_opa(obj.lv_handle(), opa as oxivgl_sys::lv_opa_t, 0);
    }
}
