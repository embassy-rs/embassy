// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{cell::RefCell, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    child::Child,
    obj::{AsLvHandle, Obj},
};

/// A calendar date (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDate {
    /// Full year, e.g. 2024.
    pub year: u16,
    /// Month 1–12.
    pub month: u8,
    /// Day 1–31.
    pub day: u8,
}

impl CalendarDate {
    /// Create a new calendar date.
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

impl From<lv_calendar_date_t> for CalendarDate {
    fn from(d: lv_calendar_date_t) -> Self {
        Self { year: d.year, month: d.month, day: d.day }
    }
}

impl From<CalendarDate> for lv_calendar_date_t {
    fn from(d: CalendarDate) -> Self {
        Self { year: d.year, month: d.month, day: d.day }
    }
}

/// LVGL calendar widget — a month view with day selection, highlighted dates,
/// and optional navigation header.
///
/// Requires `LV_USE_CALENDAR = 1` in `lv_conf.h`.
///
/// Call [`add_header_arrow`](Self::add_header_arrow) or
/// [`add_header_dropdown`](Self::add_header_dropdown) to attach a navigation
/// header for month/year browsing.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Calendar, CalendarDate, Screen};
///
/// let screen = Screen::active().unwrap();
/// let cal = Calendar::new(&screen).unwrap();
/// cal.set_today_date(2024, 3, 22)
///    .set_month_shown(2024, 3);
/// cal.add_header_arrow();
/// ```
#[derive(Debug)]
pub struct Calendar<'p> {
    obj: Obj<'p>,
    /// Owns the highlighted-dates array passed to LVGL. LVGL only stores the
    /// pointer; this Vec keeps the data alive.
    highlighted: RefCell<Vec<lv_calendar_date_t>>,
}

impl<'p> AsLvHandle for Calendar<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Calendar<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Calendar<'p> {
    /// Create a calendar as a child of `parent`.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_calendar_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Calendar { obj: Obj::from_raw(handle), highlighted: RefCell::new(Vec::new()) })
        }
    }

    /// Set today's date (shown with a special style).
    pub fn set_today_date(&self, year: u16, month: u8, day: u8) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_calendar_set_today_date(self.obj.handle(), year as u32, month as u32, day as u32) };
        self
    }

    /// Set the month to display.
    pub fn set_month_shown(&self, year: u16, month: u8) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_calendar_set_month_shown(self.obj.handle(), year as u32, month as u32) };
        self
    }

    /// Set dates to highlight. The dates are copied into an internal Vec whose
    /// pointer is passed to LVGL; LVGL stores only the pointer, so the data
    /// must remain live for the calendar's lifetime.
    pub fn set_highlighted_dates(&self, dates: &[CalendarDate]) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        let mut v: alloc::vec::Vec<lv_calendar_date_t> =
            dates.iter().copied().map(lv_calendar_date_t::from).collect();
        // SAFETY: handle non-null; v.as_mut_ptr() is valid and pinned in the
        // RefCell. LVGL stores only the pointer — we keep ownership here.
        unsafe {
            lv_calendar_set_highlighted_dates(self.obj.handle(), v.as_mut_ptr(), v.len())
        };
        *self.highlighted.borrow_mut() = v;
        self
    }

    /// Get the date the user last pressed, if any.
    pub fn get_pressed_date(&self) -> Option<CalendarDate> {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        let mut date = lv_calendar_date_t { year: 0, month: 0, day: 0 };
        // SAFETY: handle non-null; date is a valid out-param.
        let ok = unsafe { lv_calendar_get_pressed_date(self.obj.handle(), &mut date) };
        if ok == lv_result_t_LV_RESULT_OK { Some(date.into()) } else { None }
    }

    /// Get today's date as set via [`set_today_date`](Self::set_today_date).
    pub fn get_today_date(&self) -> CalendarDate {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null; lv_calendar_get_today_date returns a valid
        // pointer into the widget's internal state (never null after create).
        let ptr = unsafe { lv_calendar_get_today_date(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_calendar_get_today_date returned NULL");
        unsafe { (*ptr).into() }
    }

    /// Get the currently displayed month/year.
    pub fn get_showed_date(&self) -> CalendarDate {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null; lv_calendar_get_showed_date returns a valid
        // pointer into the widget's internal state (never null after create).
        let ptr = unsafe { lv_calendar_get_showed_date(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_calendar_get_showed_date returned NULL");
        unsafe { (*ptr).into() }
    }

    /// Add an arrow-button navigation header (← month →) to the calendar.
    ///
    /// Returns a non-owning handle to the header object (owned by the calendar).
    pub fn add_header_arrow(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null; LVGL creates the header as a child of the calendar.
        let ptr = unsafe { lv_calendar_add_header_arrow(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_calendar_add_header_arrow returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Enable or disable Chinese lunar calendar mode.
    ///
    /// When enabled, day cells display Chinese lunar calendar names
    /// (e.g. "25\n十六"). A CJK font must be provided via `cjk_font` so
    /// the btnmatrix items can render the Chinese glyphs. The font is
    /// applied to `Part::Items` of the internal button matrix.
    ///
    /// Requires `LV_USE_CALENDAR_CHINESE = 1` in `lv_conf.h`.
    pub fn set_chinese_mode(&self, en: bool, cjk_font: crate::fonts::Font) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null. Declared locally because oxivgl_sys
        // may not re-export the function across edition boundaries.
        unsafe extern "C" {
            fn lv_calendar_set_chinese_mode(obj: *mut oxivgl_sys::lv_obj_t, en: bool);
        }
        if en {
            // Set the CJK font BEFORE enabling Chinese mode, because
            // set_chinese_mode internally calls set_month_shown which
            // triggers a render pass. The btnmatrix must already have
            // the CJK font when rendering occurs.
            let bm = self.get_btnmatrix();
            bm.font(cjk_font);
            bm.style_text_font(cjk_font, crate::widgets::obj::Part::Items);
        }
        unsafe { lv_calendar_set_chinese_mode(self.obj.handle(), en) };
        self
    }

    /// Add a dropdown navigation header (month + year dropdowns) to the calendar.
    ///
    /// Returns a non-owning handle to the header object (owned by the calendar).
    pub fn add_header_dropdown(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null; LVGL creates the header as a child of the calendar.
        let ptr = unsafe { lv_calendar_add_header_dropdown(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_calendar_add_header_dropdown returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the internal button matrix used for day cells.
    ///
    /// Returns a non-owning handle (owned by the calendar).
    pub fn get_btnmatrix(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Calendar handle cannot be null");
        // SAFETY: handle non-null; lv_calendar_get_btnmatrix returns the
        // internal btnm child (never null after create).
        let ptr = unsafe { lv_calendar_get_btnmatrix(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_calendar_get_btnmatrix returned NULL");
        Child::new(Obj::from_raw(ptr))
    }
}
