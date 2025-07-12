#[cfg(feature = "ed25519-dalek")]
pub(crate) mod ed25519_dalek;

#[cfg(feature = "ed25519-salty")]
pub(crate) mod salty;
