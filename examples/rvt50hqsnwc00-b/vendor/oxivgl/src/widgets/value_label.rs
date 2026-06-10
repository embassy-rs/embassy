// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ops::Deref;

use heapless::{String, format};
use oxivgl_sys::lv_obj_t;

use super::{WidgetError, label::Label, obj::AsLvHandle};

/// A [`Label`] that formats an `f32` value with a fixed unit suffix.
///
/// [`set_value`](ValueLabel::set_value) formats as `"{:.1}{unit}"` (e.g.
/// `"12.3V"`).
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Screen, ValueLabel};
///
/// let screen = Screen::active().unwrap();
/// let mut vl = ValueLabel::new(&screen, "V").unwrap();
/// vl.set_value(14.2).unwrap(); // displays "14.2V"
/// ```
#[derive(Debug)]
pub struct ValueLabel<'p> {
    label: Label<'p>,
    unit: &'p str,
}

impl<'p> AsLvHandle for ValueLabel<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.label.lv_handle()
    }
}

impl<'p> Deref for ValueLabel<'p> {
    type Target = Label<'p>;
    fn deref(&self) -> &Label<'p> {
        &self.label
    }
}

impl<'p> ValueLabel<'p> {
    /// Create a new value label with the given unit suffix.
    pub fn new(parent: &impl AsLvHandle, unit: &'p str) -> Result<Self, WidgetError> {
        let label = Label::new(parent)?;
        Ok(ValueLabel { label, unit })
    }

    /// Update the displayed value. Formats as "{value:.1}{unit}".
    pub fn set_value(&mut self, value: f32) -> Result<(), WidgetError> {
        let s: String<10> = format!("{:.1}{}", value, self.unit)?;
        self.label.text(s.as_str());
        Ok(())
    }
}
