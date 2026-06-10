// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::string::String;
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// Per-cell control bits for [`Table`] cells.
///
/// Use bitwise OR (`|`) to combine flags.
///
/// Corresponds to `lv_table_cell_ctrl_t`. Defined as a newtype to allow
/// arbitrary combinations without restricting to the named variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableCellCtrl(pub u32);

impl TableCellCtrl {
    /// No special control bits.
    pub const NONE: Self = Self(0);
    /// Merge this cell with the one to the right.
    pub const MERGE_RIGHT: Self = Self(1 << 0);
    /// Crop cell text instead of wrapping.
    pub const TEXT_CROP: Self = Self(1 << 1);
    /// Application-defined flag 1 (e.g. checked/selected state).
    pub const CUSTOM_1: Self = Self(1 << 4);
    /// Application-defined flag 2.
    pub const CUSTOM_2: Self = Self(1 << 5);
    /// Application-defined flag 3.
    pub const CUSTOM_3: Self = Self(1 << 6);
    /// Application-defined flag 4.
    pub const CUSTOM_4: Self = Self(1 << 7);
}

impl core::ops::BitOr for TableCellCtrl {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for TableCellCtrl {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// LVGL table widget — a scrollable grid of text cells.
///
/// Requires `LV_USE_TABLE = 1` in `lv_conf.h`.
///
/// Rows and columns are added automatically when
/// [`set_cell_value`](Self::set_cell_value) addresses a cell beyond the current
/// dimensions.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Screen, Table};
///
/// let screen = Screen::active().unwrap();
/// let table = Table::new(&screen).unwrap();
/// table.set_cell_value(0, 0, "Name").set_cell_value(0, 1, "Price");
/// table.set_cell_value(1, 0, "Apple").set_cell_value(1, 1, "$7");
/// table.set_column_width(0, 120).set_column_width(1, 80);
/// ```
#[derive(Debug)]
pub struct Table<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Table<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Table<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Table<'p> {
    /// Create a table as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted); lv_init() called via LvglDriver.
        let handle = unsafe { lv_table_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Table { obj: Obj::from_raw(handle) }) }
    }

    /// Set the text of a cell. LVGL copies the string internally.
    ///
    /// New rows/columns are added automatically if `row`/`col` exceed the
    /// current count. Strings longer than 127 bytes are silently truncated.
    pub fn set_cell_value(&self, row: u32, col: u32, text: &str) -> &Self {
        let bytes = text.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated; LVGL copies the text
        // (lv_table.c: lv_strdup used internally).
        unsafe { lv_table_set_cell_value(self.lv_handle(), row, col, buf.as_ptr() as *const c_char) };
        self
    }

    /// Set the number of rows.
    ///
    /// Pre-allocating rows avoids repeated reallocation when filling a large
    /// table.
    pub fn set_row_count(&self, row_cnt: u32) -> &Self {
        // SAFETY: handle non-null; LVGL manages internal row allocation.
        unsafe { lv_table_set_row_count(self.lv_handle(), row_cnt) };
        self
    }

    /// Set the number of columns.
    pub fn set_column_count(&self, col_cnt: u32) -> &Self {
        // SAFETY: handle non-null; LVGL manages internal column allocation.
        unsafe { lv_table_set_column_count(self.lv_handle(), col_cnt) };
        self
    }

    /// Set the pixel width of a column.
    pub fn set_column_width(&self, col_id: u32, w: i32) -> &Self {
        // SAFETY: handle non-null; col_id out of range is clamped by LVGL.
        unsafe { lv_table_set_column_width(self.lv_handle(), col_id, w) };
        self
    }

    /// Add control bits to a cell.
    pub fn set_cell_ctrl(&self, row: u32, col: u32, ctrl: TableCellCtrl) -> &Self {
        // SAFETY: handle non-null; ctrl is a valid bitmask.
        unsafe { lv_table_set_cell_ctrl(self.lv_handle(), row, col, ctrl.0 as lv_table_cell_ctrl_t) };
        self
    }

    /// Clear (remove) control bits from a cell.
    pub fn clear_cell_ctrl(&self, row: u32, col: u32, ctrl: TableCellCtrl) -> &Self {
        // SAFETY: handle non-null; ctrl is a valid bitmask.
        unsafe { lv_table_clear_cell_ctrl(self.lv_handle(), row, col, ctrl.0 as lv_table_cell_ctrl_t) };
        self
    }

    /// Get the text of a cell as an owned `String`. Returns `None` if the
    /// cell is empty or out of range.
    ///
    /// An owned value is returned because any subsequent call that modifies
    /// the table (e.g. [`set_cell_value`](Self::set_cell_value),
    /// [`set_row_count`](Self::set_row_count)) may reallocate LVGL's internal
    /// cell buffer, invalidating a borrowed pointer.
    pub fn get_cell_value(&self, row: u32, col: u32) -> Option<String> {
        // SAFETY: handle non-null; lv_table_get_cell_value returns a pointer
        // into LVGL's internal cell buffer. We copy immediately via CStr → String
        // to avoid holding a raw reference across any mutation.
        let ptr = unsafe { lv_table_get_cell_value(self.lv_handle(), row, col) };
        if ptr.is_null() {
            return None;
        }
        // SAFETY: LVGL guarantees the returned string is valid NUL-terminated
        // text; we copy the bytes before returning.
        let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
        cstr.to_str().ok().map(String::from)
    }

    /// Get the current number of rows.
    pub fn get_row_count(&self) -> u32 {
        // SAFETY: handle non-null.
        unsafe { lv_table_get_row_count(self.lv_handle()) }
    }

    /// Get the current number of columns.
    pub fn get_column_count(&self) -> u32 {
        // SAFETY: handle non-null.
        unsafe { lv_table_get_column_count(self.lv_handle()) }
    }

    /// Get the pixel width of a column.
    pub fn get_column_width(&self, col_id: u32) -> i32 {
        // SAFETY: handle non-null.
        unsafe { lv_table_get_column_width(self.lv_handle(), col_id) }
    }

    /// Check whether a cell has all the given control bits set.
    pub fn has_cell_ctrl(&self, row: u32, col: u32, ctrl: TableCellCtrl) -> bool {
        // SAFETY: handle non-null.
        unsafe { lv_table_has_cell_ctrl(self.lv_handle(), row, col, ctrl.0 as lv_table_cell_ctrl_t) }
    }

    /// Get the currently selected (focused) cell.
    ///
    /// Returns `None` if no cell is selected. The sentinel value
    /// `0xFFFF` (`LV_TABLE_CELL_NONE`) indicates no selection.
    pub fn get_selected_cell(&self) -> Option<(u32, u32)> {
        let mut row: u32 = 0xFFFF;
        let mut col: u32 = 0xFFFF;
        // SAFETY: handle non-null; row/col are output parameters written by LVGL.
        unsafe { lv_table_get_selected_cell(self.lv_handle(), &mut row, &mut col) };
        if row == 0xFFFF || col == 0xFFFF { None } else { Some((row, col)) }
    }

    /// Set the selected cell.
    pub fn set_selected_cell(&self, row: u16, col: u16) -> &Self {
        // SAFETY: handle non-null.
        unsafe { lv_table_set_selected_cell(self.lv_handle(), row, col) };
        self
    }
}

#[cfg(test)]
mod tests {
    use super::TableCellCtrl;

    #[test]
    fn table_cell_ctrl_values() {
        assert_eq!(TableCellCtrl::NONE.0, 0);
        assert_eq!(TableCellCtrl::MERGE_RIGHT.0, 1);
        assert_eq!(TableCellCtrl::TEXT_CROP.0, 2);
        assert_eq!(TableCellCtrl::CUSTOM_1.0, 16);
        assert_eq!(TableCellCtrl::CUSTOM_2.0, 32);
        assert_eq!(TableCellCtrl::CUSTOM_3.0, 64);
        assert_eq!(TableCellCtrl::CUSTOM_4.0, 128);
    }

    #[test]
    fn table_cell_ctrl_bitor() {
        let combined = TableCellCtrl::MERGE_RIGHT | TableCellCtrl::TEXT_CROP;
        assert_eq!(combined.0, 3);
    }
}
