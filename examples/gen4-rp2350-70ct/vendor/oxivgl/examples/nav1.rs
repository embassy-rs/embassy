#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Navigation 1 — Two-screen push/pop demo
//!
//! A **Dashboard** screen with temperature, humidity and a "Settings" button.
//! Clicking Settings pushes a **SettingsView** with brightness/volume sliders
//! and a "Back" button that pops to the dashboard.
//!
//! Demonstrates:
//! - `NavAction::push` / `NavAction::Pop` for screen transitions
//! - `will_hide` / `did_show` lifecycle hooks
//! - State preservation across push/pop (dashboard counter keeps ticking)
//! - `ScreenAnim` for animated transitions

use oxivgl::{
    enums::{EventCode, ObjFlag},
    event::Event,
    layout::{FlexAlign, FlexFlow},
    style::{Palette, StyleBuilder, color_make, palette_darken, palette_lighten, palette_main},
    timer::Timer,
    view::{NavAction, View},
    widgets::{
        Align, Arc, ArcMode, Bar, Button, Label, Obj, Part,
        ScreenAnim, ScreenAnimType, Slider, WidgetError,
    },
};

// ── Shared animation config ─────────────────────────────────────────────────

fn slide_right() -> Option<ScreenAnim> {
    Some(ScreenAnim {
        anim_type: ScreenAnimType::MoveRight,
        duration_ms: 300,
        delay_ms: 0,
    })
}

fn slide_left() -> Option<ScreenAnim> {
    Some(ScreenAnim {
        anim_type: ScreenAnimType::MoveLeft,
        duration_ms: 300,
        delay_ms: 0,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard — home screen with live sensor gauges
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Default)]
struct Dashboard {
    // Layout containers (kept alive to prevent child deletion)
    _title_row: Option<Obj<'static>>,
    _gauge_row: Option<Obj<'static>>,
    _humidity_col: Option<Obj<'static>>,

    temp_arc: Option<Arc<'static>>,
    temp_label: Option<Label<'static>>,
    humidity_bar: Option<Bar<'static>>,
    humidity_label: Option<Label<'static>>,
    uptime_label: Option<Label<'static>>,
    settings_btn: Option<Button<'static>>,

    // Persistent state (survives push/pop cycles)
    timer: Option<Timer>,
    tick_count: u32,
    temperature: i32, // tenths of a degree
    humidity: i32,    // percent
}

impl View for Dashboard {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x1a, 0x1a, 0x2e))
            .bg_opa(255)
            .pad_all(8)
            .pad_gap(6);
        let bg_style = _b.build();

        container
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&bg_style, Part::Main);

        let mut _b = StyleBuilder::new();
        _b.text_color(palette_main(Palette::BlueGrey));
        let title_style = _b.build();

        // ── Title bar ────────────────────────────────────────────────────
        let title_row = Obj::new(container)?;
        title_row.size(304, 28);
        title_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceBetween, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let title = Label::new(&title_row)?;
        title.text("Dashboard").add_style(&title_style, Part::Main);

        let uptime = Label::new(&title_row)?;
        uptime.text("0:00").add_style(&title_style, Part::Main);
        self.uptime_label = Some(uptime);

        // ── Temperature gauge (arc) ──────────────────────────────────────
        let gauge_row = Obj::new(container)?;
        gauge_row.size(304, 100);
        gauge_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let temp_arc = Arc::new(&gauge_row)?;
        temp_arc.size(90, 90);
        temp_arc
            .remove_flag(ObjFlag::CLICKABLE)
            .remove_flag(ObjFlag::SCROLLABLE);
        temp_arc
            .set_rotation(135)
            .set_bg_angles(0, 270)
            .set_mode(ArcMode::Normal)
            .set_range(50.0); // 0–50 °C range

        let mut _b = StyleBuilder::new();
        _b.arc_color(palette_darken(Palette::Blue, 3)).arc_width(8);
        let arc_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.arc_color(palette_main(Palette::Cyan)).arc_width(8);
        let arc_ind_style = _b.build();

        temp_arc
            .add_style(&arc_style, Part::Main)
            .add_style(&arc_ind_style, Part::Indicator);
        self.temp_arc = Some(temp_arc);

        let temp_label = Label::new(&gauge_row)?;
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Cyan, 2));
        let value_style = _b.build();
        temp_label.text("--.-C").add_style(&value_style, Part::Main);
        self.temp_label = Some(temp_label);

        // Humidity section
        let humidity_col = Obj::new(&gauge_row)?;
        humidity_col.size(120, 80);
        humidity_col
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let hum_title = Label::new(&humidity_col)?;
        hum_title.text("Humidity").add_style(&title_style, Part::Main);

        let humidity_bar = Bar::new(&humidity_col)?;
        humidity_bar.size(100, 12);
        humidity_bar.set_range(100.0);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_darken(Palette::Teal, 3));
        let bar_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Teal));
        let bar_ind_style = _b.build();

        humidity_bar
            .add_style(&bar_style, Part::Main)
            .add_style(&bar_ind_style, Part::Indicator);
        self.humidity_bar = Some(humidity_bar);

        let humidity_label = Label::new(&humidity_col)?;
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Teal, 2));
        let hum_val_style = _b.build();
        humidity_label
            .text("--%")
            .add_style(&hum_val_style, Part::Main);
        self.humidity_label = Some(humidity_label);

        // ── Settings button ──────────────────────────────────────────────
        let btn = Button::new(container)?;
        btn.size(160, 40);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_darken(Palette::Indigo, 2)).radius(8);
        let btn_style = _b.build();

        btn.add_style(&btn_style, Part::Main)
            .add_flag(ObjFlag::EVENT_BUBBLE);

        let btn_label = Label::new(&btn)?;
        btn_label.text("Settings").align(Align::Center, 0, 0);
        self.settings_btn = Some(btn);

        // Keep layout containers alive (Obj::drop would delete children).
        self._title_row = Some(title_row);
        self._gauge_row = Some(gauge_row);
        self._humidity_col = Some(humidity_col);

        // Create timer for live updates.
        if self.timer.is_none() {
            self.timer = Some(Timer::new(500)?);
        }

        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let Some(timer) = &self.timer {
            if timer.triggered() {
                self.tick_count += 1;
                self.temperature = 215 + ((self.tick_count as i32 * 7) % 40);
                self.humidity = 45 + ((self.tick_count as i32 * 3) % 30);
                self.refresh_widgets();
            }
        }
        Ok(NavAction::None)
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(btn) = &self.settings_btn {
            if event.matches(btn, EventCode::CLICKED) {
                return NavAction::push(
                    SettingsView::new(self.temperature, self.humidity),
                    slide_left(),
                );
            }
        }
        NavAction::None
    }

    fn did_show(&mut self) {
        self.refresh_widgets();
    }
}

