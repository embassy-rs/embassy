#![allow(unused)]

use core::slice;

pub(crate) fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) fn is_aligned(a: u32, x: u32) -> bool {
    (a & (x - 1)) == 0
}

pub(crate) fn round_down(x: u32, a: u32) -> u32 {
    x & !(a - 1)
}

pub(crate) fn round_up(x: u32, a: u32) -> u32 {
    ((x + a - 1) / a) * a
}
