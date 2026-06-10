// SPDX-License-Identifier: MIT OR Apache-2.0
use oxivgl_sys::*;

/// LVGL material design color palette (`lv_palette_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Palette {
    /// Material Red.
    Red = 0,
    /// Material Pink.
    Pink = 1,
    /// Material Purple.
    Purple = 2,
    /// Material Deep Purple.
    DeepPurple = 3,
    /// Material Indigo.
    Indigo = 4,
    /// Material Blue.
    Blue = 5,
    /// Material Light Blue.
    LightBlue = 6,
    /// Material Cyan.
    Cyan = 7,
    /// Material Teal.
    Teal = 8,
    /// Material Green.
    Green = 9,
    /// Material Light Green.
    LightGreen = 10,
    /// Material Lime.
    Lime = 11,
    /// Material Yellow.
    Yellow = 12,
    /// Material Amber.
    Amber = 13,
    /// Material Orange.
    Orange = 14,
    /// Material Deep Orange.
    DeepOrange = 15,
    /// Material Brown.
    Brown = 16,
    /// Material Blue Grey.
    BlueGrey = 17,
    /// Material Grey.
    Grey = 18,
}

/// Gradient direction (`lv_grad_dir_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GradDir {
    /// No gradient.
    None = 0,
    /// Vertical (top to bottom).
    Ver = 1,
    /// Horizontal (left to right).
    Hor = 2,
    /// Linear gradient with custom angle.
    Linear = 3,
    /// Radial gradient from center.
    Radial = 4,
    /// Conical (sweep) gradient.
    Conical = 5,
}

/// Returns the main (500-shade) color for a palette entry as a raw
/// `lv_color_t`.
pub fn palette_main(p: Palette) -> lv_color_t {
    // SAFETY: lv_palette_main is a pure lookup function.
    unsafe { lv_palette_main(p as lv_palette_t) }
}

/// Returns a lightened shade of a palette color.
/// `level` is 1–5 (1 = lightest, 5 = darkest light variant).
pub fn palette_lighten(p: Palette, level: u8) -> lv_color_t {
    // SAFETY: pure lookup.
    unsafe { lv_palette_lighten(p as lv_palette_t, level) }
}

/// Returns a darkened shade of a palette color.
/// `level` is 1–4 (1 = lightest dark variant, 4 = darkest).
pub fn palette_darken(p: Palette, level: u8) -> lv_color_t {
    // SAFETY: pure lookup.
    unsafe { lv_palette_darken(p as lv_palette_t, level) }
}

/// Returns pure white (`#FFFFFF`).
pub fn color_white() -> lv_color_t {
    // SAFETY: pure function.
    unsafe { lv_color_white() }
}

/// Returns pure black (`#000000`).
pub fn color_black() -> lv_color_t {
    // SAFETY: pure function.
    unsafe { lv_color_black() }
}

/// Returns an `lv_color_t` from RGB components (0–255 each).
pub fn color_make(r: u8, g: u8, b: u8) -> lv_color_t {
    unsafe { lv_color_make(r, g, b) }
}

/// Mix two colors. `mix` = 0 → full `c2`, `mix` = 255 → full `c1`.
pub fn color_mix(c1: lv_color_t, c2: lv_color_t, mix: u8) -> lv_color_t {
    // SAFETY: pure function operating on value types.
    unsafe { lv_color_mix(c1, c2, mix) }
}

/// Returns the perceived brightness of a color (0–255).
pub fn color_brightness(c: lv_color_t) -> u8 {
    // SAFETY: pure function operating on a value type.
    unsafe { lv_color_brightness(c) }
}

/// Darken a color by `lvl` (0–255, where 255 = fully dark).
pub fn color_darken(c: lv_color_t, lvl: u8) -> lv_color_t {
    // SAFETY: pure function operating on value types.
    unsafe { lv_color_darken(c, lvl) }
}

/// Convert HSV to an LVGL color value. Re-exported from [`crate::prelude`].
///
/// Often paired with [`crate::math::trigo_sin`] / [`crate::math::trigo_cos`]
/// for per-frame color cycling in canvas animations.
///
/// - `h`: hue, 0–360
/// - `s`: saturation, 0–100
/// - `v`: value (brightness), 0–100
pub fn color_hsv(h: u16, s: u8, v: u8) -> lv_color_t {
    // SAFETY: pure computation, no LVGL object state.
    unsafe { lv_color_hsv_to_rgb(h, s, v) }
}

#[cfg(test)]
mod tests {
    use super::{GradDir, Palette};

    #[test]
    fn palette_discriminants() {
        assert_eq!(Palette::Red as u32, 0);
        assert_eq!(Palette::Grey as u32, 18);
    }

    #[test]
    fn grad_dir_discriminants() {
        assert_eq!(GradDir::None as u32, 0);
        assert_eq!(GradDir::Ver as u32, 1);
        assert_eq!(GradDir::Hor as u32, 2);
        assert_eq!(GradDir::Linear as u32, 3);
        assert_eq!(GradDir::Radial as u32, 4);
        assert_eq!(GradDir::Conical as u32, 5);
    }
}
