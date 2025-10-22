#![no_std]
#![no_main]

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::heapless::Vec;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::cryp::{self, *};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CRYP => cryp::InterruptHandler<peripherals::CRYP>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    let payload: &[u8] = b"hello world";
    let aad: &[u8] = b"additional data";

    let mut hw_cryp = Cryp::new(p.CRYP, p.DMA2_CH6, p.DMA2_CH5, Irqs);
    let key: [u8; 16] = [0; 16];
    let mut ciphertext: [u8; 11] = [0; 11];
    let mut plaintext: [u8; 11] = [0; 11];
    let iv: [u8; 12] = [0; 12];

    let hw_start_time = Instant::now();

    // Encrypt in hardware using AES-GCM 128-bit
    let aes_gcm = AesGcm::new(&key, &iv);
    let mut gcm_encrypt = hw_cryp.start(&aes_gcm, Direction::Encrypt).await;
    hw_cryp.aad(&mut gcm_encrypt, aad, true).await;
    hw_cryp.payload(&mut gcm_encrypt, payload, &mut ciphertext, true).await;
    let encrypt_tag = hw_cryp.finish(gcm_encrypt).await;

    // Decrypt in hardware using AES-GCM 128-bit
    let mut gcm_decrypt = hw_cryp.start(&aes_gcm, Direction::Decrypt).await;
    hw_cryp.aad(&mut gcm_decrypt, aad, true).await;
    hw_cryp
        .payload(&mut gcm_decrypt, &ciphertext, &mut plaintext, true)
        .await;
    let decrypt_tag = hw_cryp.finish(gcm_decrypt).await;

    let hw_end_time = Instant::now();
    let hw_execution_time = hw_end_time - hw_start_time;

    info!("AES-GCM Ciphertext: {:?}", ciphertext);
    info!("AES-GCM Plaintext: {:?}", plaintext);
    assert_eq!(payload, plaintext);
    assert_eq!(encrypt_tag, decrypt_tag);

    let sw_start_time = Instant::now();

    // Encrypt in software using AES-GCM 128-bit
    let mut payload_vec: Vec<u8, 32> = Vec::from_slice(&payload).unwrap();
    let cipher = Aes128Gcm::new(&key.into());
    let _ = cipher.encrypt_in_place(&iv.into(), aad.into(), &mut payload_vec);

    assert_eq!(ciphertext, payload_vec[0..ciphertext.len()]);
    assert_eq!(
        encrypt_tag,
        payload_vec[ciphertext.len()..ciphertext.len() + encrypt_tag.len()]
    );

    // Decrypt in software using AES-GCM 128-bit
    cipher
        .decrypt_in_place(&iv.into(), aad.into(), &mut payload_vec)
        .unwrap();

    let sw_end_time = Instant::now();
    let sw_execution_time = sw_end_time - sw_start_time;

    info!("Hardware Execution Time: {:?}", hw_execution_time);
    info!("Software Execution Time: {:?}", sw_execution_time);

    loop {}
}
