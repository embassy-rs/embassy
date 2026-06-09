pub enum MeterPart {
    Arc,
    Needle,
    Tick,
}

impl From<MeterPart> for u8 {
    fn from(part: MeterPart) -> Self {
        match part {
            MeterPart::Arc => lvgl_sys::lv_meter_draw_part_type_t_LV_METER_DRAW_PART_ARC as u8,
            MeterPart::Needle => {
                lvgl_sys::lv_meter_draw_part_type_t_LV_METER_DRAW_PART_NEEDLE_LINE as u8
            }
            MeterPart::Tick => lvgl_sys::lv_meter_draw_part_type_t_LV_METER_DRAW_PART_TICK as u8,
        }
    }
}
