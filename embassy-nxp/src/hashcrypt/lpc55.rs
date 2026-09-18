//! Driver for the HASHCRYPT peripheral, mode switch sckeleton

use embassy_hal_internal::Peri;
use nxp_pac::sct0::regs::Res;
use pac::hashcrypt::vals;
use pac::syscon::vals::HashAesRst::Released;

use crate::pac;
use crate::peripherals::HASHCRYPT;

pub enum KeySize {
    Bits128,
    Bits192,
    Bits256,
}

// Generic driver type
pub struct GenericDriver<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
impl<'d> GenericDriver<'d> {
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
        Self { _peri: peri }
    }

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
    // rename to generic driver or _driver
}

pub trait Digest {
    type Output;
    #[doc = "Accepts an arbitrary-length slice of bytes at the time, to be buffered until a full block is built, then drained in to the FIFO"]
    fn update(&mut self, data: &[u8]);

    #[doc = "Returns the digest of the message streamed via `update`"]
    fn finalise(self) -> Self::Output;
}

pub trait Aes {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), ()> {
        todo!("Add method boady");
        // Universal encrypt confuguration, meaning
        // MSW1ST = true, MSW1ST_OUT = true, SWAPKEY = true, SWAPDAT = true, AESDECRYPT = Encrypt
        // Chop user provided data into words, feed 4 words at the time to indata()
        // Every 4 words, poll digest and apend it to ouptut
        // If the final part of the message is less than 4 words, padd with 0s
        // Before feeding last 4 words, flip STREAMEDLAST to true
        // Check that data.len = output.len
        // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key
    }
    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), ()> {
        todo!("Add method boady");
        // Universal decrypt confuguration, meaning
        // MSW1ST = true, MSW1ST_OUT = true, SWAPKEY = true, SWAPDAT = false, AESDECRYPT = Decrypt
        // Chop user provided data into words, feed 4 words at the time to indata()
        // Every 4 words, poll digest and apend it to ouptut
        // If the final part of the message is less than 4 words, padd with 0s
        // Before feeding last 4 words, flip STREAMEDLAST to true
        // Check that data.len = output.len
        // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key
    }

    fn set_key_size(&mut self, size: KeySize) {
        todo!("Add method boady");
        // Select key size via register calls
    }

    fn key_size(&self) -> u8 {
        todo!("Add method boady !");
        // get key size via register calls
    }

    fn set_key(&mut self, key: &[u8]) -> Result<(), ()> {
        todo!("Add method boady");
        // use fn key_size to check against user provided data, compair key_size with key.len()
    }
}

// Specific driver types
// todo!("Add buffer, buffer len and message length for sha1");
pub struct Sha1<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Digest for Sha1<'a, 'd> {
    type Output = [u8; 20];

    fn update(&mut self, data: &[u8]) {
        todo!("Add update function for Sha1");
    }

    fn finalise(self) -> Self::Output {
        todo!("Add finalise method for Sha1");
    }
}

// todo!("Add buffer, buffer len and message length for sha2");
pub struct Sha256<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Digest for Sha256<'a, 'd> {
    type Output = [u8; 32];

    fn update(&mut self, data: &[u8]) {
        todo!("Add update method for Sha 256");
    }

    fn finalise(self) -> Self::Output {
        todo!("Add finalise method for Sha 256");
    }
}

pub struct AesEcb<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Aes for AesEcb<'a, 'd> {
    // No change to the default methods needed
}

impl<'a, 'd> AesEcb<'a, 'd> {
    // Does not require anything passed the default aes methods
}
pub struct AesCbc<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}
impl<'a, 'd> Aes for AesCbc<'a, 'd> {
    // Does not require anything passed the default aes methods
}
impl<'a, 'd> AesCbc<'a, 'd> {
    fn set_iv(&mut self, iv: &[u8; 16]) -> Result<(), ()> {
        todo!("Add method boady");
    }
}
pub struct AesCtr<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Aes for AesCtr<'a, 'd> {
    // Does not require anything passed the default aes methods
}

impl<'a, 'd> AesCtr<'a, 'd> {
    pub fn set_counter(&mut self, couteer: &[u8; 16]) -> Result<(), ()> {
        todo!("Add method boady");
    }
}
