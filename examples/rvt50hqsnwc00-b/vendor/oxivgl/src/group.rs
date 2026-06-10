// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL input group — focus management for keyboard/encoder navigation.

use core::marker::PhantomData;

use oxivgl_sys::*;

use crate::widgets::{AsLvHandle, WidgetError};

/// Owning wrapper around an `lv_group_t`. Calls `lv_group_delete` on drop.
///
/// A group collects focusable widgets. When assigned to a keyboard or encoder
/// input device, arrow/tab keys move focus between the group members.
///
/// # Thread safety
///
/// `Group` is `!Send + !Sync` — LVGL must be driven from a single task.
pub struct Group {
    ptr: *mut lv_group_t,
    _not_send: PhantomData<*const ()>,
}

impl core::fmt::Debug for Group {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Group").finish_non_exhaustive()
    }
}

impl Group {
    /// Create a new LVGL group.
    ///
    /// Returns `Err(WidgetError::LvglNullPointer)` if LVGL allocation fails.
    pub fn new() -> Result<Self, WidgetError> {
        // SAFETY: lv_group_create allocates and initialises an lv_group_t.
        // Returns NULL on OOM; checked below.
        // lv_group_init() is called automatically by lv_init().
        // See lvgl/src/indev/lv_indev.c — lv_group_create.
        let ptr = unsafe { lv_group_create() };
        if ptr.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        Ok(Self { ptr, _not_send: PhantomData })
    }

    /// Set this group as the default group.
    ///
    /// Indevs created without an explicit group assignment will use the default.
    pub fn set_default(&self) -> &Self {
        // SAFETY: self.ptr is non-null (checked in new()).
        // lv_group_set_default stores the pointer in a global.
        // See lvgl/src/indev/lv_indev.c — lv_group_set_default.
        unsafe { lv_group_set_default(self.ptr) };
        self
    }

    /// Add a widget to this group.
    ///
    /// After adding, arrow/tab keys will be able to focus `obj`.
    pub fn add_obj(&self, obj: &impl AsLvHandle) -> &Self {
        // SAFETY: self.ptr and obj.lv_handle() are both non-null.
        // lv_group_add_obj links the object into the group's linked list.
        // See lvgl/src/indev/lv_indev.c — lv_group_add_obj.
        unsafe { lv_group_add_obj(self.ptr, obj.lv_handle()) };
        self
    }

    /// Move focus to a specific object within this group.
    ///
    /// The object must already be a member of this group.
    /// See lvgl/src/indev/lv_indev.c — lv_group_focus_obj.
    pub fn focus_obj(&self, obj: &impl AsLvHandle) -> &Self {
        // SAFETY: obj.lv_handle() is non-null. lv_group_focus_obj moves the
        // focus cursor to the given object inside its group; no ownership is
        // transferred.
        // See lvgl/src/indev/lv_indev.c — lv_group_focus_obj.
        unsafe { lv_group_focus_obj(obj.lv_handle()) };
        self
    }

    /// Move focus to the next object in this group.
    ///
    /// Wraps around to the first object when the end is reached.
    /// See lvgl/src/indev/lv_indev.c — lv_group_focus_next.
    pub fn focus_next(&self) -> &Self {
        // SAFETY: self.ptr is non-null (checked in new()).
        // lv_group_focus_next advances the focus cursor in the group.
        // See lvgl/src/indev/lv_indev.c — lv_group_focus_next.
        unsafe { lv_group_focus_next(self.ptr) };
        self
    }

