#[macro_export]
macro_rules! lv_drv_input_pointer_evdev {
    ($disp:ident) => {
        unsafe {
            lvgl_sys::evdev_init();
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::evdev_read),
                None,
                &$disp,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_input_pointer_gtk {
    ($disp:ident) => {
        unsafe {
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::gtkdrv_mouse_read_cb),
                None,
                &$disp,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_input_pointer_sdl {
    ($disp:ident) => {
        unsafe {
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::sdl_mouse_read),
                None,
                &$disp,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_input_ad_touch {
    ($disp:ident) => {
        unsafe {
            lvgl_sys::ad_touch_init();
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::ad_touch_read),
                None,
                &$disp,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_input_ft5406ee8 {
    ($disp:ident) => {
        unsafe {
            lvgl_sys::ft5406ee8_init();
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::ft5406ee8_read),
                None,
                &$disp,
            )
        }
    };
}

#[macro_export]
macro_rules! lv_drv_input_xpt2046 {
    ($disp:ident) => {
        unsafe {
            lvgl_sys::xpt2046_init();
            $crate::input_device::pointer::Pointer::new_raw(
                Some(lvgl_sys::xpt2046_read),
                None,
                &$disp,
            )
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::input_device::InputDriver;
    use crate::tests;
    use crate::*;

    #[test]
    fn gtk_test() {
        tests::initialize_test(true);
        let disp = Display::default();
        let _input = lv_drv_input_pointer_sdl!(disp);
    }
}
