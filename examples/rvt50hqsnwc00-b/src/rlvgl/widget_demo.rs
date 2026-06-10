//! Multi-widget UI inspired by LVGL / OxivGL widget examples.
//!
//! Showcases label, button, slider, progress bar, switch, and checkbox on the
//! Riverdi RVT50 800×480 panel. Uses [rlvgl](https://github.com/SoftOboros/rlvgl)
//! (Embassy's pure-Rust LVGL-style stack) rather than the C LVGL tree from
//! Riverdi's `riverdi-50-stm32u5-lvgl/Middlewares/Third_Party/LVGL` Cube port.

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use rlvgl::core::draw::draw_widget_bg;
use rlvgl::core::event::Event;
use rlvgl::core::renderer::Renderer;
use rlvgl::core::style::Style;
use rlvgl::core::{WidgetNode, widget::Color, widget::Rect, widget::Widget};
use rlvgl::widgets::{
    checkbox::Checkbox, container::Container, label::Label, progress::ProgressBar,
    slider::Slider, switch::Switch,
};

use crate::rlvgl::pressable_button::PressableButton;

const PANEL_BG: Color = Color(16, 28, 48, 255);
const TITLE_COLOR: Color = Color(220, 230, 255, 255);
const MUTED_COLOR: Color = Color(150, 170, 200, 255);
const ACCENT: Color = Color(40, 96, 200, 255);
const ACCENT_FILL: Color = Color(70, 140, 255, 255);

/// Shared demo state updated by interactive widgets.
struct DemoState {
    level: i32,
    clicks: u32,
    switch_on: bool,
    checkbox_on: bool,
}

/// Progress bar that mirrors [`DemoState::level`].
struct LevelBar {
    bounds: Rect,
    style: Style,
    bar_color: Color,
    state: Rc<RefCell<DemoState>>,
}

impl LevelBar {
    fn new(bounds: Rect, state: Rc<RefCell<DemoState>>) -> Self {
        Self {
            bounds,
            style: Style::default(),
            bar_color: ACCENT_FILL,
            state,
        }
    }
}

impl Widget for LevelBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        let level = self.state.borrow().level;
        let mut bar = ProgressBar::new(self.bounds, 0, 100);
        bar.style = self.style;
        bar.bar_color = self.bar_color;
        bar.set_value(level);
        bar.draw(renderer);
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }
}

/// Slider that writes into [`DemoState::level`].
struct LevelSlider {
    inner: Slider,
    state: Rc<RefCell<DemoState>>,
}

impl LevelSlider {
    fn new(bounds: Rect, state: Rc<RefCell<DemoState>>) -> Self {
        let mut inner = Slider::new(bounds, 0, 100);
        inner.style.bg_color = Color(30, 44, 72, 255);
        inner.style.border_color = ACCENT;
        inner.style.border_width = 1;
        inner.style.radius = 6;
        inner.knob_color = Color(240, 244, 255, 255);
        inner.set_value(state.borrow().level);

        Self { inner, state }
    }
}

impl Widget for LevelSlider {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        self.inner.draw(renderer);
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if self.inner.handle_event(event) {
            self.state.borrow_mut().level = self.inner.value();
            return true;
        }
        false
    }
}

/// Status line that reflects the full demo state.
struct StatusLabel {
    bounds: Rect,
    style: Style,
    text_color: Color,
    state: Rc<RefCell<DemoState>>,
}

impl StatusLabel {
    fn new(bounds: Rect, state: Rc<RefCell<DemoState>>) -> Self {
        Self {
            bounds,
            style: Style::default(),
            text_color: MUTED_COLOR,
            state,
        }
    }

    fn text(&self) -> alloc::string::String {
        let s = self.state.borrow();
        alloc::format!(
            "level={}  taps={}  switch={}  check={}",
            s.level,
            s.clicks,
            if s.switch_on { "ON" } else { "off" },
            if s.checkbox_on { "ON" } else { "off" },
        )
    }
}

impl Widget for StatusLabel {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        draw_widget_bg(renderer, self.bounds, &self.style);
        renderer.draw_text(
            (self.bounds.x, self.bounds.y + self.bounds.height),
            &self.text(),
            self.text_color,
        );
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }
}

/// Switch wired to [`DemoState::switch_on`].
struct DemoSwitch {
    inner: Switch,
    state: Rc<RefCell<DemoState>>,
}

