use lvgl_sys::lv_coord_t;

pub static LV_SIZE_CONTENT: u32 = 2001 | lvgl_sys::_LV_COORD_TYPE_SPEC;

pub fn pct(pct: lv_coord_t) -> lv_coord_t {
    if pct > 0 {
        pct | unsafe {
            <u32 as TryInto<lv_coord_t>>::try_into(lvgl_sys::_LV_COORD_TYPE_SPEC).unwrap_unchecked()
        }
    } else {
        (1000 - pct)
            | unsafe {
                <u32 as TryInto<lv_coord_t>>::try_into(lvgl_sys::_LV_COORD_TYPE_SPEC)
                    .unwrap_unchecked()
            }
    }
}

pub fn coord_is_pct(pct: lv_coord_t) -> bool {
    (pct & unsafe {
        <u32 as TryInto<lv_coord_t>>::try_into(lvgl_sys::_LV_COORD_TYPE_MASK).unwrap_unchecked()
    } == unsafe { lvgl_sys::_LV_COORD_TYPE_SPEC.try_into().unwrap_unchecked() })
        && (pct
            & !unsafe {
                <u32 as TryInto<lv_coord_t>>::try_into(lvgl_sys::_LV_COORD_TYPE_MASK)
                    .unwrap_unchecked()
            }
            <= 2000)
}

pub fn coord_get_pct(pct: lv_coord_t) -> lv_coord_t {
    (pct & !unsafe {
        <u32 as TryInto<lv_coord_t>>::try_into(lvgl_sys::_LV_COORD_TYPE_MASK).unwrap_unchecked()
    }) % 1000
}
