//! Widget demo scene for the gen4 800×480 panel (rlvgl, no C LVGL).

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use defmt::info;
use rlvgl::core::WidgetNode;
use rlvgl::core::event::Event;
use rlvgl::core::style::Style;
use rlvgl::core::widget::{Color, Rect, Widget};
use rlvgl::widgets::bar::{Bar, BarMode};
use rlvgl::widgets::button::Button;
use rlvgl::widgets::container::Container;
use rlvgl::widgets::label::Label;
use rlvgl::widgets::led::Led;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const BG: Color = Color(231, 220, 200, 255);
const CARD: Color = Color(255, 253, 247, 255);
const TEXT: Color = Color(21, 21, 21, 255);
const ACCENT: Color = Color(163, 116, 24, 255);
const MUTED: Color = Color(102, 95, 84, 255);

fn card_style() -> Style {
    let mut s = Style::default();
    s.bg_color = CARD;
    s.border_color = Color(228, 216, 195, 255);
    s.border_width = 2;
    s.radius = 12;
    s
}

/// Owned widget tree for the gen4 demo.
pub struct DemoUi {
    root: WidgetNode,
    bar: Rc<RefCell<Bar>>,
    led: Rc<RefCell<Led>>,
    touch_down: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DirtyWidgets {
    None,
    Dynamic,
    Touch,
}

impl DemoUi {
    pub fn new() -> Self {
        let screen = Rect {
            x: 0,
            y: 0,
            width: DISPLAY_WIDTH as i32,
            height: DISPLAY_HEIGHT as i32,
        };

        let mut shell = Container::new(Rect {
            x: 16,
            y: 12,
            width: 768,
            height: 456,
        });
        shell.style = card_style();

        let mut title = Label::new(
            "gen4-RP2350-70CT",
            Rect {
                x: 32,
                y: 28,
                width: 400,
                height: 32,
            },
        );
        title.set_text_color(TEXT);

        let mut subtitle = Label::new(
            "rlvgl widget demo — PIO RGB + FT5446",
            Rect {
                x: 32,
                y: 58,
                width: 520,
                height: 24,
            },
        );
        subtitle.set_text_color(MUTED);

        let bar = Rc::new(RefCell::new(Bar::new(
            Rect {
                x: 32,
                y: 110,
                width: 736,
                height: 28,
            },
            0,
            100,
        )));
        {
            let mut b = bar.borrow_mut();
            b.set_value(35);
            b.set_mode(BarMode::Normal);
            b.indicator_color = ACCENT;
            b.style.bg_color = Color(252, 247, 236, 255);
        }

        let status = Rc::new(RefCell::new(Label::new(
            "Tap the button",
            Rect {
                x: 32,
                y: 160,
                width: 400,
                height: 24,
            },
        )));
        status.borrow_mut().set_text_color(TEXT);

        let led = Rc::new(RefCell::new(Led::new(Rect {
            x: 720,
            y: 150,
            width: 32,
            height: 32,
        })));
        led.borrow_mut().set_brightness(80);

        let button = Rc::new(RefCell::new(Button::new(
            "Increment",
            Rect {
                x: 32,
                y: 210,
                width: 180,
                height: 56,
            },
        )));
        {
            let mut btn = button.borrow_mut();
            btn.style_mut().bg_color = Color(252, 247, 236, 255);
            btn.style_mut().border_color = Color(197, 186, 168, 255);
            btn.style_mut().border_width = 2;
            btn.style_mut().radius = 10;

            let bar_clone = bar.clone();
            let status_clone = status.clone();
            let led_clone = led.clone();
            btn.set_on_click(move |_| {
                let mut b = bar_clone.borrow_mut();
                let next = (b.value() + 10).min(100);
                b.set_value(next);
                status_clone.borrow_mut().set_text(alloc::format!("Bar value: {next}"));
                led_clone.borrow_mut().set_brightness((next * 2).min(255) as u8);
                info!("button click -> bar {}", next);
            });
        }

        let mut hint = Label::new(
            "Template: ~/work/pio/gen4_rp2350_lvgl",
            Rect {
                x: 32,
                y: 400,
                width: 700,
                height: 24,
            },
        );
        hint.set_text_color(MUTED);

        let root = WidgetNode {
            widget: Rc::new(RefCell::new(ScreenBackground::new(screen, BG))),
            children: alloc::vec![
                WidgetNode::new(Rc::new(RefCell::new(shell))),
                WidgetNode::new(Rc::new(RefCell::new(title))),
                WidgetNode::new(Rc::new(RefCell::new(subtitle))),
                WidgetNode {
                    widget: bar.clone(),
                    children: alloc::vec![],
                    tag: Some("bar"),
                },
                WidgetNode {
                    widget: status.clone(),
                    children: alloc::vec![],
                    tag: Some("status"),
                },
                WidgetNode {
                    widget: led.clone(),
                    children: alloc::vec![],
                    tag: Some("led"),
                },
                WidgetNode {
                    widget: button,
                    children: alloc::vec![],
                    tag: Some("button"),
                },
                WidgetNode::new(Rc::new(RefCell::new(hint))),
            ],
            tag: Some("root"),
        };

        Self {
            root,
            bar,
            led,
            touch_down: false,
        }
    }

    pub fn root(&self) -> &WidgetNode {
        &self.root
    }

    /// Partially re-render only the widgets that change every animation tick
    /// (the progress bar and the LED) into the single live framebuffer.
    ///
    /// The static background, card, labels and button already reside in the
    /// persistent framebuffer, so we avoid re-writing the whole 800×480 frame
    /// each tick — only a few small widget bounds are touched.
    pub fn render_dynamic(&self, fb: *mut u16) {
        self.render_dirty(fb, DirtyWidgets::Dynamic);
    }

    pub fn render_dirty(&self, fb: *mut u16, dirty: DirtyWidgets) {
        for child in &self.root.children {
            let redraw = match dirty {
                DirtyWidgets::None => false,
                DirtyWidgets::Dynamic => matches!(child.tag, Some("bar") | Some("led")),
                DirtyWidgets::Touch => matches!(child.tag, Some("bar") | Some("status") | Some("led") | Some("button")),
            };
            if redraw {
                crate::rlvgl::render::render_node(fb, child);
            }
        }
    }

    pub fn root_mut(&mut self) -> &mut WidgetNode {
        &mut self.root
    }

    pub fn handle_touch(&mut self, x: i32, y: i32, pressed: bool) -> DirtyWidgets {
        if pressed && !self.touch_down {
            self.touch_down = true;
            self.root.dispatch_event(&Event::PressDown { x, y });
            DirtyWidgets::Touch
        } else if !pressed && self.touch_down {
            self.touch_down = false;
            self.root.dispatch_event(&Event::PressRelease { x, y });
            DirtyWidgets::Touch
        } else if pressed {
            self.root.dispatch_event(&Event::PointerMove { x, y });
            DirtyWidgets::None
        } else {
            DirtyWidgets::None
        }
    }

    pub fn tick_bar(&mut self) {
        let v = self.bar.borrow().value();
        let next = if v >= 100 { 0 } else { v + 1 };
        self.bar.borrow_mut().set_value(next);
        self.led.borrow_mut().set_brightness((next * 2).min(255) as u8);
    }
}

struct ScreenBackground {
    bounds: Rect,
    color: Color,
}

impl ScreenBackground {
    fn new(bounds: Rect, color: Color) -> Self {
        Self { bounds, color }
    }
}

impl Widget for ScreenBackground {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn draw(&self, renderer: &mut dyn rlvgl::core::renderer::Renderer) {
        renderer.fill_rect(self.bounds, self.color);
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }
}
