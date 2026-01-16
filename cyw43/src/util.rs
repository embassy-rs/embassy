#![allow(unused)]

use core::slice;

use aligned::{A4, Aligned};
use embassy_time::{Duration, Ticker};

pub(crate) const fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = size_of_val(x);
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) const fn slice16_mut(x: &mut [u32]) -> &mut [u16] {
    let len = size_of_val(x) / 2;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) const fn aligned_mut(x: &mut [u32]) -> &mut Aligned<A4, [u8]> {
    let len = size_of_val(x);
    unsafe { core::mem::transmute(slice::from_raw_parts_mut(x.as_mut_ptr() as *mut u8, len)) }
}

pub(crate) const fn aligned_ref(x: &[u32]) -> &Aligned<A4, [u8]> {
    let len = size_of_val(x);
    unsafe { core::mem::transmute(slice::from_raw_parts(x.as_ptr() as *const u8, len)) }
}

pub(crate) const fn slice32_mut(x: &mut Aligned<A4, [u8]>) -> &mut [u32] {
    let len = (size_of_val(x) + 3) / 4;
    unsafe { slice::from_raw_parts_mut(x as *mut Aligned<A4, [u8]> as *mut u32, len) }
}

pub(crate) const fn slice32_ref(x: &Aligned<A4, [u8]>) -> &[u32] {
    let len = (size_of_val(x) + 3) / 4;
    unsafe { slice::from_raw_parts(x as *const Aligned<A4, [u8]> as *const u32, len) }
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

pub(crate) async fn try_until(mut func: impl AsyncFnMut() -> bool, duration: Duration) -> bool {
    let tick = Duration::from_millis(1);
    let mut ticker = Ticker::every(tick);
    let ticks = duration.as_ticks() / tick.as_ticks();

    for _ in 0..ticks {
        if func().await {
            return true;
        }

        ticker.next().await;
    }

    false
}
