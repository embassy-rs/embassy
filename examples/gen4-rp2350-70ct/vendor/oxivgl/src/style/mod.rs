// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style system: builders, selectors, themes, gradients, and color palettes.

mod grad;
mod palette;
mod selector;
mod style;
mod theme;

pub use grad::{GradDsc, GradExtend};
pub use palette::{
    GradDir, Palette, color_black, color_brightness, color_darken, color_hsv, color_make, color_mix, color_white,
    palette_darken, palette_lighten, palette_main,
};
pub use selector::Selector;
pub use style::{
    BorderSide, ColorFilter, LV_SIZE_CONTENT, Style, StyleBuilder, TextDecor, TransitionDsc, darken_filter_cb, lv_pct,
    props,
};
pub use theme::Theme;
