// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(unsafe_op_in_unsafe_fn)] // bindgen bitfield accessors use transmute without unsafe blocks

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "use-string-functions")]
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
