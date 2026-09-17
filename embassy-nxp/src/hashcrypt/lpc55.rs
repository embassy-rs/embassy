//! Driver for the HASHCRYPT peripheral, mode switch sckeleton

use embassy_hal_internal::Peri;
use pac::hashcrypt::vals;
use pac::syscon::vals::HashAesRst::Released;

use crate::pac;
use crate::peripherals::HASHCRYPT;

// Generic driver type
struct GenericDriver<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
impl<'d> GenericDriver<'d> {
    pub fn sha1(&mut self) -> Sha1<'_, 'd> {
        Sha1 { _peri: self }
    }

    pub fn sha256(&mut self) -> Sha256<'_, 'd> {
        Sha256 { _peri: self }
    }

    pub fn aes_ecb(&mut self) -> AesEcb<'_, 'd> {
        AesEcb { _peri: self }
    }

    pub fn aes_cbc(&mut self) -> AesCbc<'_, 'd> {
        AesCbc { _peri: self }
    }

    pub fn aes_ctr(&mut self) -> AesCtr<'_, 'd> {
        AesCtr { _peri: self }
    }
}

// Specific driver types
// todo!("Add buffer, buffer len and message length for sha1");
struct Sha1<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

// todo!("Add buffer, buffer len and message length for sha2");
struct Sha256<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

// todo!("Add key ebc");
struct AesEcb<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

// todo!("Add key and iv for cbc");
struct AesCbc<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

// todo!("Add key and counter for ctr");
struct AesCtr<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}
