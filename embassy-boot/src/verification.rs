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
    ($(($t:ident, $n: literal)),*) => {$(
        impl Signature for embassy_crypto::$t::Signature {
            const SIGNATURE_SIZE: usize = 2 * $n;
            fn from_bytes(b: &[u8]) -> Result<embassy_crypto::$t::Signature, embassy_crypto::Error>  {
                embassy_crypto::$t::Signature::from_bytes(b.try_into().map_err(|_| embassy_crypto::Error::InvalidSignature)?)
            }
        }
        impl sealed::Sealed for embassy_crypto::$t::VerifyingKey {}
        impl VerifyingKey for embassy_crypto::$t::VerifyingKey {
            const KEY_SIZE: usize = 2 * $n + 1;
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

impl_signature_p!((p256, 32), (p384, 48));

impl sealed::Sealed for embassy_crypto::ed25519::Signature {}
impl Signature for embassy_crypto::ed25519::Signature {
    const SIGNATURE_SIZE: usize = 64;
    fn from_bytes(b: &[u8]) -> Result<embassy_crypto::ed25519::Signature, embassy_crypto::Error> {
        Ok(embassy_crypto::ed25519::Signature::from_bytes(
            b.try_into().map_err(|_| embassy_crypto::Error::InvalidSignature)?,
        ))
    }
}

impl sealed::Sealed for embassy_crypto::ed25519::VerifyingKey {}
impl VerifyingKey for embassy_crypto::ed25519::VerifyingKey {
    const KEY_SIZE: usize = 32;
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
