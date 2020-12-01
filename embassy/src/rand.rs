use crate::fmt::*;

pub trait Rand {
    fn rand(&self, buf: &mut [u8]);
}

static mut RAND: Option<&'static dyn Rand> = None;

pub unsafe fn set_rand(rand: &'static dyn Rand) {
    RAND = Some(rand);
}

pub fn rand(buf: &mut [u8]) {
    unsafe { unwrap!(RAND, "No rand set").rand(buf) }
}
