#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Touch Setpoint — a touch-navigated value editor with hold-to-repeat
//!
//! A numeric setpoint that mixes the two input gestures a small instrument UI
//! needs, both fed in oxivgl's own vocabulary (LVGL keys and raw `(x, y)`
//! coordinates — never a board/driver/MCU type):
//!
//! - **Hold-to-repeat (keypad)** — hold `−` / `+` and the value keeps changing.
//!   The buttons report a *held* [`Key`] into a [`KeypadState`]; the
//!   [`KeypadIndev`] is configured with
//!   [`with_repeat`](oxivgl::indev::KeypadIndev::with_repeat), so LVGL re-sends
//!   the key to the focused [`Spinbox`] first after 400 ms, then every 80 ms.
//!   This is the gesture a 3-button board needs for setpoint editing.
//! - **Touch (pointer)** — tap a preset (`Min` / `Half` / `Max`) to jump the
//!   value. Presets are driven by direct touch: a tap at the widget's
//!   coordinate. The example registers a [`PointerIndev`] — the touchscreen
//!   device — exactly as an ESP32 binary would.
//!
//! # Feeding the pointer on real hardware
//!
//! oxivgl stays BSP- and MCU-agnostic: the POINTER device takes plain `(x, y)`
//! coordinates, so the consumer's binary writes a tiny bridge from its touch
//! driver. On an M5Stack CoreS3 that is one line:
//!
//! ```ignore
//! // binary-side: oxivgl knows neither the board nor the panel.
//! let _touch = PointerIndev::new_with(|| ft6336u::read_touch());
//! //                                      ^ Option<(u16, u16)> — Some = pressed
//! ```
//!
//! On the host there is no touch panel, so the closure here returns `None`; the
//! SDL window's mouse is itself a POINTER device, so every tap you make with it
//! drives LVGL through the exact same path the FT6336U would on hardware.

use oxivgl::{
    enums::{EventCode, Key},
    event::Event,
    group::Group,
    indev::{KeypadIndev, KeypadState, PointerIndev},
    layout::{FlexAlign, FlexFlow},
    style::{Palette, Selector, StyleBuilder, color_make, palette_main},
    symbols,
    view::{NavAction, View},
    widgets::{Align, Button, Label, Obj, Part, Spinbox, WidgetError},
};

#[cfg(target_arch = "xtensa")]
use core::time::Duration;
#[cfg(not(target_arch = "xtensa"))]
use std::time::Duration;

/// Held-key state, written by the `−` / `+` buttons and read by LVGL.
///
/// `static` so it outlives the [`KeypadIndev`] (LVGL stores a pointer to it).
static KEYPAD: KeypadState = KeypadState::new();

/// Setpoint range and presets (×100, two decimals: 0.00 … 100.00).
const MIN: i32 = 0;
const MAX: i32 = 10_000;