impl DemoSwitch {
    fn new(bounds: Rect, state: Rc<RefCell<DemoState>>) -> Self {
        let mut inner = Switch::new(bounds);
        inner.style.bg_color = Color(48, 58, 82, 255);
        inner.style.border_color = ACCENT;
        inner.style.border_width = 1;
        inner.style.radius = 12;
        inner.knob_color = Color(230, 236, 248, 255);
        inner.set_on(state.borrow().switch_on);
        Self { inner, state }
    }
}

impl Widget for DemoSwitch {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        self.inner.draw(renderer);
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if self.inner.handle_event(event) {
            self.state.borrow_mut().switch_on = self.inner.is_on();
            return true;
        }
        false
    }
}

/// Checkbox wired to [`DemoState::checkbox_on`].
struct DemoCheckbox {
    inner: Checkbox,
    state: Rc<RefCell<DemoState>>,
}

impl DemoCheckbox {
    fn new(text: &str, bounds: Rect, state: Rc<RefCell<DemoState>>) -> Self {
        let mut inner = Checkbox::new(text, bounds);
        inner.style.bg_color = Color(0, 0, 0, 0);
        inner.text_color = TITLE_COLOR;
        inner.check_color = ACCENT_FILL;
        inner.set_checked(state.borrow().checkbox_on);
        Self { inner, state }
    }
}

impl Widget for DemoCheckbox {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        self.inner.draw(renderer);
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if self.inner.handle_event(event) {
            self.state.borrow_mut().checkbox_on = self.inner.is_checked();
            return true;
        }
        false
    }
}

fn caption_label(text: &str, rect: Rect) -> Label {
    let mut label = Label::new(text, rect);
    label.text_color = MUTED_COLOR;
    label.style.bg_color = Color(0, 0, 0, 0);
    label
}

fn card_container(rect: Rect) -> Container {
    let mut card = Container::new(rect);
    card.style.bg_color = Color(24, 38, 64, 255);
    card.style.border_color = Color(50, 72, 110, 255);
    card.style.border_width = 1;
    card.style.radius = 10;
    card
}

