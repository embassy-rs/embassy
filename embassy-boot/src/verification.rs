use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[derive(Debug)]
pub enum VerificationError {
    Flash(NorFlashErrorKind),
    Signature(embassy_crypto::Error),
}

#[cfg(feature = "defmt")]
impl defmt::Format for VerificationError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            VerificationError::Flash(_) => defmt::write!(fmt, "VerificationError::Flash(_)"),
            VerificationError::Signature(_) => defmt::write!(fmt, "VerificationError::Signature(_)"),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Wrapper trait for emassy-crypto hashers
pub trait Digest: sealed::Sealed {
    /// The output size of the hasher (in bytes)
    const OUTPUT_SIZE: usize;
    /// The output type of the hasher
    type Output: AsRef<[u8]>;

    /// Create a new hasher instance
    fn new() -> Self;
    /// Process data, updating the internal state
    fn update(&mut self, data: &[u8]);
    /// Retrieve result and consume hasher instance
    fn finalize(self) -> Self::Output;
}

macro_rules! impl_digest {
    ($($t:ident),*) => {$(
        impl sealed::Sealed for embassy_crypto::$t {}
        impl Digest for embassy_crypto::$t {
            const OUTPUT_SIZE: usize = <embassy_crypto::$t>::OUTPUT_SIZE;
            type Output = [u8; Self::OUTPUT_SIZE];
            fn new() -> Self {
                embassy_crypto::$t::new()
            }
            fn update(&mut self, d: &[u8]) {
                embassy_crypto::$t::update(self, d)
            }
            fn finalize(self) -> [u8; Self::OUTPUT_SIZE] {
                embassy_crypto::$t::finalize(self)
            }
        }
    )*};
}

impl_digest!(Sha1, Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256);

/// Wrapper trait for emassy-crypto signatures
pub trait Signature {
    /// The size of the signature (in bytes)
    const SIGNATURE_SIZE: usize;
    /// Create a new signature instance from a slice of bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, embassy_crypto::Error>
    where
        Self: Sized;
}

/// Wrapper trait for emassy-crypto verifying keys
pub trait VerifyingKey: sealed::Sealed {
    /// The size of the key (in bytes)
    const KEY_SIZE: usize;
    /// The signature type associated to this verifying key
    type Signature: Signature;

    /// Create a new verifying key instance from a slice of bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, embassy_crypto::Error>
    where
        Self: Sized;

    /// Verify a message using a signature
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), embassy_crypto::Error>;
}

macro_rules! impl_signature_p {
    ($($t:ident),*) => {$(
        impl Signature for embassy_crypto::$t::Signature {
            const SIGNATURE_SIZE: usize = embassy_crypto::$t::Signature::SIGNATURE_SIZE;
            fn from_bytes(b: &[u8]) -> Result<embassy_crypto::$t::Signature, embassy_crypto::Error>  {
                embassy_crypto::$t::Signature::from_bytes(b.try_into().map_err(|_| embassy_crypto::Error::InvalidSignature)?)
            }
        }
        impl sealed::Sealed for embassy_crypto::$t::VerifyingKey {}
        impl VerifyingKey for embassy_crypto::$t::VerifyingKey {
            const KEY_SIZE: usize = embassy_crypto::$t::SEC1_SIZE;
            type Signature = embassy_crypto::$t::Signature;
            fn from_bytes(b: &[u8]) -> Result<embassy_crypto::$t::VerifyingKey, embassy_crypto::Error> {
                embassy_crypto::$t::VerifyingKey::from_sec1(b.try_into().map_err(|_| embassy_crypto::Error::InvalidKey)?)
            }
            fn verify(&self, msg: &[u8], signature: &Self::Signature) -> Result<(), embassy_crypto::Error> {
                embassy_crypto::$t::VerifyingKey::verify_prehash(self, msg.try_into().map_err(|_| embassy_crypto::Error::InvalidInput)?, signature)
            }
        }
    )*};
}

impl_signature_p!(p256, p384);

impl sealed::Sealed for embassy_crypto::ed25519::Signature {}
impl Signature for embassy_crypto::ed25519::Signature {
    const SIGNATURE_SIZE: usize = embassy_crypto::ed25519::Signature::SIGNATURE_SIZE;
    fn from_bytes(b: &[u8]) -> Result<embassy_crypto::ed25519::Signature, embassy_crypto::Error> {
        Ok(embassy_crypto::ed25519::Signature::from_bytes(
            b.try_into().map_err(|_| embassy_crypto::Error::InvalidSignature)?,
        ))
    }
}

