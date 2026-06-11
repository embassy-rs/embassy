//! Host copy of the protronic lighting-scene [`WidgetView`].
//!
//! Keep in sync with `examples/rvt50hqsnwc00-b/src/oxivgl/widget_view.rs`.

extern crate alloc;

use alloc::vec::Vec;

use log::info;
use oxivgl::draw::Area;
use oxivgl::enums::{EventCode, ObjFlag};
use oxivgl::event::Event;
use oxivgl::fonts::{MONTSERRAT_14, MONTSERRAT_16};
use oxivgl::style::Selector;
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
        eyebrow: "SAMMELBEFEHL",
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

        container.bg_color(SCREEN_BG).bg_opa(255).remove_scrollable().pad(0);

        let shell = Obj::new(container)?;
        shell
            .size(720, 430)
            .pos(40, 25)
            .bg_color(SURFACE)
            .bg_opa(255)
            .border_width(0)
            .radius(18, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);

        self.labels.push(make_label(
            &shell,
            "LICHTSZENENMODUL",
            12,
            12,
            250,
            ACCENT,
            LabelKind::Eyebrow,
        )?);

        let badge = Obj::new(&shell)?;
        badge
            .size(110, 28)
            .pos(305, 21)
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
            .push(make_label(&shell, "protronic", 585, 22, 105, LOGO, LabelKind::Logo)?);
        let logo_dot = Obj::new(&shell)?;
        logo_dot
            .size(9, 9)
            .pos(688, 18)
            .bg_color(LOGO)
            .bg_opa(255)
            .border_width(0)
            .radius(RADIUS_MAX, Selector::DEFAULT)
            .remove_scrollable();
        self.objects.push(logo_dot);

        for (idx, column) in COLUMNS.iter().enumerate() {
            let x = 12 + idx as i32 * 142;
            self.create_column(&shell, column, x)?;
        }

        self.objects.push(shell);
        container.update_layout();
        Ok(())
    }

    fn register_events(&mut self) {
        if let Some(screen) = Screen::active() {
            register_event_on(self, screen.handle());
        }

        for idx in 0..self.buttons.len() {
            register_event_on(self, self.buttons[idx].handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let code = event.code();
        let target = event.target_handle() as usize;
        let btn_idx = self.button_index_for_handle(event.target_handle());

        match code {
            EventCode::PRESSED | EventCode::PRESSING | EventCode::CLICKED
            | EventCode::SHORT_CLICKED | EventCode::SINGLE_CLICKED | EventCode::LONG_PRESSED
            | EventCode::LONG_PRESSED_REPEAT => {
                info!(
                    "oxivgl widget event code={:?} target={target:#x} btn={btn_idx:?} clicks={}",
                    code.0, self.clicks
                );
            }
            _ => {}
        }

        match code {
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
        card.size(130, 344)
            .pos(x, 82)
            .bg_color(if column.highlight { CARD_BG_HIGHLIGHT } else { CARD_BG })
            .bg_opa(255)
            .border_width(1)
            .radius(14, Selector::DEFAULT)
            .remove_scrollable()
            .pad(0);
        set_border_color(&card, BORDER, 255);

        self.labels.push(make_label(
            &card,
            column.eyebrow,
            14,
            15,
            102,
            ACCENT,
            LabelKind::Eyebrow,
        )?);
        self.labels
            .push(make_label(&card, column.title, 14, 33, 102, TEXT, LabelKind::Title)?);

        for (idx, text) in column.buttons.iter().enumerate() {
            let active = column.highlight && idx == 2;
            let button = make_scene_button(&card, text, 14, 70 + idx as i32 * 92, active, &mut self.labels)?;
            if self.buttons.is_empty() {
                button.on(EventCode::CLICKED, on_demo_button_click);
            }
            self.buttons.push(button);
        }

        self.objects.push(card);
        Ok(())
    }

    pub fn log_layout(&self) {
        info!("oxivgl scene buttons count={}", self.buttons.len());
        if let Some(btn) = self.buttons.first() {
            let area = btn.get_coords();
            info!(
                "oxivgl first scene btn area x1={} y1={} x2={} y2={} handle={:#x}",
                area.x1,
                area.y1,
                area.x2,
                area.y2,
                btn.handle() as usize
            );
        }
        if let Some(btn) = self.buttons.last() {
            let area = btn.get_coords();
            info!(
                "oxivgl last scene btn area x1={} y1={} x2={} y2={} handle={:#x}",
                area.x1,
                area.y1,
                area.x2,
                area.y2,
                btn.handle() as usize
            );
        }
    }

    #[allow(dead_code)]
    pub fn find_button_at(&self, x: i32, y: i32) -> Option<(usize, Area)> {
        for (idx, btn) in self.buttons.iter().enumerate() {
            let area = btn.get_coords();
            if x >= area.x1 && x <= area.x2 && y >= area.y1 && y <= area.y2 {
                return Some((idx, area));
            }
        }
        None
    }

    fn button_index_for_handle(&self, handle: *mut oxivgl_sys::lv_obj_t) -> Option<usize> {
        self.buttons.iter().position(|btn| btn.handle() == handle)
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
    label
        .text(text)
        .pos(x, y)
        .width(w)
        .text_color(color)
        .remove_scrollable();

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
        .size(102, 78)
        .pos(x, y)
        .bg_color(if active { BUTTON_BG_ACTIVE } else { BUTTON_BG })
        .bg_opa(255)
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
    label
        .text(text)
        .width(92)
        .text_color(TEXT)
        .text_font(MONTSERRAT_16)
        .text_align(TextAlign::Center)
        .center()
        .remove_scrollable();
    labels.push(label);

    Ok(button)
}

fn set_border_color(obj: &impl AsLvHandle, color: u32, opa: u8) {
    unsafe {
        oxivgl_sys::lv_obj_set_style_border_color(obj.lv_handle(), oxivgl_sys::lv_color_hex(color), 0);
        oxivgl_sys::lv_obj_set_style_border_opa(obj.lv_handle(), opa as oxivgl_sys::lv_opa_t, 0);
    }
}