impl Dashboard {
    fn refresh_widgets(&self) {
        if let Some(arc) = &self.temp_arc {
            // temperature is in tenths of a degree; arc range is 0–50.0
            arc.set_value(self.temperature as f32 / 10.0);
        }
        if let Some(lbl) = &self.temp_label {
            use core::fmt::Write;
            let mut buf = heapless::String::<16>::new();
            let _ = write!(buf, "{}.{}C", self.temperature / 10, self.temperature % 10);
            lbl.text(&buf);
        }
        if let Some(bar) = &self.humidity_bar {
            bar.set_value(self.humidity as f32);
        }
        if let Some(lbl) = &self.humidity_label {
            use core::fmt::Write;
            let mut buf = heapless::String::<8>::new();
            let _ = write!(buf, "{}%", self.humidity);
            lbl.text(&buf);
        }
        if let Some(lbl) = &self.uptime_label {
            use core::fmt::Write;
            let secs = self.tick_count / 2;
            let mut buf = heapless::String::<16>::new();
            let _ = write!(buf, "{}:{:02}", secs / 60, secs % 60);
            lbl.text(&buf);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SettingsView — pushed on top of the dashboard
// ═══════════════════════════════════════════════════════════════════════════════

struct SettingsView {
    // Layout containers (kept alive)
    _header: Option<Obj<'static>>,
    _info_row: Option<Obj<'static>>,
    _brightness_row: Option<Obj<'static>>,
    _volume_row: Option<Obj<'static>>,

    brightness_slider: Option<Slider<'static>>,
    volume_slider: Option<Slider<'static>>,
    brightness_label: Option<Label<'static>>,
    volume_label: Option<Label<'static>>,
    back_btn: Option<Button<'static>>,

    brightness: i32,
    volume: i32,
    incoming_temp: i32,
    incoming_hum: i32,
}

impl SettingsView {
    fn new(temp: i32, hum: i32) -> Self {
        Self {
            _header: None,
            _info_row: None,
            _brightness_row: None,
            _volume_row: None,
            brightness_slider: None,
            volume_slider: None,
            brightness_label: None,
            volume_label: None,
            back_btn: None,
            brightness: 75,
            volume: 50,
            incoming_temp: temp,
            incoming_hum: hum,
        }
    }

    fn fmt_pct(v: i32) -> heapless::String<8> {
        use core::fmt::Write;
        let mut buf = heapless::String::<8>::new();
        let _ = write!(buf, "{}%", v);
        buf
    }
}

impl View for SettingsView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x16, 0x21, 0x3e))
            .bg_opa(255)
            .pad_all(10)
            .pad_gap(8);
        let bg_style = _b.build();

