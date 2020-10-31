use crate::util::Dewrap;
pub trait Rand {
    fn rand(&self, buf: &mut [u8]);
}

static mut RAND: Option<&'static dyn Rand> = None;

pub unsafe fn set_rand(rand: &'static dyn Rand) {
    RAND = Some(rand);
}

pub fn rand(buf: &mut [u8]) {
    unsafe { RAND.dexpect(defmt::intern!("No rand set")).rand(buf) }
}
