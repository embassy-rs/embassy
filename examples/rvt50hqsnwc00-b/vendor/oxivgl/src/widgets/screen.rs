// SPDX-License-Identifier: MIT OR Apache-2.0
//! [`Screen`] — non-owning reference to the active LVGL screen.

use alloc::vec::Vec;
use core::cell::RefCell;

use oxivgl_sys::*;

use super::obj::AsLvHandle;
use crate::{
    layout::{FlexAlign, FlexFlow},
    style::{Selector, Style},
};

/// Non-owning reference to the active LVGL screen. Does **not** delete it on
/// drop.
///
/// Obtain via [`Screen::active()`]. Use as a parent for top-level widgets.
///
/// # Style lifetime
///
/// Styles added via [`add_style`](Screen::add_style) are intentionally
/// **leaked** when this `Screen` is dropped. The LVGL screen object outlives
/// any Rust handle to it, so styles must remain valid indefinitely. Each
/// `add_style` call costs one `Rc` bump that is never reclaimed — this is a
/// bounded leak proportional to the number of styles added.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::Screen;
///
/// let screen = Screen::active().expect("LVGL not initialized");
/// screen.bg_color(0x06080f).bg_opa(255).pad_top(6).pad_bottom(6);
/// ```
pub struct Screen {
    handle: *mut lv_obj_t,
    /// Rc clones of styles added via `add_style`. Keeps the `lv_style_t`
    /// alive as long as this Screen reference exists (spec §5.1).
    _styles: RefCell<Vec<Style>>,
}

impl core::fmt::Debug for Screen {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Screen").finish_non_exhaustive()
    }
}

impl AsLvHandle for Screen {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.handle
    }
}

impl Screen {
    /// Returns `None` if LVGL has no active screen yet.
    pub fn active() -> Option<Self> {
        // SAFETY: lv_screen_active() is safe after lv_init(); NULL result is handled
        // below.
        let handle = unsafe { lv_screen_active() };
        if handle.is_null() { None } else { Some(Screen { handle, _styles: RefCell::new(Vec::new()) }) }
    }

    /// Get the top layer — a global overlay above all screens.
    ///
    /// Returns a non-owning handle. The top layer is owned by LVGL and
    /// must never be deleted.
    ///
    /// **Warning:** Styles added to the returned `Child` handle will **not**
    /// be freed, because `Child` suppresses `Drop`. If you add styles to
    /// the top layer, they leak for the process lifetime.
    pub fn layer_top() -> super::child::Child<super::obj::Obj<'static>> {
        // SAFETY: lv_layer_top() returns a valid global object after lv_init().
        let handle = unsafe { lv_layer_top() };
        assert!(!handle.is_null(), "lv_layer_top returned NULL");
        super::child::Child::new(super::obj::Obj::from_raw(handle))
    }

    /// Return the raw `lv_obj_t` pointer for this screen.
    pub fn handle(&self) -> *mut lv_obj_t {
        self.handle
    }

    /// Remove the scrollable flag from the screen.
    pub fn remove_scrollable(&self) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_remove_flag(self.handle, crate::enums::ObjFlag::SCROLLABLE.0) };
        self
    }

    /// Set background color from RGB hex.
    pub fn bg_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    /// Set background opacity (0–255).
    pub fn bg_opa(&self, opa: u8) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_opa(self.handle, opa as lv_opa_t, 0) };
        self
    }

    /// Set top padding.
    pub fn pad_top(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_top(self.handle, p, 0) };
        self
    }

    /// Set bottom padding.
    pub fn pad_bottom(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_bottom(self.handle, p, 0) };
        self
    }

    /// Set left padding.
    pub fn pad_left(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_left(self.handle, p, 0) };
        self
    }

    /// Set right padding.
    pub fn pad_right(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_right(self.handle, p, 0) };
        self
    }

    /// Set default text color from RGB hex.
    pub fn text_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_text_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    /// Set flex layout flow direction.
    pub fn set_flex_flow(&self, flow: FlexFlow) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_flex_flow(self.handle, flow as lv_flex_flow_t) };
        self
    }

    /// Apply a style for the given selector.
    ///
    /// Clones the `Style` Rc to keep the `lv_style_t` alive for the
    /// screen's lifetime (spec §5.1).
    pub fn add_style(&self, style: &Style, selector: impl Into<Selector>) -> &Self {
        self._styles.borrow_mut().push(style.clone());
        let selector = selector.into().raw();
        // SAFETY: handle non-null (Screen::active() returns None for null).
        // Style Rc clone above keeps the lv_style_t valid.
        unsafe { lv_obj_add_style(self.handle, style.lv_ptr(), selector) };
        self
    }

    /// Conditionally bind a style via the observer API: add `style` with
    /// `selector` when `subject == ref_value`, remove it otherwise.
    ///
    /// Same lifetime contract as [`add_style`](Self::add_style) — the style Rc
    /// clone is intentionally leaked on drop because the LVGL screen outlives
    /// this non-owning handle.
    ///
    /// The subject should outlive the screen. Both drop orders are safe
    /// (see [`Subject`](super::subject::Subject) docs).
    pub fn bind_style(
        &self,
        style: &Style,
        selector: impl Into<Selector>,
        subject: &super::subject::Subject,
        ref_value: i32,
    ) -> &Self {
        let selector = selector.into().raw();
        // Intentionally leaked on drop (same as add_style for Screen).
        self._styles.borrow_mut().push(style.clone());
        // SAFETY: handle non-null; style pointer valid for Rc lifetime
        // (repr(C) offset-0 guarantee); subject is pinned.
        unsafe {
            lv_obj_bind_style(self.handle, style.lv_ptr(), selector, subject.as_ptr(), ref_value);
        }
        self
    }

    /// Set flex alignment (main, cross, track).
    pub fn set_flex_align(&self, main: FlexAlign, cross: FlexAlign, track: FlexAlign) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe {
            lv_obj_set_flex_align(
                self.handle,
                main as lv_flex_align_t,
                cross as lv_flex_align_t,
                track as lv_flex_align_t,
            )
        };
        self
    }

    /// Create a new LVGL screen object (a root-level widget with no parent).
    ///
    /// Returns an owned [`Obj`](super::obj::Obj) that the caller manages.
    /// Use this for navigator-managed screens where normal `Obj::drop`
    /// cleanup is desired (no intentional style leaking).
    pub fn create() -> super::obj::Obj<'static> {
        // SAFETY: lv_obj_create(NULL) creates a root-level screen object.
        // Safe after lv_init().
        let handle = unsafe { lv_obj_create(core::ptr::null_mut()) };
        assert!(!handle.is_null(), "lv_obj_create(NULL) returned NULL");
        super::obj::Obj::from_raw(handle)
    }

    /// Get the system layer — a global overlay above the top layer.
    ///
    /// Returns a non-owning handle. The system layer is owned by LVGL and
    /// must never be deleted.
    ///
    /// **Warning:** Styles added to the returned `Child` handle will **not**
    /// be freed, because `Child` suppresses `Drop`. If you add styles to
    /// the system layer, they leak for the process lifetime.
    pub fn layer_sys() -> super::child::Child<super::obj::Obj<'static>> {
        // SAFETY: lv_layer_sys() returns a valid global object after lv_init().
        let handle = unsafe { lv_layer_sys() };
        assert!(!handle.is_null(), "lv_layer_sys returned NULL");
        super::child::Child::new(super::obj::Obj::from_raw(handle))
    }

    /// Load a screen with an animated transition.
    ///
    /// `scr` becomes the new active screen. The previous screen is
    /// optionally deleted after the animation completes (`auto_del`).
    pub fn load(
        scr: &impl AsLvHandle,
        anim: &ScreenAnim,
        auto_del: bool,
    ) {
        // SAFETY: scr handle is non-null (enforced by AsLvHandle contract).
        unsafe {
            lv_screen_load_anim(
                scr.lv_handle(),
                anim.anim_type as lv_screen_load_anim_t,
                anim.duration_ms,
                anim.delay_ms,
                auto_del,
            );
        }
    }

    /// Load a screen instantly with no animation.
    pub fn load_instant(scr: &impl AsLvHandle) {
        // SAFETY: scr handle is non-null (enforced by AsLvHandle contract).
        unsafe { lv_screen_load(scr.lv_handle()) };
    }
}

