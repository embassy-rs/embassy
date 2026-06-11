//! OxivGL (C LVGL v9.5) integration for the Riverdi RVT50 LTDC panel.
//!
//! Uses LVGL sources compatible with Riverdi's
//! `riverdi-50-stm32u5-lvgl/Middlewares/Third_Party/LVGL` Cube port, compiled
//! via [`oxivgl-sys`] with `conf/lv_conf.h`.

pub mod display;
pub mod indev;
pub mod platform;
pub mod touch_dbg;
#[cfg(feature = "touch")]
pub mod touch_feed;
pub mod widget_view;
