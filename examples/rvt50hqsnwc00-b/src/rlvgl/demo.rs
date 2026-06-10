//! Minimal rlvgl UI tree for the Riverdi RVT50 demo.

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use rlvgl::core::{WidgetNode, widget::Color, widget::Rect};
use rlvgl::widgets::{container::Container, label::Label};

use crate::rlvgl::pressable_button::PressableButton;

/// Build a small widget tree: title label plus a clickable counter button.
pub fn build_demo(width: i32, height: i32) -> Rc<RefCell<WidgetNode>> {
    let clicks = Rc::new(RefCell::new(0u32));

    let mut root_container = Container::new(Rect {
        x: 0,
        y: 0,
        width,
        height,
    });
    root_container.style.bg_color = Color(16, 28, 48, 255);

    let mut title = Label::new(
        "rlvgl on Riverdi RVT50",
        Rect {
            x: 40,
            y: 40,
            width: 420,
            height: 32,
        },
    );
    title.text_color = Color(220, 230, 255, 255);
    title.style.bg_color = Color(0, 0, 0, 0);

    let button = Rc::new(RefCell::new(PressableButton::new(
        "Tap me",
        Rect {
            x: 40,
            y: 120,
            width: 180,
            height: 56,
        },
    )));

    {
        let mut btn = button.borrow_mut();
        btn.style_mut().bg_color = Color(230, 236, 248, 255);
        btn.style_mut().border_color = Color(40, 96, 200, 255);
        btn.style_mut().border_width = 2;
        btn.style_mut().radius = 8;
        btn.set_pressed_colors(Color(170, 190, 230, 255), Color(20, 64, 160, 255));

        let clicks = clicks.clone();
        btn.set_on_click(move |b| {
            let n = {
                let mut c = clicks.borrow_mut();
                *c += 1;
                *c
            };
            b.set_text(alloc::format!("Tapped {n}"));
        });
    }

    Rc::new(RefCell::new(WidgetNode {
        widget: Rc::new(RefCell::new(root_container)),
        children: alloc::vec![
            WidgetNode {
                widget: Rc::new(RefCell::new(title)),
                children: alloc::vec![],
                tag: None,
            },
            WidgetNode {
                widget: button,
                children: alloc::vec![],
                tag: Some("demo-button"),
            },
        ],
        tag: Some("demo-root"),
    }))
}