/// Screen transition animation type.
///
/// Mirrors `lv_screen_load_anim_t`. Used with [`Screen::load`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ScreenAnimType {
    /// No animation — instant switch.
    None = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_NONE,
    /// New screen slides in from the left, covering the old one.
    OverLeft = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OVER_LEFT,
    /// New screen slides in from the right, covering the old one.
    OverRight = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OVER_RIGHT,
    /// New screen slides in from the top, covering the old one.
    OverTop = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OVER_TOP,
    /// New screen slides in from the bottom, covering the old one.
    OverBottom = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OVER_BOTTOM,
    /// Both screens move left together.
    MoveLeft = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_MOVE_LEFT,
    /// Both screens move right together.
    MoveRight = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_MOVE_RIGHT,
    /// Both screens move up together.
    MoveTop = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_MOVE_TOP,
    /// Both screens move down together.
    MoveBottom = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_MOVE_BOTTOM,
    /// New screen fades in over the old one.
    FadeIn = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_FADE_IN,
    /// Old screen fades out, revealing the new one.
    FadeOut = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_FADE_OUT,
    /// Old screen slides out to the left, revealing the new one.
    OutLeft = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OUT_LEFT,
    /// Old screen slides out to the right, revealing the new one.
    OutRight = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OUT_RIGHT,
    /// Old screen slides out upward, revealing the new one.
    OutTop = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OUT_TOP,
    /// Old screen slides out downward, revealing the new one.
    OutBottom = lv_screen_load_anim_t_LV_SCREEN_LOAD_ANIM_OUT_BOTTOM,
}

/// Screen transition animation parameters.
///
/// Used with [`Screen::load`] to control how screen transitions look.
#[derive(Debug, Clone, Copy)]
pub struct ScreenAnim {
    /// Animation type (slide, fade, move, etc.).
    pub anim_type: ScreenAnimType,
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Delay before animation starts, in milliseconds.
    pub delay_ms: u32,
}

impl Drop for Screen {
    fn drop(&mut self) {
        // Intentionally leak style Rc clones. The LVGL screen object outlives
        // this non-owning handle — freeing styles here would leave dangling
        // pointers in LVGL. Bounded leak: one Rc bump per add_style call.
        for style in self._styles.get_mut().drain(..) {
            core::mem::forget(style);
        }
    }
}
