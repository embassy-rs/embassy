# embassy-crypto

RustCrypto trait implementations backed by `embassy-crypto-driver` unitraits.

This crate wraps the hardware-agnostic unitraits from `embassy-crypto-driver`
with the standard RustCrypto traits, so existing RustCrypto code can use
embassy-registered crypto drivers without modification.

# crate design

- The crate must match closely to the existing rustcrypto API. Deviations from this API
  must have a very good reason that can be clearly explained.
- Aside from RNG, every operation must have a backing rustcrypto driver that serves as
  the reference implementation. *embassy-crypto* should function as a thin layer that calls
  into rustcrypto with the `driver-rustcrypto` feature enabled.
- Given that hardware takes some time to setup, the *embassy-crypto* types should batch
  operations with calls into the unitrait that operate on large buffers where possible.
  At least, the implemented traits or methods should allow the user the possibility of this.

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

# Linkage
At link time exactly one crate in the dependency tree must register a driver
using the `embassy_crypto_*_impl!` macros from `embassy-crypto-driver`.
If zero or multiple drivers are registered, linking will fail.