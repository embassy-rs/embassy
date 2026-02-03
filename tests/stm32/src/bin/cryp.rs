// required-features: cryp
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::heapless::Vec;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::cryp::*;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();

    const PAYLOAD1: &[u8] = b"payload data 1 ;zdfhzdfhS;GKJASBDG;ASKDJBAL,zdfhzdfhzdfhzdfhvljhb,jhbjhb,sdhsdghsdhsfhsghzdfhzdfhzdfhzdfdhsdthsthsdhsgaadfhhgkdgfuoyguoft6783567";
    const PAYLOAD2: &[u8] = b"payload data 2 ;SKEzdfhzdfhzbhgvljhb,jhbjhb,sdhsdghsdhsfhsghshsfhshstsdthadfhsdfjhsfgjsfgjxfgjzdhgDFghSDGHjtfjtjszftjzsdtjhstdsdhsdhsdhsdhsdthsthsdhsgfh";
    const AAD1: &[u8] = b"additional data 1 stdargadrhaethaethjatjatjaetjartjstrjsfkk;'jopofyuisrteytweTASTUIKFUKIXTRDTEREharhaeryhaterjartjarthaethjrtjarthaetrhartjatejatrjsrtjartjyt1";
    const AAD2: &[u8] = b"additional data 2 stdhthsthsthsrthsrthsrtjdykjdukdyuldadfhsdghsdghsdghsadghjk'hioethjrtjarthaetrhartjatecfgjhzdfhgzdfhzdfghzdfhzdfhzfhjatrjsrtjartjytjfytjfyg";

    let in_dma = peri!(p, CRYP_IN_DMA);
    let out_dma = peri!(p, CRYP_OUT_DMA);
    let irq = irqs!(UART);

    let mut hw_cryp = Cryp::new(p.CRYP, in_dma, out_dma, irq);
    let key: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let mut ciphertext: [u8; PAYLOAD1.len() + PAYLOAD2.len()] = [0; PAYLOAD1.len() + PAYLOAD2.len()];
    let mut plaintext: [u8; PAYLOAD1.len() + PAYLOAD2.len()] = [0; PAYLOAD1.len() + PAYLOAD2.len()];
    let iv: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    // Encrypt in hardware using AES-GCM 128-bit in blocking mode.
    let aes_gcm = AesGcm::new(&key, &iv);
    let mut gcm_encrypt = hw_cryp.start_blocking(&aes_gcm, Direction::Encrypt);
    hw_cryp.aad_blocking(&mut gcm_encrypt, AAD1, false);
    hw_cryp.aad_blocking(&mut gcm_encrypt, AAD2, true);
    hw_cryp.payload_blocking(&mut gcm_encrypt, PAYLOAD1, &mut ciphertext[..PAYLOAD1.len()], false);
    hw_cryp.payload_blocking(&mut gcm_encrypt, PAYLOAD2, &mut ciphertext[PAYLOAD1.len()..], true);
    let encrypt_tag = hw_cryp.finish_blocking(gcm_encrypt);

    // Decrypt in hardware using AES-GCM 128-bit in async (DMA) mode.
    let mut gcm_decrypt = hw_cryp.start(&aes_gcm, Direction::Decrypt).await;
    hw_cryp.aad(&mut gcm_decrypt, AAD1, false).await;
    hw_cryp.aad(&mut gcm_decrypt, AAD2, true).await;
    hw_cryp
        .payload(&mut gcm_decrypt, &ciphertext, &mut plaintext, true)
        .await;
    let decrypt_tag = hw_cryp.finish(gcm_decrypt).await;

    info!("AES-GCM Ciphertext: {:?}", ciphertext);
    info!("AES-GCM Plaintext: {:?}", plaintext);
    defmt::assert!(PAYLOAD1 == &plaintext[..PAYLOAD1.len()]);
    defmt::assert!(PAYLOAD2 == &plaintext[PAYLOAD1.len()..]);
    defmt::assert!(encrypt_tag == decrypt_tag);

    // Encrypt in software using AES-GCM 128-bit
    let mut payload_vec: Vec<u8, { PAYLOAD1.len() + PAYLOAD2.len() + 16 }> = Vec::from_slice(&PAYLOAD1).unwrap();
    payload_vec.extend_from_slice(&PAYLOAD2).unwrap();
    let cipher = Aes128Gcm::new(&key.into());
    let mut aad: Vec<u8, { AAD1.len() + AAD2.len() }> = Vec::from_slice(&AAD1).unwrap();
    aad.extend_from_slice(&AAD2).unwrap();
    let _ = cipher.encrypt_in_place(&iv.into(), &aad, &mut payload_vec);

    defmt::assert!(ciphertext == payload_vec[0..ciphertext.len()]);
    defmt::assert!(encrypt_tag == payload_vec[ciphertext.len()..ciphertext.len() + encrypt_tag.len()]);

    // Decrypt in software using AES-GCM 128-bit
    cipher.decrypt_in_place(&iv.into(), &aad, &mut payload_vec).unwrap();

    info!("Test OK");
    cortex_m::asm::bkpt();
}
