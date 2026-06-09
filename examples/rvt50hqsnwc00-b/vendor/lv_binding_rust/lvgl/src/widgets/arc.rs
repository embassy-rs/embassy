use crate::lv_core::obj::NativeObject;
use crate::widgets::Arc;

impl Arc<'_> {
    // /// Set the start angle, for the given arc part.
    // /// 0 degrees for the right, 90 degrees for the bottom, etc.
    // pub fn set_start_angle(&mut self, angle: u16, part: ArcPart) -> LvResult<()> {
    //     match part {
    //         ArcPart::Background => unsafe {
    //             lvgl_sys::lv_arc_set_bg_start_angle(self.core.raw()?.as_mut(), angle)
    //         },
    //         ArcPart::Indicator => unsafe {
    //             lvgl_sys::lv_arc_set_start_angle(self.core.raw()?.as_mut(), angle)
    //         },
    //     }
    //     Ok(())
    // }
    //
    // /// Set the end angle, for the given arc part.
    // /// 0 degrees for the right, 90 degrees for the bottom, etc.
    // pub fn set_end_angle(&self, angle: u16, part: ArcPart) -> LvResult<()> {
    //     match part {
    //         ArcPart::Background => unsafe {
    //             lvgl_sys::lv_arc_set_bg_end_angle(self.core.raw()?.as_mut(), angle)
    //         },
    //         ArcPart::Indicator => unsafe {
    //             lvgl_sys::lv_arc_set_end_angle(self.core.raw()?.as_mut(), angle)
    //         },
    //     }
    //     Ok(())
    // }
    //
    // /// Rotate the arc, `angle` degrees clockwise.
    // pub fn set_rotation(&mut self, angle: u16) -> LvResult<()> {
    //     unsafe {
    //         lvgl_sys::lv_arc_set_rotation(self.core.raw()?.as_mut(), angle);
    //     }
    //     Ok(())
    // }
}
/*
/// The different parts, of an arc object.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ArcPart {
    /// The background of the arc.
    Background = lvgl_sys::LV_ARC_PART_BG as u8,
    /// The indicator of the arc.
    /// This is what moves/changes, depending on the arc's value.
    Indicator = lvgl_sys::LV_ARC_PART_INDIC as u8,
}

impl From<ArcPart> for u8 {
    fn from(part: ArcPart) -> Self {
        part as u8
    }
}
*/
