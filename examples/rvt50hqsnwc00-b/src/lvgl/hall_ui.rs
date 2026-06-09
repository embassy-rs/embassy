//! JSON-driven hall lighting UI on top of [lv_binding_rust](https://github.com/lvgl/lv_binding_rust)
//! (vendored master under `vendor/lv_binding_rust/`).
//!
//! The widget tree (one card per field plus a group card with three lux
//! buttons each) is built with the safe [`Btn::create`] / [`Label::create`]
//! / [`Obj::create`] constructors. After build we drop the widget handles
//! and keep raw [`lvgl_sys::lv_obj_t`] pointers, because LVGL owns the
//! widgets in its tree — the Rust handles only matter during construction.
//! [`HallUi::set_button_active`] swaps a button's style via `lvgl_sys`.

extern crate alloc;

use core::fmt::Write as _;
use core::ptr::NonNull;
use core::time::Duration;

use cstr_core::CString;
use heapless::{String, Vec};
use lvgl::style::Style;
use lvgl::widgets::{Btn, Label};
use lvgl::{Align, Event, NativeObject, Obj, Part, Widget};

use crate::lvgl::display::Rvt50Display;
use crate::lvgl::input::{self, Rvt50Touch};
use crate::lvgl::theme::{self, Theme};
use crate::touch_config::{
    self, ALL_PREFIX, CENTRAL_OFF_LABEL, FIELDS, FieldLayout, GROUP_BUTTON_BASE, GROUP_EYEBROW, HALL_NAME, LUX_SUFFIX,
    OFF_LABEL, SUMMARY_READY,
};

const MAX_BUTTONS: usize = 64;

pub type PressHandler = fn(u8);
pub type ReleaseHandler = fn();

/// Geometry of the row of three lux buttons inside a card.
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
    buttons: Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS>,
    btn_style: &'static mut Style,
    btn_active_style: &'static mut Style,
}

impl HallUi {
    /// Initialize LVGL drivers and build the hall UI from `touch_config`.
    pub fn build(framebuffer: *mut u16, on_press: PressHandler, on_release: ReleaseHandler) -> lvgl::LvResult<Self> {
        let display = Rvt50Display::register(framebuffer)?;
        let touch = Rvt50Touch::register(&display.inner)?;

        let theme = Theme::new();
        let mut buttons: Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS> = Vec::new();

        let mut screen = display.inner.get_scr_act()?;
        screen.add_style(Part::Main, theme::leak(&theme.screen_bg));

        build_header(&mut screen, &theme)?;
        build_summary(&mut screen, &theme)?;

        for field in FIELDS.iter() {
            build_field_card(&mut screen, &theme, field, on_press, on_release, &mut buttons)?;
        }
        build_group_card(&mut screen, &theme, on_press, on_release, &mut buttons)?;

        // Persist one inactive / one active style for `set_button_active` to
        // swap between via `lvgl_sys`. The per-button styles attached during
        // build are independent leaked clones owned by LVGL.
        Ok(Self {
            _display: display,
            _touch: touch,
            buttons,
            btn_style: theme::leak(&theme.btn),
            btn_active_style: theme::leak(&theme.btn_active),
        })
    }

    /// Forward a touch sample from the I2C driver into the LVGL pointer state.
    pub fn set_touch(&self, x: u16, y: u16, pressed: bool) {
        input::set_touch(x, y, pressed);
    }

    /// Mark a button as active/inactive based on CAN minp feedback.
    pub fn set_button_active(&mut self, index: usize, active: bool) {
        let Some(btn_ptr) = self.buttons.get(index).copied() else {
            return;
        };
        let style: *mut lvgl_sys::lv_style_t = if active {
            &mut *self.btn_active_style as *mut Style as *mut _
        } else {
            &mut *self.btn_style as *mut Style as *mut _
        };
        // SAFETY: `btn_ptr` is a non-null `lv_obj_t*` produced by
        // `lv_btn_create` during `build` and still owned by LVGL's widget
        // tree; `style` points at a leaked `Style` that lives for `'static`.
        // Both calls are the standard LVGL pattern for swapping a style at
        // runtime and are safe under LVGL's single-threaded model.
        unsafe {
            lvgl_sys::lv_obj_remove_style(
                btn_ptr.as_ptr(),
                core::ptr::null_mut(),
                lvgl_sys::LV_PART_MAIN as lvgl_sys::lv_style_selector_t,
            );
            lvgl_sys::lv_obj_add_style(
                btn_ptr.as_ptr(),
                style,
                lvgl_sys::LV_PART_MAIN as lvgl_sys::lv_style_selector_t,
            );
        }
    }

    /// Advance LVGL by `ms` and run the timer handler. Call once per frame
    /// (~5 ms in `lvgl_touch_can`).
    pub fn tick_and_run(&self, ms: u64) {
        lvgl::tick_inc(Duration::from_millis(ms));
        lvgl::task_handler();
    }
}