        container
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&bg_style, Part::Main);

        // ── Header with back button ──────────────────────────────────────
        let header = Obj::new(container)?;
        header.size(304, 32);
        header
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let back_btn = Button::new(&header)?;
        back_btn.size(60, 28);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_darken(Palette::Red, 2)).radius(6);
        let btn_style = _b.build();

        back_btn
            .add_style(&btn_style, Part::Main)
            .add_flag(ObjFlag::EVENT_BUBBLE);

        let back_label = Label::new(&back_btn)?;
        back_label.text("< Back").align(Align::Center, 0, 0);
        self.back_btn = Some(back_btn);

        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Orange, 1));
        let title_style = _b.build();

        let title = Label::new(&header)?;
        title.text("Settings").add_style(&title_style, Part::Main);

        // ── Info row: show values from dashboard ─────────────────────────
        let info_row = Obj::new(container)?;
        info_row.size(280, 24);
        info_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let mut _b = StyleBuilder::new();
        _b.text_color(palette_darken(Palette::BlueGrey, 1));
        let info_style = _b.build();

        let temp_info = Label::new(&info_row)?;
        {
            use core::fmt::Write;
            let mut buf = heapless::String::<32>::new();
            let _ = write!(
                buf,
                "Temp: {}.{}C",
                self.incoming_temp / 10,
                self.incoming_temp % 10
            );
            temp_info.text(&buf);
        }
        temp_info.add_style(&info_style, Part::Main);

        let hum_info = Label::new(&info_row)?;
        {
            use core::fmt::Write;
            let mut buf = heapless::String::<16>::new();
            let _ = write!(buf, "Hum: {}%", self.incoming_hum);
            hum_info.text(&buf);
        }
        hum_info.add_style(&info_style, Part::Main);

        // ── Brightness slider ────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Amber, 2));
        let amber_style = _b.build();

        let (br, bs, bl) =
            create_slider_row(container, "Brightness", self.brightness, &amber_style, Palette::Amber)?;
        self._brightness_row = Some(br);
        self.brightness_slider = Some(bs);
        self.brightness_label = Some(bl);

        // ── Volume slider ────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::LightGreen, 2));
        let green_style = _b.build();

        let (vr, vs, vl) =
            create_slider_row(container, "Volume", self.volume, &green_style, Palette::LightGreen)?;
        self._volume_row = Some(vr);
        self.volume_slider = Some(vs);
        self.volume_label = Some(vl);

        // Keep layout containers alive.
        self._header = Some(header);
        self._info_row = Some(info_row);

        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let (Some(slider), Some(lbl)) = (&self.brightness_slider, &self.brightness_label) {
            let v = slider.get_value();
            if v != self.brightness {
                self.brightness = v;
                lbl.text(&Self::fmt_pct(v));
            }
        }
        if let (Some(slider), Some(lbl)) = (&self.volume_slider, &self.volume_label) {
            let v = slider.get_value();
            if v != self.volume {
                self.volume = v;
                lbl.text(&Self::fmt_pct(v));
            }
        }
        Ok(NavAction::None)
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(btn) = &self.back_btn {
            if event.matches(btn, EventCode::CLICKED) {
                return NavAction::Pop(slide_right());
            }
        }
        NavAction::None
    }

    fn will_hide(&mut self) {
        if let Some(slider) = &self.brightness_slider {
            self.brightness = slider.get_value();
        }
        if let Some(slider) = &self.volume_slider {
            self.volume = slider.get_value();
        }
    }
}

fn create_slider_row(
    parent: &Obj<'static>,
    name: &str,
    initial_value: i32,
    label_style: &oxivgl::style::Style,
    color: Palette,
) -> Result<(Obj<'static>, Slider<'static>, Label<'static>), WidgetError> {
    let row = Obj::new(parent)?;
    row.size(280, 40);
    row.set_flex_flow(FlexFlow::Row)
        .set_flex_align(FlexAlign::SpaceBetween, FlexAlign::Center, FlexAlign::Center)
        .remove_flag(ObjFlag::SCROLLABLE);

    let lbl = Label::new(&row)?;
    lbl.text(name).add_style(label_style, Part::Main);

    let slider = Slider::new(&row)?;
    slider.size(140, 10);
    slider.set_range(0, 100).set_value(initial_value);

    let mut _b = StyleBuilder::new();
    _b.bg_color(palette_main(color));
    let knob_style = _b.build();

    let mut _b = StyleBuilder::new();
    _b.bg_color(palette_darken(color, 1));
    let ind_style = _b.build();

    let mut _b = StyleBuilder::new();
    _b.bg_color(palette_darken(color, 3));
    let track_style = _b.build();

    slider
        .add_style(&track_style, Part::Main)
        .add_style(&ind_style, Part::Indicator)
        .add_style(&knob_style, Part::Knob);

    let val_lbl = Label::new(&row)?;
    val_lbl
        .text(&SettingsView::fmt_pct(initial_value))
        .add_style(label_style, Part::Main);

    Ok((row, slider, val_lbl))
}

// ── Entry point ──────────────────────────────────────────────────────────────

oxivgl_examples_common::example_main_nav!(Dashboard::default());
