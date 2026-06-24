// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-safe style selector combining [`Part`](crate::widgets::Part) and
//! [`ObjState`](crate::enums::ObjState) bits.
//!
//! Replaces raw `u32` in all style-related methods.
//!
//! ```ignore
//! use oxivgl::style::Selector;
//! use oxivgl::enums::ObjState;
//! use oxivgl::widgets::Part;
//!
//! btn.add_style(&style, Selector::DEFAULT);
//! btn.add_style(&style, ObjState::PRESSED);
//! slider.add_style(&style, Part::Indicator | ObjState::PRESSED);
//! ```

/// Style selector = [`Part`](crate::widgets::Part) +
/// [`ObjState`](crate::enums::ObjState) bits. Pass to methods like
/// [`Obj::add_style`](crate::widgets::Obj::add_style),
/// [`Obj::radius`](crate::widgets::Obj::radius), etc.
#[derive(Clone, Copy, Debug, Default)]
pub struct Selector(u32);

impl Selector {
    /// Default selector (main part, default state).
    pub const DEFAULT: Self = Self(0);

    /// Raw `u32` value for passing to LVGL FFI.
    pub fn raw(self) -> u32 {
        self.0
    }
}

impl From<crate::widgets::Part> for Selector {
    fn from(p: crate::widgets::Part) -> Self {
        Self(p as u32)
    }
}

impl From<crate::enums::ObjState> for Selector {
    fn from(s: crate::enums::ObjState) -> Self {
        Self(s.0)
    }
}

impl core::ops::BitOr<crate::enums::ObjState> for crate::widgets::Part {
    type Output = Selector;
    fn bitor(self, rhs: crate::enums::ObjState) -> Selector {
        Selector(self as u32 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{enums::ObjState, widgets::Part};

    #[test]
    fn default_is_zero() {
        assert_eq!(Selector::DEFAULT.raw(), 0);
        assert_eq!(Selector::default().raw(), 0);
    }

    #[test]
    fn from_part() {
        let s: Selector = Part::Indicator.into();
        assert_eq!(s.raw(), Part::Indicator as u32);
    }

    #[test]
    fn from_state() {
        let s: Selector = ObjState::PRESSED.into();
        assert_eq!(s.raw(), ObjState::PRESSED.0);
    }

    #[test]
    fn part_bitor_state() {
        let s = Part::Indicator | ObjState::PRESSED;
        assert_eq!(s.raw(), Part::Indicator as u32 | ObjState::PRESSED.0);
    }
}
