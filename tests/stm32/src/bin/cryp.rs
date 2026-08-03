// required-features: cryp
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{AeadInOut, KeyInit};
use aes_gcm::aes::cipher::InOutBuf;
use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::cryp::*;
use panic_probe as _;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
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
    let iv: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    let mut ciphertext: [u8; PAYLOAD1.len() + PAYLOAD2.len()] = [0; PAYLOAD1.len() + PAYLOAD2.len()];
    let mut plaintext: [u8; PAYLOAD1.len() + PAYLOAD2.len()] = [0; PAYLOAD1.len() + PAYLOAD2.len()];

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
    let cipher = Aes128Gcm::new(&key.into());

    // Build AAD on the stack
    let mut aad = [0u8; AAD1.len() + AAD2.len()];
    aad[..AAD1.len()].copy_from_slice(AAD1);
    aad[AAD1.len()..].copy_from_slice(AAD2);

    // Build software payload buffer on the stack
    let mut sw_buf = [0u8; PAYLOAD1.len() + PAYLOAD2.len()];
    sw_buf[..PAYLOAD1.len()].copy_from_slice(PAYLOAD1);
    sw_buf[PAYLOAD1.len()..].copy_from_slice(PAYLOAD2);

    // Encrypt in-place; tag returned separately (no Buffer trait needed)
    let sw_tag = cipher
        .encrypt_inout_detached(&iv.into(), &aad, InOutBuf::from(&mut sw_buf[..]))
        .unwrap();

    defmt::assert!(ciphertext == sw_buf);

    // Explicit type annotation avoids ambiguous AsRef impls on hybrid_array::Array
    let encrypt_tag_slice: &[u8] = encrypt_tag.as_ref();
    let sw_tag_slice: &[u8] = sw_tag.as_ref();
    defmt::assert!(encrypt_tag_slice == sw_tag_slice);

    // Decrypt in-place; pass the tag back for verification
    cipher
        .decrypt_inout_detached(&iv.into(), &aad, InOutBuf::from(&mut sw_buf[..]), &sw_tag)
        .unwrap();

    defmt::assert!(PAYLOAD1 == &sw_buf[..PAYLOAD1.len()]);
    defmt::assert!(PAYLOAD2 == &sw_buf[PAYLOAD1.len()..]);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
