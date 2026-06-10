//! Button with visible press feedback for capacitive touch demos.

extern crate alloc;

use alloc::{boxed::Box, string::String};

use rlvgl::core::draw::draw_widget_bg;
use rlvgl::core::event::Event;
use rlvgl::core::renderer::Renderer;
use rlvgl::core::style::Style;
use rlvgl::core::widget::{Color, Rect, Widget};

type ClickHandler = Box<dyn FnMut(&mut PressableButton)>;

/// Clickable button that darkens while the contact point is inside its bounds.
pub struct PressableButton {
    bounds: Rect,
    text: String,
    style: Style,
    text_color: Color,
    pressed_bg: Color,
    pressed_border: Color,
    pressed: bool,
    on_click: Option<ClickHandler>,
}

impl PressableButton {
    pub fn new(text: impl Into<String>, bounds: Rect) -> Self {
        Self {
            bounds,
            text: text.into(),
            style: Style::default(),
            text_color: Color(0, 0, 0, 255),
            pressed_bg: Color(180, 190, 210, 255),
            pressed_border: Color(20, 64, 160, 255),
            pressed: false,
            on_click: None,
        }
    }

    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_pressed_colors(&mut self, bg: Color, border: Color) {
        self.pressed_bg = bg;
        self.pressed_border = border;
    }

    pub fn set_on_click<F: FnMut(&mut Self) + 'static>(&mut self, handler: F) {
        self.on_click = Some(Box::new(handler));
    }

    fn inside_bounds(&self, x: i32, y: i32) -> bool {
        let b = self.bounds;
        x >= b.x && x < b.x + b.width && y >= b.y && y < b.y + b.height
    }

    fn update_pressed(&mut self, x: i32, y: i32) -> bool {
        let inside = self.inside_bounds(x, y);
        self.pressed = inside;
        inside
    }

    fn draw_style(&self) -> Style {
        let mut style = self.style;
        if self.pressed {
            style.bg_color = self.pressed_bg;
            style.border_color = self.pressed_border;
        }
        style
    }
}

impl Widget for PressableButton {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        let style = self.draw_style();
        draw_widget_bg(renderer, self.bounds, &style);
        renderer.draw_text(
            (self.bounds.x, self.bounds.y + self.bounds.height),
            &self.text,
            self.text_color.with_alpha(style.alpha),
        );
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::PressDown { x, y } => {
                self.update_pressed(*x, *y)
            }
            Event::PointerMove { x, y } => {
                self.update_pressed(*x, *y)
            }
            Event::PressRelease { x, y } => {
                let inside = self.inside_bounds(*x, *y);
                self.pressed = false;
                if inside {
                    if let Some(mut cb) = self.on_click.take() {
                        cb(self);
                        self.on_click = Some(cb);
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}
