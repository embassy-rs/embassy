//! Hardware-accelerated crypto with CRACEN: SHA-256, HMAC, AES-GCM and ChaCha20-Poly1305.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{AesGcm, ChaChaVariant, Direction, Sha256, Symmetric};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // The AES, hash and ChaCha20-Poly1305 engines share one DMA, so one driver owns all three.
    // Operations of the three kinds can still be interleaved: their state lives in contexts,
    // not in the driver.
    let mut crypto = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);

    // Hashing and HMAC. Data can be fed in any number of chunks.
    let mut ctx = crypto.hash_start::<Sha256>();
    crypto.hash_blocking_update(&mut ctx, b"hello ");
    crypto.hash_blocking_update(&mut ctx, b"world");
    let digest = crypto.hash_blocking_finish(ctx);
    info!("sha256(\"hello world\") = {:02x}", digest);

    let mut ctx = crypto.hmac_start::<Sha256>(b"secret key");
    crypto.hash_blocking_update(&mut ctx, b"hello world");
    let mac = crypto.hash_blocking_finish(ctx);
    info!("hmac-sha256 = {:02x}", mac);

    // AES-256-GCM authenticated encryption, then decryption.
    let key = [0x42u8; 32];
    let iv = [0x01u8; 12];
    let aad = b"header";
    let mut data = *b"attack at dawn";
    let cipher = unwrap!(AesGcm::new(&key, &iv));

    let mut ctx = crypto.aes_start(cipher, Direction::Encrypt);
    unwrap!(crypto.aes_blocking_aad(&mut ctx, aad, true));
    unwrap!(crypto.aes_blocking_payload_in_place(&mut ctx, &mut data, true));
    let tag = unwrap!(unwrap!(crypto.aes_blocking_finish(ctx)));
    info!("ciphertext {:02x} tag {:02x}", data, tag);

    let mut ctx = crypto.aes_start(cipher, Direction::Decrypt);
    unwrap!(crypto.aes_blocking_aad(&mut ctx, aad, true));
    unwrap!(crypto.aes_blocking_payload_in_place(&mut ctx, &mut data, true));
    let computed = unwrap!(unwrap!(crypto.aes_blocking_finish(ctx)));
    // Compare the tag in constant time before trusting the plaintext.
    let ok = computed.iter().zip(&tag).fold(0, |acc, (a, b)| acc | (a ^ b)) == 0;
    info!("plaintext {:a} tag ok {}", data, ok);

    // ChaCha20-Poly1305.
    let key = [0x77u8; 32];
    let nonce = [0x11u8; 12];
    let mut data = *b"some more data";
    let mut ctx = crypto.chachapoly_start(ChaChaVariant::ChaCha20, &key, &nonce, Direction::Encrypt);
    unwrap!(crypto.chachapoly_blocking_aad(&mut ctx, aad, true));
    unwrap!(crypto.chachapoly_blocking_payload_in_place(&mut ctx, &mut data, true));
    let tag = unwrap!(crypto.chachapoly_blocking_finish(ctx));
    info!("chacha20-poly1305 ciphertext {:02x} tag {:02x}", data, tag);
}
