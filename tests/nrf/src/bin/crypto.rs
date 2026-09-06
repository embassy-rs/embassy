// required-features: crypto
#![no_std]
#![no_main]

//! Exercises the hardware accelerators through `embassy-crypto`'s API.

#[path = "../common.rs"]
mod common;
#[path = "../crypto_vectors.rs"]
mod vectors;

use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_crypto::{
    Aes128, Aes128CbcDecrypt, Aes128CbcEncrypt, Aes128Ccm, Aes128Cmac, Aes128Ctr, HmacSha256, Sha1, Sha256,
};
use embassy_executor::Spawner;
use panic_probe as _;
use vectors::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());

    let mut msg = [0u8; 2049];
    pattern(&mut msg, 1);

    info!("digest");
    for &(len, ref digest) in SHA256 {
        assert_eq!(Sha256::digest(&msg[..len]), *digest);
        let mut h = Sha256::new();
        for chunk in msg[..len].chunks(37) {
            h.update(chunk);
        }
        assert_eq!(h.finalize(), *digest);
    }
    for &(len, ref digest) in SHA1 {
        assert_eq!(Sha1::digest(&msg[..len]), *digest);
    }

    info!("two hashers in flight, interleaved, plus clone");
    let mut a = Sha256::new();
    let mut b = Sha1::new();
    for chunk in msg[..1000].chunks(100) {
        a.update(chunk);
        b.update(chunk);
    }
    let fork = a.clone();
    a.update(b"more");
    let (_, sha256_1000) = unwrap!(SHA256.iter().find(|(len, _)| *len == 1000));
    let (_, sha1_1000) = unwrap!(SHA1.iter().find(|(len, _)| *len == 1000));
    assert_eq!(fork.finalize(), *sha256_1000);
    assert_eq!(b.finalize(), *sha1_1000);
    assert_ne!(a.finalize(), *sha256_1000);

    info!("hmac");
    let mut key = [0u8; 200];
    pattern(&mut key, 0x55);
    let mut msg2 = [0u8; 1000];
    pattern(&mut msg2, 2);
    for &(key_len, len, ref digest) in HMAC_SHA256 {
        let mut mac = HmacSha256::new(&key[..key_len]);
        mac.update(&msg2[..len]);
        assert_eq!(mac.finalize(), *digest);
        assert_eq!(HmacSha256::mac(&key[..key_len], &msg2[..len]), *digest);
        let mut mac = HmacSha256::new(&key[..key_len]);
        mac.update(&msg2[..len]);
        unwrap!(mac.verify(digest));
    }

    info!("aes ecb");
    let mut pt = [0u8; 1024];
    pattern(&mut pt, 3);
    let aes = Aes128::new(&KEY128);
    let mut buf = pt;
    unwrap!(aes.encrypt_blocks(&mut buf));
    assert_eq!(buf, AES128_ECB);
    unwrap!(aes.decrypt_blocks(&mut buf));
    assert_eq!(buf, pt);
    let mut out = [0u8; 1024];
    unwrap!(aes.encrypt_blocks_to(&pt, &mut out));
    assert_eq!(out, AES128_ECB);

    info!("aes cbc");
    let mut enc = Aes128CbcEncrypt::new(&KEY128, &CBC_IV);
    let mut buf = pt;
    unwrap!(enc.encrypt(&mut buf[..160]));
    unwrap!(enc.encrypt(&mut buf[160..]));
    assert_eq!(buf, AES128_CBC);
    let mut dec = Aes128CbcDecrypt::new(&KEY128, &CBC_IV);
    unwrap!(dec.decrypt(&mut buf[..16]));
    unwrap!(dec.decrypt(&mut buf[16..]));
    assert_eq!(buf, pt);

    info!("aes ctr");
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 5);
    let mut buf = pt;
    let mut ctr = Aes128Ctr::new(&KEY128, &CTR_IV0);
    ctr.apply_keystream(&mut buf[..1]);
    ctr.apply_keystream(&mut buf[1..100]);
    ctr.apply_keystream(&mut buf[100..]);
    assert_eq!(buf, AES128_CTR0);
    Aes128Ctr::new(&KEY128, &CTR_IV0).apply_keystream(&mut buf);
    assert_eq!(buf, pt);

    info!("aes cmac");
    let mut msg6 = [0u8; 1024];
    pattern(&mut msg6, 6);
    for &(len, ref tag) in AES128_CMAC {
        let mut mac = Aes128Cmac::new(&KEY128);
        for chunk in msg6[..len].chunks(13) {
            mac.update(chunk);
        }
        assert_eq!(mac.finalize(), *tag);
    }

    info!("aes ccm");
    let mut nonce = [0u8; 13];
    pattern(&mut nonce, 7);
    // Some vectors carry more associated data than fits on the stack.
    static mut AAD: [u8; 65285] = [0; 65285];
    let aad: &mut [u8] = unsafe { &mut *core::ptr::addr_of_mut!(AAD) };
    pattern(aad, 8);
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 9);
    let ccm = Aes128Ccm::new(&KEY128);
    for v in AES128_CCM {
        let mut buf = [0u8; 1000];
        buf[..v.pt_len].copy_from_slice(&pt[..v.pt_len]);
        let mut tag = [0u8; 16];
        unwrap!(ccm.encrypt(
            &nonce[..v.nonce_len],
            &aad[..v.aad_len],
            &mut buf[..v.pt_len],
            &mut tag[..v.tag_len]
        ));
        assert_eq!(buf[..v.pt_len], v.ct[..]);
        assert_eq!(tag[..v.tag_len], v.tag[..]);
        unwrap!(ccm.decrypt(
            &nonce[..v.nonce_len],
            &aad[..v.aad_len],
            &mut buf[..v.pt_len],
            &tag[..v.tag_len]
        ));
        assert_eq!(buf[..v.pt_len], pt[..v.pt_len]);
        tag[0] ^= 1;
        assert!(
            ccm.decrypt(
                &nonce[..v.nonce_len],
                &aad[..v.aad_len],
                &mut buf[..v.pt_len],
                &tag[..v.tag_len]
            )
            .is_err()
        );
    }

    #[cfg(feature = "aes256")]
    {
        info!("aes gcm");
        let gcm = embassy_crypto::Aes128Gcm::new(&KEY128);
        let nonce: [u8; 12] = nonce[..12].try_into().unwrap();
        for v in AES128_GCM {
            let mut buf = [0u8; 1000];
            buf[..v.pt_len].copy_from_slice(&pt[..v.pt_len]);
            let tag = unwrap!(gcm.encrypt(&nonce, &aad[..v.aad_len], &mut buf[..v.pt_len]));
            assert_eq!(buf[..v.pt_len], v.ct[..]);
            assert_eq!(tag, v.tag);
            unwrap!(gcm.decrypt(&nonce, &aad[..v.aad_len], &mut buf[..v.pt_len], &tag));
            assert_eq!(buf[..v.pt_len], pt[..v.pt_len]);
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
