//! LVGL integration for Riverdi RVT50 (Embassy + [lv_binding_rust](https://github.com/lvgl/lv_binding_rust)).
//!
//! Display and touch drivers follow patterns from
//! [riverdi-50-stm32u5-lvgl](https://github.com/riverdi/riverdi-50-stm32u5-lvgl) and `lvgl-port/port.c`.

pub mod display;
pub mod hall_ui;
pub mod input;
pub mod theme;

pub use display::Rvt50Display;
pub use hall_ui::HallUi;
pub use input::Rvt50Touch;

use core::time::Duration;

/// Advance LVGL timers and run the handler (call every ~5 ms).
pub fn tick_and_run(ms: u64) {
    lvgl::tick_inc(Duration::from_millis(ms));
    lvgl::task_handler();
}
