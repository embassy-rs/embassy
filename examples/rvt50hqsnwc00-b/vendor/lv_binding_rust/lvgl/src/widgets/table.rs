use crate::lv_core::obj::NativeObject;
use crate::widgets::Table;
use core::mem::MaybeUninit;

impl Table<'_> {
    /// Sets the column width. Row height cannot be set manually and is
    /// calculated by LVGL based on styling parameters.
    pub fn set_col_width(&mut self, column: u16, width: i16) {
        unsafe { lvgl_sys::lv_table_set_col_width(self.core.raw().as_ptr(), column, width) }
    }

    /// Returns the selected cell as a tuple of (row, column).
    pub fn get_selected_cell(&self) -> (u16, u16) {
        let mut row = MaybeUninit::<u16>::uninit();
        let mut col = MaybeUninit::<u16>::uninit();
        unsafe {
            lvgl_sys::lv_table_get_selected_cell(
                self.core.raw().as_ptr(),
                row.as_mut_ptr(),
                col.as_mut_ptr(),
            );
            // The values get initialised by LVGL
            (row.assume_init(), col.assume_init())
        }
    }
}