/// Set the text of a label from a Rust `&str`. `lv_label_set_text` copies the
/// string into a label-owned buffer, so the temporary [`CString`] is allowed
/// to drop after this call.
fn set_label_text<'a>(label: &mut Label<'a>, text: &str) -> lvgl::LvResult<()> {
    let cstring = CString::new(text).map_err(|_| lvgl::LvError::InvalidReference)?;
    label.set_text(cstring.as_c_str());
    Ok(())
}

fn build_header<'a>(screen: &'a mut impl NativeObject, theme: &Theme) -> lvgl::LvResult<()> {
    let mut header = Obj::create(screen)?;
    header.set_size(touch_config::DISPLAY_WIDTH as i16, 44);
    header.set_pos(0, 0);
    header.add_style(Part::Main, theme::leak(&theme.header));

    let mut label = Label::create(&mut header)?;
    label.add_style(Part::Main, theme::leak(&theme.text));
    set_label_text(&mut label, HALL_NAME)?;
    label.set_pos(12, 12);
    Ok(())
}

fn build_summary<'a>(screen: &'a mut impl NativeObject, theme: &Theme) -> lvgl::LvResult<()> {
    let mut label = Label::create(screen)?;
    label.add_style(Part::Main, theme::leak(&theme.muted));
    set_label_text(&mut label, SUMMARY_READY)?;
    label.set_align(Align::TopMid, 0, 50);
    Ok(())
}

fn build_field_card<'a>(
    screen: &'a mut impl NativeObject,
    theme: &Theme,
    field: &FieldLayout,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS>,
) -> lvgl::LvResult<()> {
    let mut card = Obj::create(screen)?;
    card.add_style(Part::Main, theme::leak(&theme.card));
    card.set_pos(field.x as i16, field.y as i16);
    card.set_size(field.w as i16, field.h as i16);

    let mut eyebrow = Label::create(&mut card)?;
    eyebrow.add_style(Part::Main, theme::leak(&theme.muted));
    set_label_text(&mut eyebrow, field.eyebrow)?;

    let mut title = Label::create(&mut card)?;
    title.add_style(Part::Main, theme::leak(&theme.text));
    set_label_text(&mut title, field.label)?;
    title.set_pos(0, 14);

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

fn build_group_card<'a>(
    screen: &'a mut impl NativeObject,
    theme: &Theme,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS>,
) -> lvgl::LvResult<()> {
    let margin = 8i16;
    let card_h = (touch_config::DISPLAY_HEIGHT / 10).max(56) as i16;
    let card_y = touch_config::DISPLAY_HEIGHT as i16 - card_h - margin;
    let card_w = touch_config::DISPLAY_WIDTH as i16 - 2 * margin;

    let mut card = Obj::create(screen)?;
    card.add_style(Part::Main, theme::leak(&theme.card));
    card.set_pos(margin, card_y);
    card.set_size(card_w, card_h);

    let mut eyebrow = Label::create(&mut card)?;
    eyebrow.add_style(Part::Main, theme::leak(&theme.muted));
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

#[derive(Clone, Copy)]
enum LuxLabel {
    Lux(u16),
    AllLux(u16),
    Off,
    CentralOff,
}

impl LuxLabel {
    fn render(self, out: &mut String<32>) {
        let _ = match self {
            LuxLabel::Lux(v) => write!(out, "{v} {LUX_SUFFIX}"),
            LuxLabel::AllLux(v) => write!(out, "{ALL_PREFIX} {v} {LUX_SUFFIX}"),
            LuxLabel::Off => write!(out, "{OFF_LABEL}"),
            LuxLabel::CentralOff => write!(out, "{CENTRAL_OFF_LABEL}"),
        };
    }
}

fn add_lux_buttons<'a>(
    card: &'a mut impl NativeObject,
    theme: &Theme,
    row: &ButtonRow,
    spec: &[(LuxLabel, u8); 3],
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS>,
) -> lvgl::LvResult<()> {
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

fn add_lux_button<'a>(
    parent: &'a mut impl NativeObject,
    theme: &Theme,
    label: LuxLabel,
    button_index: u8,
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    on_press: PressHandler,
    on_release: ReleaseHandler,
    buttons: &mut Vec<NonNull<lvgl_sys::lv_obj_t>, MAX_BUTTONS>,
) -> lvgl::LvResult<()> {
    let mut button = Btn::create(parent)?;
    let raw_btn = button.raw();
    button.set_pos(x, y);
    button.set_size(w, h);
    button.add_style(Part::Main, theme::leak(&theme.btn));

    let mut text_label = Label::create(&mut button)?;
    text_label.add_style(Part::Main, theme::leak(&theme.text));
    let mut text = String::<32>::new();
    label.render(&mut text);
    set_label_text(&mut text_label, &text)?;
    text_label.set_align(Align::Center, 0, 0);

    button.on_event(move |_btn, event| match event {
        Event::Pressed => on_press(button_index),
        Event::Released | Event::PressLost => on_release(),
        _ => {}
    })?;

    let _ = buttons.push(raw_btn);
    Ok(())
}
