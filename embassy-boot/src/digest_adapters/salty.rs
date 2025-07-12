use digest::typenum::U64;
use digest::{FixedOutput, HashMarker, OutputSizeUser, Update};

pub struct Sha512(salty::Sha512);

impl Default for Sha512 {
    fn default() -> Self {
        Self(salty::Sha512::new())
    }
}

impl Update for Sha512 {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data)
    }
}

impl FixedOutput for Sha512 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        let result = self.0.finalize();
        out.as_mut_slice().copy_from_slice(result.as_slice())
    }
}

impl OutputSizeUser for Sha512 {
    type OutputSize = U64;
}

impl HashMarker for Sha512 {}
