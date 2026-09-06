// required-features: hash
#![no_std]
#![no_main]

//! The HASH peripheral through `embassy-crypto`'s API, cross-checked against software.

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_crypto::{HmacSha256, Sha1, Sha256};
use embassy_executor::Spawner;
use hmac::{Hmac as SoftwareHmac, KeyInit, Mac};
use panic_probe as _;
use sha2::{Digest, Sha256 as SoftwareSha256};

fn pattern(buf: &mut [u8], seed: u8) {
    let mut x = seed;
    for b in buf.iter_mut() {
        x = x.wrapping_mul(31).wrapping_add(7);
        *b = x;
    }
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let _p: embassy_stm32::Peripherals = init();

    let mut msg = [0u8; 1100];
    pattern(&mut msg, 1);

    info!("sha256, various lengths");
    for len in [0usize, 1, 3, 55, 56, 63, 64, 65, 127, 128, 129, 500, 1024, 1100] {
        let sw: [u8; 32] = SoftwareSha256::digest(&msg[..len]).into();
        defmt::assert_eq!(Sha256::digest(&msg[..len]), sw, "len {}", len);
    }

    info!("sha256, incremental with odd chunks, plus clone");
    let sw: [u8; 32] = SoftwareSha256::digest(&msg).into();
    let mut h = Sha256::new();
    for chunk in msg.chunks(37) {
        h.update(chunk);
    }
    let fork = h.clone();
    h.update(b"more");
    defmt::assert_eq!(fork.finalize(), sw);
    defmt::assert_ne!(h.finalize(), sw);

    info!("two hashers in flight, interleaved");
    let mut a = Sha256::new();
    let mut b = Sha1::new();
    for (i, chunk) in msg.chunks(100).enumerate() {
        if i % 2 == 0 {
            a.update(chunk);
            b.update(chunk);
        } else {
            b.update(chunk);
            a.update(chunk);
        }
    }
    let sw_sha1: [u8; 20] = sha1::Sha1::digest(&msg).into();
    defmt::assert_eq!(a.finalize(), sw);
    defmt::assert_eq!(b.finalize(), sw_sha1);

    info!("hmac-sha256, short and long keys");
    let mut key = [0u8; 200];
    pattern(&mut key, 0x55);
    for key_len in [1usize, 16, 32, 64, 65, 200] {
        for len in [0usize, 1, 64, 300] {
            let mut sw = SoftwareHmac::<SoftwareSha256>::new_from_slice(&key[..key_len]).unwrap();
            sw.update(&msg[..len]);
            let sw: [u8; 32] = sw.finalize().into_bytes().into();
            defmt::assert_eq!(
                HmacSha256::mac(&key[..key_len], &msg[..len]),
                sw,
                "key {} len {}",
                key_len,
                len
            );

            let mut m = HmacSha256::new(&key[..key_len]);
            m.update(&msg[..len]);
            m.verify(&sw).unwrap();
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
