// required-features: cryp
#![no_std]
#![no_main]

//! The CRYP peripheral through `embassy-crypto`'s API, cross-checked against software.

#[path = "../common.rs"]
mod common;
use aes_gcm::aead::{AeadInOut, KeyInit};
use aes_gcm::aes::cipher::{BlockCipherEncrypt, InOutBuf};
use common::*;
use defmt_rtt as _;
use embassy_crypto::{Aes128, Aes128CbcDecrypt, Aes128CbcEncrypt, Aes128Ccm, Aes128Ctr, Aes128Gcm, Aes256Gcm, Error};
use embassy_executor::Spawner;
use panic_probe as _;

fn pattern(buf: &mut [u8], seed: u8) {
    let mut x = seed;
    for b in buf.iter_mut() {
        x = x.wrapping_mul(31).wrapping_add(7);
        *b = x;
    }
}

/// Software AES-128 block encryption.
fn sw_block(key: &[u8; 16], block: &mut [u8; 16]) {
    let aes = aes_gcm::aes::Aes128::new(key.into());
    aes.encrypt_block(block.into());
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let _p: embassy_stm32::Peripherals = init();

    let mut key = [0u8; 16];
    pattern(&mut key, 3);
    let mut key256 = [0u8; 32];
    pattern(&mut key256, 4);
    let mut iv = [0u8; 16];
    pattern(&mut iv, 5);
    let mut pt = [0u8; 256];
    pattern(&mut pt, 6);

    info!("aes-128 ecb");
    let aes = Aes128::new(&key);
    let mut buf = pt;
    aes.encrypt_blocks(&mut buf).unwrap();
    for (i, chunk) in buf.chunks(16).enumerate() {
        let mut sw: [u8; 16] = pt[i * 16..i * 16 + 16].try_into().unwrap();
        sw_block(&key, &mut sw);
        defmt::assert_eq!(chunk, &sw, "block {}", i);
    }
    aes.decrypt_blocks(&mut buf).unwrap();
    defmt::assert_eq!(buf, pt);
    let mut out = [0u8; 256];
    aes.encrypt_blocks_to(&pt, &mut out).unwrap();
    aes.decrypt_blocks(&mut out).unwrap();
    defmt::assert_eq!(out, pt);
    defmt::assert_eq!(aes.encrypt_blocks(&mut buf[..15]), Err(Error::InvalidInput));

    info!("aes-128 cbc");
    let mut buf = pt;
    let mut enc = Aes128CbcEncrypt::new(&key, &iv);
    enc.encrypt(&mut buf[..64]).unwrap();
    enc.encrypt(&mut buf[64..]).unwrap();
    // CBC's first block is E(P0 ^ IV), and each later block chains on the previous ciphertext.
    let mut prev = iv;
    for (i, chunk) in buf.chunks(16).enumerate() {
        let mut sw = [0u8; 16];
        for j in 0..16 {
            sw[j] = pt[i * 16 + j] ^ prev[j];
        }
        sw_block(&key, &mut sw);
        defmt::assert_eq!(chunk, &sw, "block {}", i);
        prev.copy_from_slice(chunk);
    }
    let mut dec = Aes128CbcDecrypt::new(&key, &iv);
    dec.decrypt(&mut buf[..32]).unwrap();
    dec.decrypt(&mut buf[32..]).unwrap();
    defmt::assert_eq!(buf, pt);

    info!("aes-128 ctr");
    let mut buf = pt;
    let mut ctr = Aes128Ctr::new(&key, &iv);
    let mut pos = 0;
    for len in [1usize, 7, 16, 20, 3, 17, 100] {
        ctr.apply_keystream(&mut buf[pos..pos + len]);
        pos += len;
    }
    ctr.apply_keystream(&mut buf[pos..]);
    let mut counter = iv;
    for (i, chunk) in buf.chunks(16).enumerate() {
        let mut ks = counter;
        sw_block(&key, &mut ks);
        for j in 0..16 {
            defmt::assert_eq!(chunk[j], pt[i * 16 + j] ^ ks[j], "block {} byte {}", i, j);
        }
        for b in counter.iter_mut().rev() {
            *b = b.wrapping_add(1);
            if *b != 0 {
                break;
            }
        }
    }
    Aes128Ctr::new(&key, &iv).apply_keystream(&mut buf);
    defmt::assert_eq!(buf, pt);

    info!("aes-128 gcm");
    let nonce: [u8; 12] = iv[..12].try_into().unwrap();
    let aad = b"additional authenticated data";
    let gcm = Aes128Gcm::new(&key);
    let mut buf = [0u8; 100];
    buf.copy_from_slice(&pt[..100]);
    let tag = gcm.encrypt(&nonce, aad, &mut buf).unwrap();
    let sw = aes_gcm::Aes128Gcm::new(&key.into());
    let mut sw_buf = [0u8; 100];
    sw_buf.copy_from_slice(&pt[..100]);
    let sw_tag = sw
        .encrypt_inout_detached(&nonce.into(), aad, InOutBuf::from(&mut sw_buf[..]))
        .unwrap();
    defmt::assert_eq!(buf, sw_buf);
    defmt::assert_eq!(tag, sw_tag.as_slice());
    gcm.decrypt(&nonce, aad, &mut buf, &tag).unwrap();
    defmt::assert_eq!(buf, pt[..100]);
    buf[7] ^= 1;
    defmt::assert_eq!(gcm.decrypt(&nonce, aad, &mut buf, &tag), Err(Error::InvalidSignature));

    info!("aes-256 gcm");
    let gcm = Aes256Gcm::new(&key256);
    let mut buf = [0u8; 100];
    buf.copy_from_slice(&pt[..100]);
    let tag = gcm.encrypt(&nonce, aad, &mut buf).unwrap();
    let sw = aes_gcm::Aes256Gcm::new(&key256.into());
    let mut sw_buf = [0u8; 100];
    sw_buf.copy_from_slice(&pt[..100]);
    let sw_tag = sw
        .encrypt_inout_detached(&nonce.into(), aad, InOutBuf::from(&mut sw_buf[..]))
        .unwrap();
    defmt::assert_eq!(buf, sw_buf);
    defmt::assert_eq!(tag, sw_tag.as_slice());
    gcm.decrypt(&nonce, aad, &mut buf, &tag).unwrap();
    defmt::assert_eq!(buf, pt[..100]);

    info!("aes-128 ccm");
    let ccm = Aes128Ccm::new(&key);
    let nonce13 = &iv[..13];
    let mut buf = [0u8; 100];
    buf.copy_from_slice(&pt[..100]);
    let mut tag = [0u8; 8];
    ccm.encrypt(nonce13, aad, &mut buf, &mut tag).unwrap();
    defmt::assert_ne!(buf, pt[..100]);
    ccm.decrypt(nonce13, aad, &mut buf, &tag).unwrap();
    defmt::assert_eq!(buf, pt[..100]);
    tag[0] ^= 1;
    defmt::assert_eq!(ccm.decrypt(nonce13, aad, &mut buf, &tag), Err(Error::InvalidSignature));

    info!("Test OK");
    cortex_m::asm::bkpt();
}
