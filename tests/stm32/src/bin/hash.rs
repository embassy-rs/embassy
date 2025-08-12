// required-features: hash
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::hash::*;
use embassy_stm32::{bind_interrupts, hash, peripherals};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha224, Sha256};
use {defmt_rtt as _, panic_probe as _};

type HmacSha256 = Hmac<Sha256>;

#[cfg(any(feature = "stm32l4a6zg", feature = "stm32h755zi", feature = "stm32h753zi"))]
bind_interrupts!(struct Irqs {
   HASH_RNG => hash::InterruptHandler<peripherals::HASH>;
});

#[cfg(any(
    feature = "stm32wba52cg",
    feature = "stm32l552ze",
    feature = "stm32h563zi",
    feature = "stm32h503rb",
    feature = "stm32u5a5zj",
    feature = "stm32u585ai",
    feature = "stm32h7s3l8"
))]
bind_interrupts!(struct Irqs {
    HASH => hash::InterruptHandler<peripherals::HASH>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();
    let mut hw_hasher = Hash::new_blocking(p.HASH, Irqs);

    let test_1: &[u8] = b"as;dfhaslfhas;oifvnasd;nifvnhasd;nifvhndlkfghsd;nvfnahssdfgsdafgsasdfasdfasdfasdfasdfghjklmnbvcalskdjghalskdjgfbaslkdjfgbalskdjgbalskdjbdfhsdfhsfghsfghfgh";
    let test_2: &[u8] = b"fdhalksdjfhlasdjkfhalskdjfhgal;skdjfgalskdhfjgalskdjfglafgadfgdfgdafgaadsfgfgdfgadrgsyfthxfgjfhklhjkfgukhulkvhlvhukgfhfsrghzdhxyfufynufyuszeradrtydyytserr";
    let test_3: &[u8] = b"a.ewtkluGWEBR.KAJRBTA,RMNRBG,FDMGB.kger.tkasjrbt.akrjtba.krjtba.ktmyna,nmbvtyliasd;gdrtba,sfvs.kgjzshd.gkbsr.tksejb.SDkfBSE.gkfgb>ESkfbSE>gkJSBESE>kbSE>fk";

    // Start an SHA-256 digest.
    let mut sha256context = hw_hasher.start(Algorithm::SHA256, DataType::Width8, None);
    hw_hasher.update_blocking(&mut sha256context, test_1);

    // Interrupt the SHA-256 digest to compute an SHA-224 digest.
    let mut sha224context = hw_hasher.start(Algorithm::SHA224, DataType::Width8, None);
    hw_hasher.update_blocking(&mut sha224context, test_3);
    let mut sha224_digest_buffer: [u8; 28] = [0; 28];
    let _ = hw_hasher.finish_blocking(sha224context, &mut sha224_digest_buffer);

    // Finish the SHA-256 digest.
    hw_hasher.update_blocking(&mut sha256context, test_2);
    let mut sha256_digest_buffer: [u8; 32] = [0; 32];
    let _ = hw_hasher.finish_blocking(sha256context, &mut sha256_digest_buffer);

    // Compute the SHA-256 digest in software.
    let mut sw_sha256_hasher = Sha256::new();
    sw_sha256_hasher.update(test_1);
    sw_sha256_hasher.update(test_2);
    let sw_sha256_digest = sw_sha256_hasher.finalize();

    //Compute the SHA-224 digest in software.
    let mut sw_sha224_hasher = Sha224::new();
    sw_sha224_hasher.update(test_3);
    let sw_sha224_digest = sw_sha224_hasher.finalize();

    // Compare the SHA-256 digests.
    info!("Hardware SHA-256 Digest: {:?}", sha256_digest_buffer);
    info!("Software SHA-256 Digest: {:?}", sw_sha256_digest[..]);
    defmt::assert!(sha256_digest_buffer == sw_sha256_digest[..]);

    // Compare the SHA-224 digests.
    info!("Hardware SHA-256 Digest: {:?}", sha224_digest_buffer);
    info!("Software SHA-256 Digest: {:?}", sw_sha224_digest[..]);
    defmt::assert!(sha224_digest_buffer == sw_sha224_digest[..]);

    let hmac_key: [u8; 64] = [0x55; 64];

    // Compute HMAC in hardware.
    let mut sha256hmac_context = hw_hasher.start(Algorithm::SHA256, DataType::Width8, Some(&hmac_key));
    hw_hasher.update_blocking(&mut sha256hmac_context, test_1);
    hw_hasher.update_blocking(&mut sha256hmac_context, test_2);
    let mut hw_hmac: [u8; 32] = [0; 32];
    hw_hasher.finish_blocking(sha256hmac_context, &mut hw_hmac);

    // Compute HMAC in software.
    let mut sw_mac = HmacSha256::new_from_slice(&hmac_key).unwrap();
    sw_mac.update(test_1);
    sw_mac.update(test_2);
    let sw_hmac = sw_mac.finalize().into_bytes();

    info!("Hardware HMAC: {:?}", hw_hmac);
    info!("Software HMAC: {:?}", sw_hmac[..]);
    defmt::assert!(hw_hmac == sw_hmac[..]);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