/// Build the multi-widget demo tree for the given panel size.
pub fn build_widget_demo(width: i32, height: i32) -> Rc<RefCell<WidgetNode>> {
    let state = Rc::new(RefCell::new(DemoState {
        level: 35,
        clicks: 0,
        switch_on: false,
        checkbox_on: true,
    }));

    let mut root_container = Container::new(Rect {
        x: 0,
        y: 0,
        width,
        height,
    });
    root_container.style.bg_color = PANEL_BG;

    let mut title = Label::new(
        "RVT50 Widget Demo (rlvgl / LVGL-style)",
        Rect {
            x: 32,
            y: 28,
            width: 520,
            height: 32,
        },
    );
    title.text_color = TITLE_COLOR;
    title.style.bg_color = Color(0, 0, 0, 0);

    let mut subtitle = Label::new(
        "Riverdi 800x480 — Embassy LTDC RGB565",
        Rect {
            x: 32,
            y: 58,
            width: 420,
            height: 24,
        },
    );
    subtitle.text_color = MUTED_COLOR;
    subtitle.style.bg_color = Color(0, 0, 0, 0);

    let button = Rc::new(RefCell::new(PressableButton::new(
        "Tap counter",
        Rect {
            x: 32,
            y: 110,
            width: 200,
            height: 56,
        },
    )));
    {
        let mut btn = button.borrow_mut();
        btn.style_mut().bg_color = Color(230, 236, 248, 255);
        btn.style_mut().border_color = ACCENT;
        btn.style_mut().border_width = 2;
        btn.style_mut().radius = 8;
        btn.set_pressed_colors(Color(170, 190, 230, 255), Color(20, 64, 160, 255));

        let state = state.clone();
        btn.set_on_click(move |b| {
            let n = {
                let mut s = state.borrow_mut();
                s.clicks += 1;
                s.clicks
            };
            b.set_text(alloc::format!("Tapped {n}"));
        });
    }

    let slider = Rc::new(RefCell::new(LevelSlider::new(
        Rect {
            x: 32,
            y: 210,
            width: 320,
            height: 40,
        },
        state.clone(),
    )));

    let progress = Rc::new(RefCell::new(LevelBar::new(
        Rect {
            x: 32,
            y: 270,
            width: 320,
            height: 28,
        },
        state.clone(),
    )));
    {
        let mut bar = progress.borrow_mut();
        bar.style.bg_color = Color(30, 44, 72, 255);
        bar.style.border_color = ACCENT;
        bar.style.border_width = 1;
        bar.style.radius = 6;
    }

    let switch = Rc::new(RefCell::new(DemoSwitch::new(
        Rect {
            x: 32,
            y: 340,
            width: 72,
            height: 36,
        },
        state.clone(),
    )));

    let checkbox = Rc::new(RefCell::new(DemoCheckbox::new(
        "Enable highlight",
        Rect {
            x: 140,
            y: 340,
            width: 220,
            height: 36,
        },
        state.clone(),
    )));

    let status = Rc::new(RefCell::new(StatusLabel::new(
        Rect {
            x: 32,
            y: 400,
            width: 480,
            height: 24,
        },
        state,
    )));

    // Decorative meter card on the right (static arc-like gauge using rounded rects).
    let meter_card = card_container(Rect {
        x: 420,
        y: 110,
        width: 340,
        height: 280,
    });
    let meter_title = caption_label(
        "Widgets",
        Rect {
            x: 440,
            y: 130,
            width: 120,
            height: 20,
        },
    );
    let meter_items = [
        "Label",
        "Button",
        "Slider",
        "Bar",
        "Switch",
        "Checkbox",
    ];
    let meter_children: alloc::vec::Vec<WidgetNode> = meter_items
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let y = 168 + i as i32 * 34;
            let mut chip = Label::new(
                *name,
                Rect {
                    x: 440,
                    y,
                    width: 120,
                    height: 22,
                },
            );
            chip.text_color = TITLE_COLOR;
            chip.style.bg_color = Color(36, 54, 88, 255);
            chip.style.border_color = ACCENT;
            chip.style.border_width = 1;
            chip.style.radius = 4;

            WidgetNode {
                widget: Rc::new(RefCell::new(chip)),
                children: alloc::vec![],
                tag: None,
            }
        })
        .collect();

    let mut card_children = alloc::vec![WidgetNode {
        widget: Rc::new(RefCell::new(meter_title)),
        children: alloc::vec![],
        tag: None,
    }];
    card_children.extend(meter_children);

    Rc::new(RefCell::new(WidgetNode {
        widget: Rc::new(RefCell::new(root_container)),
        children: alloc::vec![
            WidgetNode {
                widget: Rc::new(RefCell::new(title)),
                children: alloc::vec![],
                tag: Some("title"),
            },
            WidgetNode {
                widget: Rc::new(RefCell::new(subtitle)),
                children: alloc::vec![],
                tag: None,
            },
            WidgetNode {
                widget: button,
                children: alloc::vec![],
                tag: Some("counter-button"),
            },
            WidgetNode {
                widget: widget_node_from_label(caption_label(
                    "Brightness",
                    Rect {
                        x: 32,
                        y: 188,
                        width: 160,
                        height: 20,
                    },
                )),
                children: alloc::vec![],
                tag: None,
            },
            WidgetNode {
                widget: slider,
                children: alloc::vec![],
                tag: Some("level-slider"),
            },
            WidgetNode {
                widget: progress,
                children: alloc::vec![],
                tag: Some("level-bar"),
            },
            WidgetNode {
                widget: widget_node_from_label(caption_label(
                    "Switch",
                    Rect {
                        x: 32,
                        y: 318,
                        width: 80,
                        height: 20,
                    },
                )),
                children: alloc::vec![],
                tag: None,
            },
            WidgetNode {
                widget: switch,
                children: alloc::vec![],
                tag: Some("demo-switch"),
            },
            WidgetNode {
                widget: checkbox,
                children: alloc::vec![],
                tag: Some("demo-checkbox"),
            },
            WidgetNode {
                widget: status,
                children: alloc::vec![],
                tag: Some("status"),
            },
            WidgetNode {
                widget: Rc::new(RefCell::new(meter_card)),
                children: card_children,
                tag: Some("widget-card"),
            },
        ],
        tag: Some("widget-demo-root"),
    }))
}

fn widget_node_from_label(label: Label) -> Rc<RefCell<dyn Widget>> {
    Rc::new(RefCell::new(label))
}
