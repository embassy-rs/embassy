#[macro_export]
macro_rules! lv_drv_disp_fbdev {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::fbdev_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::fbdev_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(lvgl_sys::fbdev_exit),
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_drm {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::drm_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::drm_flush),
                None,
                None,
                None,
                None,
                Some(lvgl_sys::drm_wait_vsync),
                None,
                None,
                None,
                Some(lvgl_sys::drm_exit),
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_gtk {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::gtkdrv_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::gtkdrv_flush_cb),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_sdl {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::sdl_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::sdl_display_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_gc9a01 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            match lvgl_sys::GC9A01_init() {
                0 => (),
                c = panic!("GC9A01_init() returned error code {c}")
            };
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::GC9A01_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    }
}

#[macro_export]
macro_rules! lv_drv_disp_ili9341 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::ili9341_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::ili9341_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_r61581 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::r61581_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::r61581_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_sharp_mip {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::sharp_mip_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::sharp_mip_flush),
                Some(lvgl_sys::sharp_mip_rounder),
                Some(lvgl_sys::sharp_mip_set_px),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_ssd1963 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::ssd1963_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::ssd1963_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_st7565 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::st7565_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::st7565_flush),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_disp_uc1610 {
    ($draw_buffer:ident, $hor_res:ident, $ver_res:ident) => {
        unsafe {
            lvgl_sys::uc1610_init();
            $crate::Display::register_raw(
                $draw_buffer,
                $hor_res,
                $ver_res,
                Some(lvgl_sys::uc1610_flush_cb),
                Some(lvgl_sys::uc1610_rounder_cb),
                Some(lvgl_sys::uc1610_set_px_cb),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::tests;
    use crate::DrawBuffer;

    #[test]
    fn gtk_test() {
        const HOR_RES: u32 = 240;
        const VER_RES: u32 = 240;
        tests::initialize_test(false);
        let buffer = DrawBuffer::<{ (HOR_RES * VER_RES) as usize }>::default();
        let _disp = lv_drv_disp_sdl!(buffer, HOR_RES, VER_RES).unwrap();
    }
}
