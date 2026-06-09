//! JSON-driven hall lighting UI on top of [lv_binding_rust](https://github.com/lvgl/lv_binding_rust).
//!
//! Builds a screen with one card per field plus a group card; each card has
//! three lux buttons (`500`, `300`, `OFF`/central-off) wired to the press/release
//! callbacks supplied by the binary. The widget hierarchy is driven by the
//! generated [`crate::touch_config`] tables so the UI matches the project
//! JSON without any hand-maintained constants.

extern crate alloc;

use core::ptr::NonNull;
use core::time::Duration;

use cstr_core::CString;
use heapless::Vec;
use lvgl::widgets::{Btn, Label};
use lvgl::{Align, Event, LvError, LvResult, NativeObject, Obj, Part, Widget};

use crate::lvgl::display::Rvt50Display;
use crate::lvgl::input::{self, Rvt50Touch};
use crate::lvgl::theme::Theme;
use crate::touch_config::{
    self, ALL_PREFIX, CENTRAL_OFF_LABEL, FIELDS, FieldLayout, GROUP_BUTTON_BASE, GROUP_EYEBROW, HALL_NAME, LUX_SUFFIX,
    OFF_LABEL, SUMMARY_READY,
};

/// Maximum number of buttons the UI can hold; covers any project we generate.
const MAX_BUTTONS: usize = 64;

pub type PressHandler = fn(u8);
pub type ReleaseHandler = fn();

/// Geometry of the row of three lux buttons inside a card.
///
/// Layout is `4 px` left margin + `4 px` gap between buttons, matching the
/// values previously hard-coded in `build_field_card` / `build_group_card`.
struct ButtonRow {
    btn_w: i16,
    btn_h: i16,
    btn_y: i16,
}

impl ButtonRow {
    /// Layout for a per-field card (`btn_h` switches at the 80 px height
    /// breakpoint, matching the original UI).
    fn for_field(card_w: u16, card_h: u16) -> Self {
        let btn_h = if card_h > 80 { 36 } else { 32 };
        let btn_w = ((card_w.saturating_sub(16)) / 3) as i16;
        let btn_y = card_h.saturating_sub(btn_h as u16 + 12) as i16;
        Self { btn_w, btn_h, btn_y }
    }

    /// Layout for the group card across the bottom of the screen.
    fn for_group(card_w: i16, card_h: i16) -> Self {
        let btn_h = card_h - 28;
        let btn_w = (card_w - 24) / 3;
        let btn_y = card_h - btn_h - 4;
        Self { btn_w, btn_h, btn_y }
    }

    fn col_x(&self, col: i16) -> i16 {
        4 + col * (self.btn_w + 4)
    }
}

/// JSON-driven hall lighting UI.
pub struct HallUi {
    _display: Rvt50Display,
    _touch: Rvt50Touch,
    theme: Theme,
    buttons: Vec<Btn, MAX_BUTTONS>,
}

impl HallUi {
    /// Initialize LVGL drivers and build the hall UI from `touch_config`.
    pub fn build(framebuffer: *mut u16, on_press: PressHandler, on_release: ReleaseHandler) -> LvResult<Self> {
        let display = Rvt50Display::register(framebuffer)?;
        let touch = Rvt50Touch::register(&display.inner)?;

        let theme = Theme::new();
        let mut buttons = Vec::new();
        let mut screen = display.inner.get_scr_act()?;
        theme.apply_screen(&mut screen)?;

        build_header(&mut screen, &theme)?;
        build_summary(&mut screen, &theme)?;

        for field in FIELDS.iter() {
            build_field_card(&mut screen, &theme, field, on_press, on_release, &mut buttons)?;
        }
        build_group_card(&mut screen, &theme, on_press, on_release, &mut buttons)?;

        Ok(Self {
            _display: display,
            _touch: touch,
            theme,
            buttons,
        })
    }

    /// Forward a touch sample from the I2C driver into the LVGL pointer state.
    pub fn set_touch(&self, x: u16, y: u16, pressed: bool) {
        input::set_touch(x, y, pressed);
    }

    /// Mark a button as active/inactive based on CAN minp feedback.
    pub fn set_button_active(&mut self, index: usize, active: bool) {
        if let Some(btn) = self.buttons.get_mut(index) {
            let style = if active {
                &mut self.theme.btn_active
            } else {
                &mut self.theme.btn
            };
            let _ = btn.add_style(Part::Main, style);
        }
    }

    /// Advance the LVGL tick by `ms` milliseconds and run the timer handler.
    /// Call once per UI iteration (~5 ms in `lvgl_touch_can`).
    pub fn tick_and_run(&self, ms: u64) {
        lvgl::tick_inc(Duration::from_millis(ms));
        lvgl::task_handler();
    }
}

/// Allocate a generic [`Obj`] as a child of `parent`.
///
/// `lv_binding_rust` 0.6 does not yet expose a safe `Obj::create(parent)`
/// constructor (only `Obj::default()` which creates without a parent), so this
/// is the single place we drop into `lvgl_sys` from the UI builder.
fn obj_create(parent: &mut impl NativeObject) -> LvResult<Obj> {
    // SAFETY: `parent.raw()?` returns a non-null pointer to an initialised
    // `lv_obj_t`. `lv_obj_create` only reads it to attach a fresh child.
    unsafe {
        let ptr = lvgl_sys::lv_obj_create(parent.raw()?.as_mut());
        NonNull::new(ptr).map(Obj::from_raw).ok_or(LvError::InvalidReference)
    }
}

/// Set the text of a label from a Rust `&str`.
///
/// `lv_label_set_text` copies the string into a label-owned buffer, so the
/// temporary [`CString`] is allowed to drop after this call.
fn set_label_text(label: &mut Label, text: &str) -> LvResult<()> {
    let cstring = CString::new(text).map_err(|_| LvError::InvalidReference)?;
    label.set_text(cstring.as_c_str())
}

