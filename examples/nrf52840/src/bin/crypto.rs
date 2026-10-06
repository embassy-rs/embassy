//! Hardware-accelerated crypto with the CryptoCell: SHA-256, HMAC, AES-CCM and ChaCha20.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{AesCcm, AesCmac, ChaChaVariant, Direction, Sha256, Symmetric};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // The AES, hash and ChaCha engines share one DMA, so one driver owns all three. Operations
    // of the three kinds can still be interleaved: their state lives in contexts, not in the
    // driver.
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

    // AES-CCM authenticated encryption, then decryption.
    let key = [0x42u8; 16];
    let nonce = [0x01u8; 13];
    let aad = b"header";
    let mut data = *b"attack at dawn";
    let cipher = unwrap!(AesCcm::new(&key, &nonce, aad.len(), data.len(), 8));

    let mut ctx = crypto.aes_start(cipher, Direction::Encrypt);
    unwrap!(crypto.aes_blocking_aad(&mut ctx, aad, true));
    unwrap!(crypto.aes_blocking_payload_in_place(&mut ctx, &mut data, true));
    let tag = unwrap!(unwrap!(crypto.aes_blocking_finish(ctx)));
    info!("ciphertext {:02x} tag {:02x}", data, tag[..8]);

    let mut ctx = crypto.aes_start(cipher, Direction::Decrypt);
    unwrap!(crypto.aes_blocking_aad(&mut ctx, aad, true));
    unwrap!(crypto.aes_blocking_payload_in_place(&mut ctx, &mut data, true));
    let computed = unwrap!(unwrap!(crypto.aes_blocking_finish(ctx)));
    // Compare the tag in constant time before trusting the plaintext.
    let ok = computed[..8].iter().zip(&tag[..8]).fold(0, |acc, (a, b)| acc | (a ^ b)) == 0;
    info!("plaintext {:a} tag ok {}", data, ok);

    // AES-CMAC.
    let mut ctx = crypto.aes_start(unwrap!(AesCmac::new(&key)), Direction::Encrypt);
    unwrap!(crypto.aes_blocking_payload(&mut ctx, b"message", &mut [], true));
    let mac = unwrap!(unwrap!(crypto.aes_blocking_finish(ctx)));
    info!("aes-cmac = {:02x}", mac);

    // ChaCha20 stream cipher.
    let mut ctx = crypto.chacha_start(ChaChaVariant::ChaCha20, &[0x77u8; 32], &[0x11u8; 12], 1);
    let mut data = *b"some more data";
    crypto.chacha_blocking_apply_keystream_in_place(&mut ctx, &mut data);
    info!("chacha20 ciphertext {:02x}", data);
}
