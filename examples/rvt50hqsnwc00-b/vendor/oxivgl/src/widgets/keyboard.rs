// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    buttonmatrix::{ButtonmatrixCtrl, ButtonmatrixMap},
    obj::{AsLvHandle, Obj},
};

/// Keyboard input mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum KeyboardMode {
    /// Lowercase letters (default).
    TextLower = 0,
    /// Uppercase letters.
    TextUpper = 1,
    /// Special characters.
    Special = 2,
    /// Numeric keypad.
    Number = 3,
    /// User-defined mode 1.
    User1 = 4,
    /// User-defined mode 2.
    User2 = 5,
    /// User-defined mode 3.
    User3 = 6,
    /// User-defined mode 4.
    User4 = 7,
}

/// LVGL keyboard widget (on-screen virtual keyboard).
///
/// Requires `LV_USE_KEYBOARD = 1` in `lv_conf.h` (which also requires
/// `LV_USE_BUTTONMATRIX` and `LV_USE_TEXTAREA`).
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Keyboard, KeyboardMode, Screen, Textarea};
///
/// let screen = Screen::active().unwrap();
/// let ta = Textarea::new(&screen).unwrap();
/// let kb = Keyboard::new(&screen).unwrap();
/// kb.set_textarea(&ta);
/// kb.set_mode(KeyboardMode::Number);
/// ```
#[derive(Debug)]
pub struct Keyboard<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Keyboard<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Keyboard<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Keyboard<'p> {
    /// Create a new keyboard widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_keyboard_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Keyboard { obj: Obj::from_raw(handle) }) }
    }

    /// Associate the keyboard with a textarea.
    ///
    /// Key presses will be sent to the textarea. Pass a different textarea
    /// to switch focus.
    pub fn set_textarea(&self, ta: &impl AsLvHandle) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Keyboard handle cannot be null");
        // SAFETY: handle non-null; ta.lv_handle() is a valid LVGL textarea object.
        unsafe { lv_keyboard_set_textarea(self.obj.handle(), ta.lv_handle()) };
        self
    }

    /// Set the keyboard mode (letter case, special chars, numbers).
    pub fn set_mode(&self, mode: KeyboardMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Keyboard handle cannot be null");
        // SAFETY: handle non-null; mode is a valid lv_keyboard_mode_t value.
        unsafe { lv_keyboard_set_mode(self.obj.handle(), mode as lv_keyboard_mode_t) };
        self
    }

    /// Get the current keyboard mode.
    pub fn get_mode(&self) -> KeyboardMode {
        // SAFETY: handle non-null (checked in new()).
        let raw = unsafe { lv_keyboard_get_mode(self.lv_handle()) };
        match raw {
            1 => KeyboardMode::TextUpper,
            2 => KeyboardMode::Special,
            3 => KeyboardMode::Number,
            4 => KeyboardMode::User1,
            5 => KeyboardMode::User2,
            6 => KeyboardMode::User3,
            7 => KeyboardMode::User4,
            _ => KeyboardMode::TextLower,
        }
    }

    /// Get whether popovers (key press previews) are enabled.
    pub fn get_popovers(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_keyboard_get_popovers(self.lv_handle()) }
    }

    /// Set a custom key map and control flags for a keyboard mode.
    ///
    /// LVGL stores the raw pointers; both `map` and `ctrl` MUST be `'static`
    /// (spec-memory-lifetime §1/§3).
    pub fn set_map(
        &self,
        mode: KeyboardMode,
        map: &'static ButtonmatrixMap,
        ctrl: &'static [ButtonmatrixCtrl],
    ) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Keyboard handle cannot be null");
        // SAFETY: handle non-null; map and ctrl are 'static. LVGL stores
        // the pointers; 'static satisfies the lifetime requirement.
        // ButtonmatrixCtrl is repr(transparent) over lv_buttonmatrix_ctrl_t.
        unsafe {
            lv_keyboard_set_map(
                self.obj.handle(),
                mode as lv_keyboard_mode_t,
                map.0.as_ptr() as *const *const c_char,
                ctrl.as_ptr() as *const lv_buttonmatrix_ctrl_t,
            )
        };
        self
    }
}
