#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Menu Keypad — focus-navigated menu driven by on-screen (or hardware) keys
//!
//! A vertical menu whose items are navigated with **discrete keys** rather than
//! by tapping each one. The three on-screen buttons at the bottom (`<`, `OK`,
//! `>`) feed an LVGL `KEYPAD` input device: tapping `>` moves the focus
//! highlight to the next item, `<` to the previous, and `OK` activates the
//! focused item.
//!
//! This is "method 1" — **LVGL owns the focus**. The view exposes its
//! [`Group`] via [`View::input_group`], and the navigator routes the keypad to
//! it. The exact same code works when the keys come from hardware buttons (e.g.
//! the M5Stack Fire's three front buttons) — only the *producer* of the key
//! presses differs.
//!
//! Demonstrates:
//! - [`KeypadState`] + [`KeypadIndev`] — a keypad device fed by the application
//! - On-screen buttons simulating key presses via `PRESSED` / `RELEASED`
//! - [`View::input_group`] routing focus to the menu's [`Group`]
//! - LVGL focus highlighting and `ENTER`-to-activate

use oxivgl::{
    enums::{EventCode, Key, ObjFlag, ObjState},
    event::Event,
    group::Group,
    indev::{KeypadIndev, KeypadState},
    layout::{FlexAlign, FlexFlow},
    style::{Palette, StyleBuilder, color_make, palette_darken, palette_lighten, palette_main},
    view::{NavAction, View},
    widgets::{Align, Button, Label, Obj, Part, WidgetError},
};

/// Shared key state, written by the on-screen buttons and read by LVGL.
///
/// `static` so it outlives the [`KeypadIndev`] (LVGL stores a pointer to it).
static KEYPAD: KeypadState = KeypadState::new();

/// Menu entries.
const ITEMS: [&str; 5] = ["Display", "Audio", "Network", "Power", "About"];

#[derive(Default)]
struct MenuView {
    // Layout containers + labels kept alive: an owned widget wrapper deletes
    // its LVGL object on drop, so anything not stored would vanish.
    _menu_col: Option<Obj<'static>>,
    _nav_row: Option<Obj<'static>>,
    _title: Option<Label<'static>>,
    _item_labels: heapless::Vec<Label<'static>, 8>,
    _nav_labels: heapless::Vec<Label<'static>, 4>,

    // Focusable menu items, in the same order as `ITEMS`.
    items: heapless::Vec<Button<'static>, 8>,

    // On-screen key buttons (touch → simulated keys). Not in the focus group.
    btn_prev: Option<Button<'static>>,
    btn_ok: Option<Button<'static>>,
    btn_next: Option<Button<'static>>,

    status: Option<Label<'static>>,

    // The focus group (exposed via input_group) and the keypad device that
    // drives it. Both survive for the life of the view.
    group: Option<Group>,
    keypad: Option<KeypadIndev>,
}

