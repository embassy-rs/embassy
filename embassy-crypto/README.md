# embassy-crypto

RustCrypto trait implementations backed by `embassy-crypto-driver` unitraits.

This crate wraps the hardware-agnostic unitraits from `embassy-crypto-driver`
with the standard RustCrypto traits, so existing RustCrypto code can use
embassy-registered crypto drivers without modification.

# Supported Operations

## Digests
- `Md5`, `Sha1`, `Sha224`, `Sha256`, `Sha384`, `Sha512`, `Sha512_224`, `Sha512_256`

## HMAC
- `HmacSha1`, `HmacSha224`, `HmacSha256`, `HmacSha384`, `HmacSha512`, `HmacSha512_224`, `HmacSha512_256`

## Block Ciphers
- `Aes128Ecb`, `Aes256Ecb` — ECB mode
- `Aes128Cbc`, `Aes256Cbc` — CBC mode

## AEAD
- `Aes128Gcm`, `Aes256Gcm` — GCM mode
- `Aes128Ccm<TagSize, NonceSize>`, `Aes256Ccm<TagSize, NonceSize>` — CCM mode

## Elliptic Curve (P256)
- `p256::SecretKey`, `p256::PublicKey`, `p256::SharedSecret` — ECDH primitives
- `p256::ecdsa::SigningKey`, `p256::ecdsa::VerifyingKey`, `p256::ecdsa::Signature` — ECDSA primitives

# Digest Usage
```rust,ignore
use embassy_crypto::Sha256;
use digest::Digest;

let mut hasher = Sha256::new();
hasher.update(b"hello world");
let result = hasher.finalize();
```

# HMAC Usage
```rust,ignore
use embassy_crypto::HmacSha256;
use digest::Mac;

let mut mac = HmacSha256::new_from_slice(b"my key").unwrap();
mac.update(b"hello world");
let result = mac.finalize();
```

# Block Cipher Usage
```rust,ignore
use embassy_crypto::Aes128Cbc;
use cipher::{BlockEncryptMut, KeyIvInit};

let mut cipher = Aes128Cbc::new_from_slices(b"my secret key!!!", b"my iv!!!").unwrap();
let mut block = [0u8; 16];
cipher.encrypt_block_mut((&mut block).into());
```

# AEAD Usage
```rust,ignore
use embassy_crypto::Aes128Gcm;
use aead::{Aead, KeyInit, Nonce};

let cipher = Aes128Gcm::new_from_slice(b"my secret key!!!").unwrap();
let nonce = Nonce::from_slice(b"unique nonce");
let ciphertext = cipher.encrypt(nonce, b"plaintext message".as_ref()).unwrap();
```

# P256 ECDH Usage
```rust,ignore
use embassy_crypto::p256::{SecretKey, PublicKey};

let (secret_key, public_key) = SecretKey::generate().unwrap();

let peer_public_key = PublicKey::from_bytes(&[0u8; 65]); // received from peer
let shared_secret = secret_key.diffie_hellman(&peer_public_key).unwrap();
```

# P256 ECDSA Usage
```rust,ignore
use embassy_crypto::p256::ecdsa::{SigningKey, VerifyingKey, Signature};
use signature::{Signer, Verifier};

let signing_key = SigningKey::from_bytes(&[0u8; 32]);
let signature: Signature = signing_key.sign(b"message");

let verifying_key = signing_key.verifying_key().unwrap();
verifying_key.verify(b"message", &signature).unwrap();
```

# Linkage
At link time exactly one crate in the dependency tree must register a driver
using the `embassy_crypto_*_impl!` macros from `embassy-crypto-driver`.
If zero or multiple drivers are registered, linking will fail.