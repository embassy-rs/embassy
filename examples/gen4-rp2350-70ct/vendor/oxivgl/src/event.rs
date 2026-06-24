// SPDX-License-Identifier: MIT OR Apache-2.0
//! Safe wrapper around LVGL events (`lv_event_t`).

use oxivgl_sys::*;

use crate::draw::{DrawTask, Layer};
use crate::enums::{EventCode, Key};
use crate::widgets::{AsLvHandle, Child, Obj};

/// Safe wrapper around an LVGL event (`lv_event_t`).
///
/// Passed to [`View::on_event`](crate::view::View::on_event) and
/// [`Obj::on`] callbacks — valid only for the duration of the callback.
pub struct Event {
    raw: *mut lv_event_t,
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Event").finish_non_exhaustive()
    }
}

impl Event {
    pub(crate) fn from_raw(raw: *mut lv_event_t) -> Self {
        Self { raw }
    }

    /// Event code (e.g. `EventCode::CLICKED`).
    pub fn code(&self) -> EventCode {
        // SAFETY: raw pointer valid for callback duration.
        EventCode(unsafe { lv_event_get_code(self.raw) })
    }

    /// Raw handle of the widget that originally received the event.
    pub fn target_handle(&self) -> *mut lv_obj_t {
        // SAFETY: raw pointer valid for callback duration.
        unsafe { lv_event_get_target_obj(self.raw) }
    }

    /// Non-owning reference to the event target widget.
    /// The returned `Obj` does NOT own the LVGL object — do not store it.
    pub fn target(&self) -> Child<Obj<'_>> {
        Child::new(Obj::from_raw(self.target_handle()))
    }

    /// Raw handle of the widget whose event handler is currently running
    /// (differs from target when events bubble).
    pub fn current_target_handle(&self) -> *mut lv_obj_t {
        // SAFETY: raw pointer valid for callback duration.
        unsafe { lv_event_get_current_target_obj(self.raw) }
    }

    /// Check if this event matches a specific widget and event code.
    ///
    /// ```ignore
    /// fn on_event(&mut self, event: &Event) {
    ///     if event.matches(&self.btn, EventCode::CLICKED) {
    ///         // handle click
    ///     }
    /// }
    /// ```
    pub fn matches(&self, widget: &impl AsLvHandle, code: EventCode) -> bool {
        self.code() == code && self.target_handle() == widget.lv_handle()
    }

    /// Set a style property on the event target. Convenience for event handlers
    /// that need to modify the originating widget (e.g. event bubbling).
    pub fn target_style_bg_color(&self, color: lv_color_t, selector: impl Into<crate::style::Selector>) {
        let selector = selector.into().raw();
        // SAFETY: target_handle() returns a valid LVGL object for callback duration.
        unsafe { lv_obj_set_style_bg_color(self.target_handle(), color, selector) };
    }

    /// Get the draw task associated with a `DRAW_TASK_ADDED` event.
    ///
    /// Returns `None` if the event has no draw task (wrong event type).
    /// The returned handle is valid only for the duration of this callback.
    pub fn draw_task(&self) -> Option<DrawTask> {
        // SAFETY: raw pointer valid for callback duration.
        let ptr = unsafe { lv_event_get_draw_task(self.raw) };
        if ptr.is_null() {
            None
        } else {
            Some(DrawTask::from_raw(ptr))
        }
    }

    /// Key code from a `KEY` event.
    ///
    /// Returns the key code wrapped as [`Key`], or `None` if called on a
    /// non-`KEY` event (LVGL returns 0 which is not a defined key).
    ///
    /// Only meaningful when `event.code() == EventCode::KEY`.
    pub fn key(&self) -> Option<Key> {
        // SAFETY: raw pointer valid for callback duration.
        // lv_event_get_key returns 0 for non-KEY events, which is not a valid
        // lv_key_t value — treat 0 as None.
        // See lvgl/src/indev/lv_indev.c — lv_event_get_key.
        let k = unsafe { lv_event_get_key(self.raw) };
        if k == 0 { None } else { Some(Key(k)) }
    }

    /// Get the draw layer for a draw event (`DRAW_MAIN_END`, `DRAW_TASK_ADDED`, etc.).
    ///
    /// Returns `None` if the event has no associated layer (wrong event type).
    /// The returned handle is valid only for the duration of this callback.
    pub fn layer(&self) -> Option<Layer> {
        // SAFETY: raw pointer valid for callback duration.
        let ptr = unsafe { lv_event_get_layer(self.raw) };
        if ptr.is_null() {
            None
        } else {
            Some(Layer::from_raw(ptr))
        }
    }
}
