//! OxivGL (C LVGL v9.5) integration for the Riverdi RVT50 LTDC panel.
//!
//! Uses LVGL sources compatible with Riverdi's
//! `riverdi-50-stm32u5-lvgl/Middlewares/Third_Party/LVGL` Cube port, compiled
//! via [`oxivgl-sys`] with `conf/lv_conf.h`.
//!
//! Capacitive touch is always enabled: `CTP_INT` EXTI wakes
//! [`touch_feed::run_touch_int_task`], which samples I2C and queues events for
//! the UI task.

pub mod display;
pub mod indev;
pub mod platform;
pub mod touch_dbg;
pub mod touch_feed;
pub mod widget_view;
