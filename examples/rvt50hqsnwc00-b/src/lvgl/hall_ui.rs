//! JSON-driven hall lighting UI (widgets + CAN button callbacks).
//!
//! Built with [lv_binding_rust](https://github.com/lvgl/lv_binding_rust) on top of the
//! Riverdi RVT50 display stack in `crate::lvgl::{display, input}`.

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use core::ptr::NonNull;

use cstr_core::{CStr, CString};
use heapless::Vec;
use lvgl::widgets::{Btn, Label};
use lvgl::{Align, Event, LvError, LvResult, NativeObject, Obj, Part, Widget};
use lvgl_sys;

use crate::lvgl::display::Rvt50Display;
use crate::lvgl::input::{self, Rvt50Touch};
use crate::lvgl::theme::Theme;
use crate::touch_config::{
    self, ALL_PREFIX, CENTRAL_OFF_LABEL, FIELDS, GROUP_BUTTON_BASE, GROUP_EYEBROW, HALL_NAME,
    LUX_SUFFIX, OFF_LABEL, SUMMARY_READY,
};

const MAX_BUTTONS: usize = 64;

pub type PressHandler = fn(u8);
pub type ReleaseHandler = fn();

pub struct HallUi {
    _display: Rvt50Display,
    _touch: Rvt50Touch,
    theme: Theme,
    buttons: Vec<Btn, MAX_BUTTONS>,
}

impl HallUi {
    /// Init LVGL drivers and build the hall UI from `touch_config`.
    pub fn build(
        framebuffer: *mut u16,
        on_press: PressHandler,
        on_release: ReleaseHandler,
    ) -> LvResult<Self> {
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

    pub fn set_touch(&self, x: u16, y: u16, pressed: bool) {
        input::set_touch(x, y, pressed);
    }

    pub fn set_button_active(&mut self, index: usize, active: bool) {
        if let Some(btn) = self.buttons.get_mut(index) {
            let style = if active {
                &mut self.theme.btn_active
            } else {
                &mut self.theme.btn
            };
            btn.add_style(Part::Main, style).ok();
        }
    }
}

fn obj_create(parent: &mut impl NativeObject) -> LvResult<Obj> {
    unsafe {
        let ptr = lvgl_sys::lv_obj_create(parent.raw()?.as_mut());
        NonNull::new(ptr)
            .map(Obj::from_raw)
            .ok_or(LvError::InvalidReference)
    }
}

fn leaked_label(text: &str) -> LvResult<&'static CStr> {
    CString::new(text)
        .map(|s| Box::leak(Box::new(s)).as_c_str())
        .map_err(|_| LvError::InvalidReference)
}

fn build_header(screen: &mut Obj, theme: &Theme) -> LvResult<()> {
    let mut header = obj_create(screen)?;
    header.set_size(touch_config::DISPLAY_WIDTH as i16, 44)?;
    header.set_pos(0, 0)?;
    header.add_style(Part::Main, &mut theme.header.clone())?;

    let mut label = Label::create(&mut header)?;
    theme.label_text(&mut label)?;
    label.set_text(leaked_label(HALL_NAME)?)?;
    label.set_pos(12, 12)?;
    Ok(())
}

fn build_summary(screen: &mut Obj, theme: &Theme) -> LvResult<()> {
    let mut label = Label::create(screen)?;
    theme.label_muted(&mut label)?;
    label.set_text(leaked_label(SUMMARY_READY)?)?;
    label.set_align(Align::TopMid, 0, 50)?;
    Ok(())
}

fn build_field_card(
    screen: &mut Obj,
    theme: &Theme,
    field: &touch_config::FieldLayout,
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
    eyebrow.set_text(leaked_label(field.eyebrow)?)?;

    let mut title = Label::create(&mut card)?;
    theme.label_text(&mut title)?;
    title.set_text(leaked_label(field.label)?)?;
    title.set_pos(0, 14)?;

    let btn_h = if field.h > 80 { 36 } else { 32 };
    let btn_w = ((field.w.saturating_sub(16)) / 3) as i16;
    let btn_y = (field.h.saturating_sub(btn_h as u16 + 12)) as i16;

    add_lux_button(
        &mut card,
        theme,
        &format!("500 {LUX_SUFFIX}"),
        field.button_base,
        4,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    add_lux_button(
        &mut card,
        theme,
        &format!("300 {LUX_SUFFIX}"),
        field.button_base + 1,
        8 + btn_w,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    add_lux_button(
        &mut card,
        theme,
        OFF_LABEL,
        field.button_base + 2,
        12 + 2 * btn_w,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    Ok(())
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
    eyebrow.set_text(leaked_label(GROUP_EYEBROW)?)?;

    let btn_h = card_h - 28;
    let btn_w = (card_w - 24) / 3;
    let btn_y = card_h - btn_h - 4;

    add_lux_button(
        &mut card,
        theme,
        &format!("{ALL_PREFIX} 500 {LUX_SUFFIX}"),
        GROUP_BUTTON_BASE,
        4,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    add_lux_button(
        &mut card,
        theme,
        &format!("{ALL_PREFIX} 300 {LUX_SUFFIX}"),
        GROUP_BUTTON_BASE + 1,
        8 + btn_w,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    add_lux_button(
        &mut card,
        theme,
        CENTRAL_OFF_LABEL,
        GROUP_BUTTON_BASE + 2,
        12 + 2 * btn_w,
        btn_y,
        btn_w,
        btn_h,
        on_press,
        on_release,
        buttons,
    )?;
    Ok(())
}

fn add_lux_button(
    parent: &mut Obj,
    theme: &Theme,
    text: &str,
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

    let mut label = Label::create(&mut button)?;
    theme.label_text(&mut label)?;
    label.set_text(leaked_label(text)?)?;
    label.set_align(Align::Center, 0, 0)?;

    button.on_event(move |_btn, event| match event {
        Event::Pressed => on_press(button_index),
        Event::Released | Event::PressLost => on_release(),
        _ => {}
    })?;

    buttons.push(button).map_err(|_| LvError::LvOOMemory)?;
    Ok(())
}
