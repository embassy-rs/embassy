//! OxivGL (C LVGL v9.5) integration for the gen4-RP2350-70CT PIO RGB panel.
//!
//! Real LVGL compiled via [`oxivgl-sys`] (with `conf/lv_conf.h`), driven into
//! the single persistent PSRAM scan-out framebuffer of [`crate::pio_rgb`].
//!
//! This is the OxivGL counterpart to the hand-rolled [`crate::rlvgl`] demo and
//! reuses the same board support: FT5446 capacitive touch
//! ([`crate::touch_feed`]) feeds the LVGL pointer indev, and the PIO/DMA
//! scan-out streams the framebuffer that LVGL flushes into.

pub mod display;
pub mod fonts;
pub mod indev;
pub mod platform;
pub mod widget_view;
