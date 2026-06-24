// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use super::AsLvHandle;

/// Non-owning wrapper for a widget whose LVGL object is owned by a parent
/// container.
///
/// LVGL automatically deletes children when their parent is deleted.
///
/// Dropping a `Child<W>` is always safe and does nothing — LVGL handles
/// cleanup via the parent tree.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Arc, Child, Label, Obj, Screen};
///
/// struct Gauge<'p> {
///     container: Child<Obj<'p>>,
///     arc:       Child<Arc<'p>>,
///     label:     Child<Label<'p>>,
/// }
/// ```
#[repr(transparent)]
pub struct Child<W>(ManuallyDrop<W>);

impl<W> Child<W> {
    /// Wrap `widget` as a non-owning child.
    ///
    /// The caller must ensure the widget's LVGL parent outlives this `Child`.
    /// LVGL will delete the object when the parent is deleted; Rust's `Drop` is
    /// suppressed.
    pub fn new(widget: W) -> Self {
        Child(ManuallyDrop::new(widget))
    }
}

impl<W> Deref for Child<W> {
    type Target = W;
    fn deref(&self) -> &W {
        &self.0
    }
}

impl<W> DerefMut for Child<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.0
    }
}

impl<W: Default> Default for Child<W> {
    fn default() -> Self {
        Child(ManuallyDrop::new(W::default()))
    }
}

impl<W: core::fmt::Debug> core::fmt::Debug for Child<W> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&*self.0, f)
    }
}

impl<W: AsLvHandle> AsLvHandle for Child<W> {
    fn lv_handle(&self) -> *mut oxivgl_sys::lv_obj_t {
        self.0.lv_handle()
    }
}