impl sealed::Sealed for embassy_crypto::ed25519::VerifyingKey {}
impl VerifyingKey for embassy_crypto::ed25519::VerifyingKey {
    const KEY_SIZE: usize = embassy_crypto::ed25519::VerifyingKey::KEY_SIZE;
    type Signature = embassy_crypto::ed25519::Signature;
    fn from_bytes(b: &[u8]) -> Result<embassy_crypto::ed25519::VerifyingKey, embassy_crypto::Error> {
        Ok(embassy_crypto::ed25519::VerifyingKey::from_bytes(
            b.try_into().map_err(|_| embassy_crypto::Error::InvalidKey)?,
        ))
    }
    fn verify(&self, msg: &[u8], signature: &Self::Signature) -> Result<(), embassy_crypto::Error> {
        embassy_crypto::ed25519::VerifyingKey::verify(self, msg, signature)
    }
}

macro_rules! verification_funcs {
    ($flash: path $(, $async: tt, $await: tt)?) => {
        pub(crate) $( $async )? fn verify<DFU: $flash, D: Digest, V: VerifyingKey>(
            dfu: &mut DFU,
            public_key: &[u8],
            signature: &[u8],
            update_len: u32,
            chunk_buf: &mut [u8],
        ) -> Result<(), VerificationError> {
            let public_key = V::from_bytes(public_key)
                .map_err(|error| VerificationError::Signature(error))?;
            let signature = V::Signature::from_bytes(signature)
                .map_err(|error| VerificationError::Signature(error))?;

            let message = hash::<_, D>(dfu, update_len, chunk_buf) $(.$await)?
                .map_err(|error| VerificationError::Flash(error.kind()))?;

            public_key.verify(message.as_ref(), &signature)
                .map_err(|error| VerificationError::Signature(error))
        }

        pub(crate) $( $async )? fn hash<DFU: $flash, D: super::Digest>(
            dfu: &mut DFU,
            update_len: u32,
            chunk_buf: &mut [u8],
        ) -> Result<D::Output, DFU::Error> {
            let mut digest = D::new();
            for offset in (0..update_len).step_by(chunk_buf.len()) {
                dfu.read(offset, chunk_buf) $(.$await)? ?;
                let len = chunk_buf.len().min((update_len - offset) as _);
                digest.update(&chunk_buf[..len]);
            }
            Ok(digest.finalize())
        }
    };
}

pub mod blocking {
    use super::*;
    verification_funcs!(embedded_storage::nor_flash::ReadNorFlash);
}
pub mod asynch {
    use super::*;
    verification_funcs!(embedded_storage_async::nor_flash::ReadNorFlash, async, await);
}

#[cfg(test)]
mod tests {
    use embassy_crypto::{ed25519, p256, p384};

    use super::{Signature, VerifyingKey};

    #[test]
    fn key_sizes() {
        // Uncompressed SEC1 points: 0x04 || x || y.
        assert_eq!(<p256::VerifyingKey as VerifyingKey>::KEY_SIZE, 65);
        assert_eq!(<p384::VerifyingKey as VerifyingKey>::KEY_SIZE, 97);
        assert_eq!(<ed25519::VerifyingKey as VerifyingKey>::KEY_SIZE, 32);
    }

    #[test]
    fn signature_sizes() {
        // r || s
        assert_eq!(<p256::Signature as Signature>::SIGNATURE_SIZE, 64);
        assert_eq!(<p384::Signature as Signature>::SIGNATURE_SIZE, 96);
        assert_eq!(<ed25519::Signature as Signature>::SIGNATURE_SIZE, 64);
    }

    #[test]
    fn key_size_is_what_from_bytes_takes() {
        // The P-256 generator point, uncompressed.
        let generator = [
            0x04, 0x6B, 0x17, 0xD1, 0xF2, 0xE1, 0x2C, 0x42, 0x47, 0xF8, 0xBC, 0xE6, 0xE5, 0x63, 0xA4, 0x40, 0xF2, 0x77,
            0x03, 0x7D, 0x81, 0x2D, 0xEB, 0x33, 0xA0, 0xF4, 0xA1, 0x39, 0x45, 0xD8, 0x98, 0xC2, 0x96, 0x4F, 0xE3, 0x42,
            0xE2, 0xFE, 0x1A, 0x7F, 0x9B, 0x8E, 0xE7, 0xEB, 0x4A, 0x7C, 0x0F, 0x9E, 0x16, 0x2B, 0xCE, 0x33, 0x57, 0x6B,
            0x31, 0x5E, 0xCE, 0xCB, 0xB6, 0x40, 0x68, 0x37, 0xBF, 0x51, 0xF5,
        ];
        assert_eq!(generator.len(), <p256::VerifyingKey as VerifyingKey>::KEY_SIZE);
        assert!(<p256::VerifyingKey as VerifyingKey>::from_bytes(&generator).is_ok());
        assert!(<p256::VerifyingKey as VerifyingKey>::from_bytes(&generator[1..]).is_err());
    }
}