fn build_header(screen: &mut Obj, theme: &Theme) -> LvResult<()> {
    let mut header = obj_create(screen)?;
    header.set_size(touch_config::DISPLAY_WIDTH as i16, 44)?;
    header.set_pos(0, 0)?;
    header.add_style(Part::Main, &mut theme.header.clone())?;

    let mut label = Label::create(&mut header)?;
    theme.label_text(&mut label)?;
    set_label_text(&mut label, HALL_NAME)?;
    label.set_pos(12, 12)?;
    Ok(())
}

fn build_summary(screen: &mut Obj, theme: &Theme) -> LvResult<()> {
    let mut label = Label::create(screen)?;
    theme.label_muted(&mut label)?;
    set_label_text(&mut label, SUMMARY_READY)?;
    label.set_align(Align::TopMid, 0, 50)?;
    Ok(())
}

fn build_field_card(
    screen: &mut Obj,
    theme: &Theme,
    field: &FieldLayout,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<Btn, MAX_BUTTONS>,
) -> LvResult<()> {
    let mut card = obj_create(screen)?;
    card.add_style(Part::Main, &mut theme.card.clone())?;
    card.set_pos(field.x as i16, field.y as i16)?;
    card.set_size(field.w as i16, field.h as i16)?;

    let mut eyebrow = Label::create(&mut card)?;
    theme.label_muted(&mut eyebrow)?;
    set_label_text(&mut eyebrow, field.eyebrow)?;

    let mut title = Label::create(&mut card)?;
    theme.label_text(&mut title)?;
    set_label_text(&mut title, field.label)?;
    title.set_pos(0, 14)?;

    let row = ButtonRow::for_field(field.w, field.h);
    add_lux_buttons(
        &mut card,
        theme,
        &row,
        &[
            (LuxLabel::Lux(500), field.button_base),
            (LuxLabel::Lux(300), field.button_base + 1),
            (LuxLabel::Off, field.button_base + 2),
        ],
        on_press,
        on_release,
        buttons,
    )
}

fn build_group_card(
    screen: &mut Obj,
    theme: &Theme,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<Btn, MAX_BUTTONS>,
) -> LvResult<()> {
    let margin = 8i16;
    let card_h = (touch_config::DISPLAY_HEIGHT / 10).max(56) as i16;
    let card_y = touch_config::DISPLAY_HEIGHT as i16 - card_h - margin;
    let card_w = touch_config::DISPLAY_WIDTH as i16 - 2 * margin;

    let mut card = obj_create(screen)?;
    card.add_style(Part::Main, &mut theme.card.clone())?;
    card.set_pos(margin, card_y)?;
    card.set_size(card_w, card_h)?;

    let mut eyebrow = Label::create(&mut card)?;
    theme.label_muted(&mut eyebrow)?;
    set_label_text(&mut eyebrow, GROUP_EYEBROW)?;

    let row = ButtonRow::for_group(card_w, card_h);
    add_lux_buttons(
        &mut card,
        theme,
        &row,
        &[
            (LuxLabel::AllLux(500), GROUP_BUTTON_BASE),
            (LuxLabel::AllLux(300), GROUP_BUTTON_BASE + 1),
            (LuxLabel::CentralOff, GROUP_BUTTON_BASE + 2),
        ],
        on_press,
        on_release,
        buttons,
    )
}

/// Tag describing one of the four label shapes used by the hall UI buttons.
#[derive(Clone, Copy)]
enum LuxLabel {
    Lux(u16),
    AllLux(u16),
    Off,
    CentralOff,
}

impl LuxLabel {
    fn render(self, out: &mut heapless::String<32>) {
        use core::fmt::Write as _;
        let _ = match self {
            LuxLabel::Lux(v) => write!(out, "{v} {LUX_SUFFIX}"),
            LuxLabel::AllLux(v) => write!(out, "{ALL_PREFIX} {v} {LUX_SUFFIX}"),
            LuxLabel::Off => write!(out, "{OFF_LABEL}"),
            LuxLabel::CentralOff => write!(out, "{CENTRAL_OFF_LABEL}"),
        };
    }
}

fn add_lux_buttons(
    card: &mut Obj,
    theme: &Theme,
    row: &ButtonRow,
    spec: &[(LuxLabel, u8); 3],
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<Btn, MAX_BUTTONS>,
) -> LvResult<()> {
    for (col, (label, button_index)) in spec.iter().enumerate() {
        add_lux_button(
            card,
            theme,
            *label,
            *button_index,
            row.col_x(col as i16),
            row.btn_y,
            row.btn_w,
            row.btn_h,
            on_press,
            on_release,
            buttons,
        )?;
    }
    Ok(())
}

fn add_lux_button(
    parent: &mut Obj,
    theme: &Theme,
    label: LuxLabel,
    button_index: u8,
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<Btn, MAX_BUTTONS>,
) -> LvResult<()> {
    let mut button = Btn::create(parent)?;
    button.set_pos(x, y)?;
    button.set_size(w, h)?;
    button.add_style(Part::Main, &mut theme.btn.clone())?;

    let mut text_label = Label::create(&mut button)?;
    theme.label_text(&mut text_label)?;
    let mut text = heapless::String::<32>::new();
    label.render(&mut text);
    set_label_text(&mut text_label, &text)?;
    text_label.set_align(Align::Center, 0, 0)?;

    button.on_event(move |_btn, event| match event {
        Event::Pressed => on_press(button_index),
        Event::Released | Event::PressLost => on_release(),
        _ => {}
    })?;

    buttons.push(button).map_err(|_| LvError::LvOOMemory)?;
    Ok(())
}