impl View for MenuView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x12, 0x12, 0x20))
            .bg_opa(255)
            .pad_all(6)
            .pad_gap(4);
        let bg_style = _b.build();

        container
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&bg_style, Part::Main);

        // ── Title ────────────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Blue, 2));
        let title_style = _b.build();

        let title = Label::new(container)?;
        title.text("Settings").add_style(&title_style, Part::Main);
        self._title = Some(title);

        // ── Menu column (focusable items) ────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x1c, 0x1c, 0x30))
            .bg_opa(255)
            .radius(8)
            .pad_all(6)
            .pad_gap(4);
        let panel_style = _b.build();

        let menu_col = Obj::new(container)?;
        menu_col.size(288, 140);
        menu_col
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&panel_style, Part::Main)
            .remove_flag(ObjFlag::SCROLLABLE);

        // Item background, item text, and the focus-highlight applied to the
        // focused item (LVGL sets ObjState::FOCUSED on the keypad's current item).
        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_darken(Palette::BlueGrey, 4)).radius(5);
        let item_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Blue));
        let item_focus_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.text_color(color_make(0xe8, 0xe8, 0xf2));
        let item_text_style = _b.build();

        let group = Group::new()?;

        for name in ITEMS {
            let btn = Button::new(&menu_col)?;
            btn.size(264, 22);
            btn.add_style(&item_style, Part::Main)
                .add_style(&item_focus_style, ObjState::FOCUSED)
                // Bubble events so on_event() sees CLICKED (from touch or ENTER).
                .add_flag(ObjFlag::EVENT_BUBBLE);

            let label = Label::new(&btn)?;
            label
                .text(name)
                .add_style(&item_text_style, Part::Main)
                .align(Align::Center, 0, 0);

            // Add the item to the focus group so keys navigate between items.
            group.add_obj(&btn);
            let _ = self.items.push(btn);
            let _ = self._item_labels.push(label);
        }

        // ── On-screen key row: `<`  OK  `>` ──────────────────────────────
        let nav_row = Obj::new(container)?;
        nav_row.size(280, 38);
        nav_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Indigo)).radius(8);
        let key_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.text_color(color_make(0xff, 0xff, 0xff));
        let key_text_style = _b.build();

        let (prev_btn, prev_lbl) = make_key_button(&nav_row, "<", &key_style, &key_text_style)?;
        let (ok_btn, ok_lbl) = make_key_button(&nav_row, "OK", &key_style, &key_text_style)?;
        let (next_btn, next_lbl) = make_key_button(&nav_row, ">", &key_style, &key_text_style)?;
        self.btn_prev = Some(prev_btn);
        self.btn_ok = Some(ok_btn);
        self.btn_next = Some(next_btn);
        let _ = self._nav_labels.push(prev_lbl);
        let _ = self._nav_labels.push(ok_lbl);
        let _ = self._nav_labels.push(next_lbl);

        // ── Status line ──────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Green, 1));
        let status_style = _b.build();

        let status = Label::new(container)?;
        status
            .text("Use < > to move, OK to select")
            .add_style(&status_style, Part::Main);
        self.status = Some(status);

        // The keypad device, fed by KEYPAD. Created here (after lv_init) so the
        // navigator can bind input_group() to it. The first group member is
        // focused automatically when added above.
        self.keypad = Some(KeypadIndev::new(&KEYPAD)?);

        self._menu_col = Some(menu_col);
        self._nav_row = Some(nav_row);
        self.group = Some(group);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        // On-screen key buttons → simulated keys. Press on touch-down, release
        // on touch-up, so a tap is one step and a hold repeats (LVGL derives
        // repeat from the held state).
        for (btn, key) in [
            (&self.btn_prev, Key::PREV),
            (&self.btn_ok, Key::ENTER),
            (&self.btn_next, Key::NEXT),
        ] {
            if let Some(b) = btn {
                if event.matches(b, EventCode::PRESSED) {
                    KEYPAD.press(key);
                } else if event.matches(b, EventCode::RELEASED) {
                    KEYPAD.release();
                }
            }
        }

        // A menu item was activated (ENTER on the focused item, or a direct
        // touch). Reflect it in the status line.
        for (i, item) in self.items.iter().enumerate() {
            if event.matches(item, EventCode::CLICKED) {
                if let Some(status) = &self.status {
                    use core::fmt::Write;
                    let mut buf = heapless::String::<32>::new();
                    let _ = write!(buf, "Selected: {}", ITEMS[i]);
                    status.text(&buf);
                }
            }
        }

        NavAction::None
    }

    fn input_group(&self) -> Option<oxivgl::group::GroupRef> {
        self.group.as_ref().map(|g| g.as_ref())
    }
}

/// Build one on-screen key button (`<`, `OK`, `>`) with a centred label.
fn make_key_button(
    parent: &Obj<'static>,
    text: &str,
    style: &oxivgl::style::Style,
    label_style: &oxivgl::style::Style,
) -> Result<(Button<'static>, Label<'static>), WidgetError> {
    let btn = Button::new(parent)?;
    btn.size(72, 36);
    btn.add_style(style, Part::Main)
        // Not added to the focus group — these are touch controls, not items.
        .add_flag(ObjFlag::EVENT_BUBBLE);
    let label = Label::new(&btn)?;
    label
        .text(text)
        .add_style(label_style, Part::Main)
        .align(Align::Center, 0, 0);
    // Returned so the caller can keep it alive — dropping it deletes the label.
    Ok((btn, label))
}

// ── Entry point ──────────────────────────────────────────────────────────────

oxivgl_examples_common::example_main_nav!(MenuView::default());
