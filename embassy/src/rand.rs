use crate::fmt::*;

pub trait Rand {
    fn rand(&self, buf: &mut [u8]);
}

#[cfg(feature = "std")]
static mut RAND: Option<&'static dyn Rand> = Some(&if_std::Rand);
#[cfg(not(feature = "std"))]
static mut RAND: Option<&'static dyn Rand> = None;

pub unsafe fn set_rand(rand: &'static dyn Rand) {
    RAND = Some(rand);
}

pub fn rand(buf: &mut [u8]) {
    unsafe { unwrap!(RAND, "No rand set").rand(buf) }
}

#[cfg(feature = "std")]
mod if_std {
    use rand_core::{OsRng, RngCore};

    pub(crate) struct Rand;
    impl super::Rand for Rand {
        fn rand(&self, buf: &mut [u8]) {
            OsRng.fill_bytes(buf)
        }
    }
}