    /// Assign this group to all keyboard and encoder input devices.
    ///
    /// Iterates all registered indevs with `lv_indev_get_next` and calls
    /// `lv_indev_set_group` on those whose type is `KEYPAD` or `ENCODER`.
    /// See lvgl/src/indev/lv_indev.c — lv_indev_get_next, lv_indev_set_group.
    pub fn assign_to_keyboard_indevs(&self) -> &Self {
        // SAFETY: lv_indev_get_next(NULL) returns the first registered indev.
        // Iterating with the returned pointer is safe as long as LVGL is
        // initialised (guaranteed by the existence of any widget or driver).
        // lv_indev_get_type and lv_indev_set_group are safe to call on any
        // non-null lv_indev_t pointer.
        unsafe {
            let mut indev = lv_indev_get_next(core::ptr::null_mut());
            while !indev.is_null() {
                let kind = lv_indev_get_type(indev);
                if kind == lv_indev_type_t_LV_INDEV_TYPE_KEYPAD
                    || kind == lv_indev_type_t_LV_INDEV_TYPE_ENCODER
                {
                    lv_indev_set_group(indev, self.ptr);
                }
                indev = lv_indev_get_next(indev);
            }
        }
        self
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        // SAFETY: self.ptr was returned by lv_group_create and is non-null.
        // lv_group_delete frees all internal linked-list nodes and the group
        // itself. Objects previously added are not freed — they are owned by
        // the widget wrappers.
        // See lvgl/src/indev/lv_indev.c — lv_group_delete.
        unsafe { lv_group_delete(self.ptr) };
    }
}

/// Non-owning handle to an LVGL group (no `Drop`).
///
/// Useful when borrowing the default group without taking ownership.
/// Obtain via [`group_get_default`].
pub struct GroupRef {
    ptr: *mut lv_group_t,
    _not_send: PhantomData<*const ()>,
}

impl GroupRef {
    /// Add a widget to this group.
    pub fn add_obj(&self, obj: &impl AsLvHandle) -> &Self {
        // SAFETY: self.ptr is non-null (checked in group_get_default()).
        // See lvgl/src/indev/lv_indev.c — lv_group_add_obj.
        unsafe { lv_group_add_obj(self.ptr, obj.lv_handle()) };
        self
    }

    /// Move focus to a specific object within this group.
    ///
    /// The object must already be a member of this group.
    /// See lvgl/src/indev/lv_indev.c — lv_group_focus_obj.
    pub fn focus_obj(&self, obj: &impl AsLvHandle) -> &Self {
        // SAFETY: obj.lv_handle() is non-null. lv_group_focus_obj moves the
        // focus cursor to the given object inside its group.
        // See lvgl/src/indev/lv_indev.c — lv_group_focus_obj.
        unsafe { lv_group_focus_obj(obj.lv_handle()) };
        self
    }

    /// Move focus to the next object in this group.
    ///
    /// Wraps around to the first object when the end is reached.
    /// See lvgl/src/indev/lv_indev.c — lv_group_focus_next.
    pub fn focus_next(&self) -> &Self {
        // SAFETY: self.ptr is non-null (checked in group_get_default()).
        // lv_group_focus_next advances the focus cursor in the group.
        // See lvgl/src/indev/lv_indev.c — lv_group_focus_next.
        unsafe { lv_group_focus_next(self.ptr) };
        self
    }
}

/// Get a non-owning handle to the current default group.
///
/// Returns `None` if no default group has been set.
pub fn group_get_default() -> Option<GroupRef> {
    // SAFETY: lv_group_get_default returns the globally stored default pointer
    // or NULL. No side effects.
    // See lvgl/src/indev/lv_indev.c — lv_group_get_default.
    let ptr = unsafe { lv_group_get_default() };
    if ptr.is_null() {
        None
    } else {
        Some(GroupRef { ptr, _not_send: PhantomData })
    }
}

/// Remove a widget from its group.
///
/// After this call the widget will no longer receive keyboard/encoder focus
/// through the group mechanism. The widget itself is not deleted.
/// See lvgl/src/indev/lv_indev.c — lv_group_remove_obj.
pub fn group_remove_obj(obj: &impl AsLvHandle) {
    // SAFETY: obj.lv_handle() is non-null. lv_group_remove_obj unlinks the
    // object from whatever group it belongs to, or is a no-op if not in any.
    unsafe { lv_group_remove_obj(obj.lv_handle()) };
}
