//! Hardware-accelerated crypto with the CryptoCell: SHA-256, HMAC, AES-CCM and ChaCha20.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::aes::{Aes, AesCcm, AesCmac, Direction};
use embassy_nrf::chacha::ChaCha;
use embassy_nrf::hash::{Hash, Sha256};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Hashing and HMAC. Data can be fed in any number of chunks.
    let mut hash = Hash::new_blocking(p.HASH);
    let mut ctx = hash.start::<Sha256>();
    hash.blocking_update(&mut ctx, b"hello ");
    hash.blocking_update(&mut ctx, b"world");
    let digest = hash.blocking_finish(ctx);
    info!("sha256(\"hello world\") = {:02x}", digest);

    let mut ctx = hash.start_hmac::<Sha256>(b"secret key");
    hash.blocking_update(&mut ctx, b"hello world");
    let mac = hash.blocking_finish(ctx);
    info!("hmac-sha256 = {:02x}", mac);

    // AES-CCM authenticated encryption, then decryption.
    let mut aes = Aes::new_blocking(p.AES);
    let key = [0x42u8; 16];
    let nonce = [0x01u8; 13];
    let aad = b"header";
    let mut data = *b"attack at dawn";
    let cipher = unwrap!(AesCcm::new(&key, &nonce, aad.len(), data.len(), 8));

    let mut ctx = aes.start(cipher, Direction::Encrypt);
    unwrap!(aes.blocking_aad(&mut ctx, aad, true));
    unwrap!(aes.blocking_payload_in_place(&mut ctx, &mut data, true));
    let tag = unwrap!(unwrap!(aes.blocking_finish(ctx)));
    info!("ciphertext {:02x} tag {:02x}", data, tag[..8]);

    let mut ctx = aes.start(cipher, Direction::Decrypt);
    unwrap!(aes.blocking_aad(&mut ctx, aad, true));
    unwrap!(aes.blocking_payload_in_place(&mut ctx, &mut data, true));
    let computed = unwrap!(unwrap!(aes.blocking_finish(ctx)));
    // Compare the tag in constant time before trusting the plaintext.
    let ok = computed[..8].iter().zip(&tag[..8]).fold(0, |acc, (a, b)| acc | (a ^ b)) == 0;
    info!("plaintext {:a} tag ok {}", data, ok);

    // AES-CMAC.
    let mut ctx = aes.start(unwrap!(AesCmac::new(&key)), Direction::Encrypt);
    unwrap!(aes.blocking_payload(&mut ctx, b"message", &mut [], true));
    let mac = unwrap!(unwrap!(aes.blocking_finish(ctx)));
    info!("aes-cmac = {:02x}", mac);

    // ChaCha20 stream cipher.
    let mut chacha = ChaCha::new_blocking(p.CHACHA);
    let mut ctx = chacha.start(&[0x77u8; 32], &[0x11u8; 12], 1);
    let mut data = *b"some more data";
    chacha.blocking_apply_keystream_in_place(&mut ctx, &mut data);
    info!("chacha20 ciphertext {:02x}", data);
}