#[derive(Default)]
struct SetpointView {
    spinbox: Option<Spinbox<'static>>,
    btn_minus: Option<Button<'static>>,
    btn_plus: Option<Button<'static>>,
    presets: heapless::Vec<(Button<'static>, i32), 3>,
    _labels: heapless::Vec<Label<'static>, 8>,
    // Layout containers — kept alive because dropping an Obj wrapper deletes the
    // LVGL object (and all its children).
    _rows: heapless::Vec<Obj<'static>, 2>,

    // The focus group (exposed via input_group) and the input devices. All
    // survive for the life of the view.
    group: Option<Group>,
    keypad: Option<KeypadIndev>,
    pointer: Option<PointerIndev>,
}

impl View for SetpointView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x12, 0x12, 0x20))
            .bg_opa(255)
            .pad_all(8)
            .pad_gap(8);
        let bg_style = _b.build();
        container
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&bg_style, Part::Main);

        // ── Title ────────────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(color_make(0xe8, 0xe8, 0xf2));
        let text_style = _b.build();

        let title = Label::new(container)?;
        title.text("Setpoint").add_style(&text_style, Part::Main);
        let _ = self._labels.push(title);

        // ── Value row: [ − ]  spinbox  [ + ] ─────────────────────────────
        let row = Obj::new(container)?;
        row.size(300, 56);
        row.set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(oxivgl::enums::ObjFlag::SCROLLABLE);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Indigo)).radius(8);
        let key_style = _b.build();

        let btn_minus = Button::new(&row)?;
        btn_minus.size(48, 48).add_style(&key_style, Part::Main);
        btn_minus.style_bg_image_src_symbol(&symbols::MINUS, Selector::DEFAULT);
        btn_minus.bubble_events();

        let spinbox = Spinbox::new(&row)?;
        spinbox
            .set_range(MIN, MAX)
            .set_digit_format(5, 2)
            .set_step(10)
            .step_prev();
        spinbox.size(150, 48);

        let btn_plus = Button::new(&row)?;
        btn_plus.size(48, 48).add_style(&key_style, Part::Main);
        btn_plus.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);
        btn_plus.bubble_events();

        // ── Preset row (touch): Min | Half | Max ─────────────────────────
        let preset_row = Obj::new(container)?;
        preset_row.size(300, 44);
        preset_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(oxivgl::enums::ObjFlag::SCROLLABLE);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::BlueGrey)).radius(8);
        let preset_style = _b.build();

        for (name, value) in [("Min", MIN), ("Half", MAX / 2), ("Max", MAX)] {
            let btn = Button::new(&preset_row)?;
            btn.size(88, 36).add_style(&preset_style, Part::Main);
            btn.bubble_events();
            let lbl = Label::new(&btn)?;
            lbl.text(name)
                .add_style(&text_style, Part::Main)
                .align(Align::Center, 0, 0);
            let _ = self.presets.push((btn, value));
            let _ = self._labels.push(lbl);
        }

        // ── Hint ─────────────────────────────────────────────────────────
        let hint = Label::new(container)?;
        hint.text("Hold -/+ to repeat - tap a preset")
            .add_style(&text_style, Part::Main);
        let _ = self._labels.push(hint);

        // ── Focus group + input devices ──────────────────────────────────
        // The spinbox is the single focusable target; the keypad's repeated
        // UP/DOWN keys land on it. Created after lv_init so the navigator can
        // bind input_group() to the keypad.
        let group = Group::new()?;
        group.add_obj(&spinbox); // also focuses it (first member)

        // Hold-to-repeat: after 400 ms, then every 80 ms while a key is held.
        self.keypad = Some(KeypadIndev::new(&KEYPAD)?.with_repeat(
            Duration::from_millis(400),
            Duration::from_millis(80),
        ));

        // The touchscreen device. On hardware, return your panel's reading
        // (e.g. `ft6336u::read_touch()`); on host it stays idle and the SDL
        // mouse provides the live POINTER input.
        self.pointer = Some(PointerIndev::new_with(read_touch)?);

        self.spinbox = Some(spinbox);
        self.btn_minus = Some(btn_minus);
        self.btn_plus = Some(btn_plus);
        let _ = self._rows.push(row);
        let _ = self._rows.push(preset_row);
        self.group = Some(group);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        // Hold-to-repeat: a held button reports a held key; releasing clears it.
        // LVGL derives the repeat and re-sends UP/DOWN to the focused spinbox.
        for (btn, key) in [
            (&self.btn_minus, Key::DOWN),
            (&self.btn_plus, Key::UP),
        ] {
            if let Some(b) = btn {
                if event.matches(b, EventCode::PRESSED) {
                    KEYPAD.press(key);
                } else if event.matches(b, EventCode::RELEASED) {
                    KEYPAD.release();
                }
            }
        }

        // Presets: a tap jumps the value directly.
        if let Some(spinbox) = &self.spinbox {
            for (btn, value) in &self.presets {
                if event.matches(btn, EventCode::CLICKED) {
                    spinbox.set_value(*value);
                }
            }
        }

        NavAction::None
    }

    fn input_group(&self) -> Option<oxivgl::group::GroupRef> {
        self.group.as_ref().map(|g| g.as_ref())
    }
}

/// Touch source for the [`PointerIndev`]. `Some((x, y))` = pressed at that
/// coordinate, `None` = released.
///
/// On hardware, return your panel's reading here (e.g. `ft6336u::read_touch()`).
/// On the host there is no panel, so this is always `None` and the SDL mouse
/// drives the POINTER device instead.
fn read_touch() -> Option<(u16, u16)> {
    None
}

// ── Entry point ──────────────────────────────────────────────────────────────

oxivgl_examples_common::example_main_nav!(SetpointView::default());
