// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::boxed::Box;
use core::{marker::PhantomData, pin::Pin, ptr::null_mut};

use oxivgl_sys::*;

use super::Style;
use crate::widgets::WidgetError;

unsafe extern "C" {
    /// LVGL button widget class descriptor. Not emitted by bindgen (extern
    /// data); declared here for use in the theme apply callback.
    static lv_button_class: lv_obj_class_t;
}

unsafe extern "C" fn apply_cb_trampoline(th: *mut lv_theme_t, obj: *mut lv_obj_t) {
    // SAFETY: `th` is the pinned lv_theme_t we installed; user_data points to
    // the lv_style_t inside the Rc<StyleInner> that Theme keeps alive via its
    // Style clone.
    unsafe {
        if lv_obj_check_type(obj, &lv_button_class) {
            let style_ptr = (*th).user_data as *const lv_style_t;
            lv_obj_add_style(obj, style_ptr, 0);
        }
    }
}

/// Owned LVGL theme extension.
///
/// Extends the active display theme so that every
/// [`Button`](crate::widgets::Button) created after [`Theme::extend_current`]
/// is called receives the supplied [`Style`] automatically.
///
/// # Lifetime contract
/// The `Theme` value **must** be kept alive for as long as any widgets styled
/// by it exist. Dropping it while LVGL holds a pointer to the inner
/// `lv_theme_t` is undefined behaviour. Store it in the `View` struct.
///
/// # Single-display assumption
/// Passes `NULL` to `lv_display_get_theme` / `lv_display_set_theme`, which
/// selects the default (first) display.
pub struct Theme {
    /// Heap-pinned so the address passed to `lv_display_set_theme` stays
    /// stable.
    inner: Pin<Box<lv_theme_t>>,
    /// Keeps the `lv_style_t` pointed to by `inner.user_data` alive.
    _style: Style,
    _not_send: PhantomData<*mut ()>,
}

impl core::fmt::Debug for Theme {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Theme").finish_non_exhaustive()
    }
}

impl Theme {
    /// Extend the active display theme: buttons created after this call will
    /// have `style` applied by the theme machinery.
    ///
    /// Returns [`WidgetError::LvglNullPointer`] when no display / theme is
    /// active yet.
    pub fn extend_current(style: Style) -> Result<Self, WidgetError> {
        // SAFETY: lv_display_get_theme(NULL) selects the default display.
        let th_act = unsafe { lv_display_get_theme(null_mut()) };
        if th_act.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }

        // Copy the active theme, then pin it on the heap so its address is stable.
        // SAFETY: th_act is non-null and points to a valid lv_theme_t.
        let th_new: lv_theme_t = unsafe { *th_act };
        let mut inner = Box::into_pin(Box::new(th_new));

        // SAFETY: lv_theme_t is Unpin (plain C struct); get_unchecked_mut is safe here.
        // style.lv_ptr() is valid for the Rc's lifetime; Theme stores a clone.
        unsafe {
            let p = inner.as_mut().get_unchecked_mut();
            p.user_data = style.lv_ptr() as *mut core::ffi::c_void;
            lv_theme_set_parent(p, th_act);
            lv_theme_set_apply_cb(p, Some(apply_cb_trampoline));
            lv_display_set_theme(null_mut(), p);
        }

        Ok(Theme { inner, _style: style, _not_send: PhantomData })
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        // Restore the parent theme so LVGL doesn't hold a dangling pointer.
        // SAFETY: inner is pinned and still valid at drop time.
        unsafe {
            let p = self.inner.as_mut().get_unchecked_mut();
            lv_display_set_theme(null_mut(), p.parent);
        }
    }
}
