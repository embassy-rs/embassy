#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn _bindgen_raw_src() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/bindings.rs"))
}

mod string_impl;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sanity_check() {
        unsafe {
            lv_init();

            let horizontal_resolution = lv_disp_get_hor_res(core::ptr::null_mut());
            assert_eq!(horizontal_resolution, 0);

            let vertical_resolution = lv_disp_get_ver_res(core::ptr::null_mut());
            assert_eq!(vertical_resolution, 0);
        }
    }
}
