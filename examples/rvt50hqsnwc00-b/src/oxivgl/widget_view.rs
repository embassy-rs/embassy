//! OxivGL widget showcase view for the Riverdi RVT50 (LVGL v9.5).

extern crate alloc;

use defmt::info;
use oxivgl::enums::{EventCode, ObjFlag, ObjState};
use oxivgl::event::Event;
use oxivgl::view::{register_event_on, NavAction, View};
use oxivgl::widgets::{Bar, Button, Checkbox, Label, Obj, Slider, Switch, WidgetError};
use oxivgl_sys::lv_screen_active;

/// Multi-widget demo inspired by LVGL `lv_demo_widgets` / OxivGL examples.
#[derive(Default)]
pub struct WidgetView {
    info_label: Option<Label<'static>>,
    bar: Option<Bar<'static>>,
    _slider: Option<Slider<'static>>,
    _btn: Option<Button<'static>>,
    _switch: Option<Switch<'static>>,
    _checkbox: Option<Checkbox<'static>>,
    clicks: u32,
}

impl View for WidgetView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        container.bg_color(0x102030).bg_opa(255);

        let title = Label::new(container)?;
        title
            .text("OxivGL on Riverdi RVT50")
            .pos(24, 20)
            .text_color(0xDCE6FF);

        let subtitle = Label::new(container)?;
        subtitle
            .text("C LVGL v9.5 via oxivgl-sys — Embassy LTDC")
            .pos(24, 48)
            .text_color(0x96AACC);

        let btn = Button::new(container)?;
        btn.size(180, 52)
            .pos(24, 90)
            .add_flag(ObjFlag::CLICKABLE)
            .bubble_events();
        let btn_label = Label::new(&btn)?;
        btn_label.text("Tap counter").center();

        let slider = Slider::new(container)?;
        slider
            .size(320, 20)
            .pos(24, 170)
            .add_flag(ObjFlag::CLICKABLE)
            .bubble_events();
        slider.set_value(35);

        let bar = Bar::new(container)?;
        bar.size(320, 24).pos(24, 210);
        bar.set_range(100.0).set_value(35.0);

        let switch = Switch::new(container)?;
        switch
            .pos(24, 270)
            .add_flag(ObjFlag::CLICKABLE)
            .bubble_events();

        let checkbox = Checkbox::new(container)?;
        checkbox
            .text("Highlight panel")
            .pos(24, 320)
            .add_flag(ObjFlag::CLICKABLE)
            .bubble_events();

        let info = Label::new(container)?;
        info.text("Interact with the widgets…")
            .pos(24, 380)
            .width(480);

        self._btn = Some(btn);
        self._slider = Some(slider);
        self.bar = Some(bar);
        self._switch = Some(switch);
        self._checkbox = Some(checkbox);
        self.info_label = Some(info);
        Ok(())
    }

    fn register_events(&mut self) {
        // Screen-level handler catches bubbled events from children.
        // SAFETY: lv_init() completed; screen is active.
        register_event_on(self, unsafe { lv_screen_active() });

        if let Some(ref btn) = self._btn {
            register_event_on(self, btn.handle());
        }
        if let Some(ref slider) = self._slider {
            register_event_on(self, slider.handle());
        }
        if let Some(ref switch) = self._switch {
            register_event_on(self, switch.handle());
        }
        if let Some(ref checkbox) = self._checkbox {
            register_event_on(self, checkbox.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let code = event.code();
        match code {
            EventCode::CLICKED
            | EventCode::SHORT_CLICKED
            | EventCode::SINGLE_CLICKED => {
                info!("oxivgl widget click ({:?})", code.0);
                self.clicks += 1;
                self.refresh_info();
            }
            EventCode::VALUE_CHANGED => {
                info!("oxivgl widget value changed");
                if let (Some(slider), Some(bar)) = (self._slider.as_ref(), self.bar.as_ref()) {
                    let _ = bar.set_value(slider.get_value() as f32);
                }
                self.refresh_info();
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
    fn refresh_info(&mut self) {
        let level = self._slider.as_ref().map(|s| s.get_value()).unwrap_or(0);
        let switch_on = self
            ._switch
            .as_ref()
            .map(|s| s.has_state(ObjState::CHECKED))
            .unwrap_or(false);
        let checked = self
            ._checkbox
            .as_ref()
            .map(|c| c.has_state(ObjState::CHECKED))
            .unwrap_or(false);
        if let Some(ref info) = self.info_label {
            let _ = info.text(&alloc::format!(
                "taps={}  level={}  switch={}  check={}",
                self.clicks,
                level,
                if switch_on { "ON" } else { "off" },
                if checked { "ON" } else { "off" },
            ));
        }
    }
}
