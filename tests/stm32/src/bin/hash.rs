// required-features: hash
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::hash::*;
use embassy_stm32::mode::Blocking;
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

fn test_interrupt(hw_hasher: &mut Hash<'_, peripherals::HASH, Blocking>) {
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
}

// This uses sha512, so only supported on hash_v3 and up
#[cfg(feature = "hash-v34")]
fn test_sizes(hw_hasher: &mut Hash<'_, peripherals::HASH, Blocking>) {
    let in1 = b"4BPuGudaDK";
    let in2 = b"cfFIGf0XSNhFBQ5LaIqzjnRKDRkoWweJI06HLUcicIUGjpuDNfOTQNSrRxDoveDPlazeZtt06SIYO5CvHvsJ98XSfO9yJEMHoDpDAmNQtwZOPlKmdiagRXsJ7w7IjdKpQH6I2t";

    for i in 1..10 {
        // sha512 block size is 128, so test around there
        for j in [1, 1, 2, 3, 4, 5, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133] {
            info!("test_sizes i {} j {}", i, j);
            let mut sw = sha2::Sha512::new();
            let mut ctx = hw_hasher.start(Algorithm::SHA512, DataType::Width8, None);

            sw.update(&in1[..i]);
            sw.update(&in2[..j]);
            hw_hasher.update_blocking(&mut ctx, &in1[..i]);
            hw_hasher.update_blocking(&mut ctx, &in2[..j]);

            let sw_digest = sw.finalize();
            let mut hw_digest = [0u8; 64];
            hw_hasher.finish_blocking(ctx, &mut hw_digest);
            info!("Hardware: {:?}", hw_digest);
            info!("Software: {:?}", sw_digest[..]);
            defmt::assert!(hw_digest == *sw_digest);
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();
    let mut hw_hasher = Hash::new_blocking(p.HASH, Irqs);

    test_interrupt(&mut hw_hasher);
    // Run it a second time to check hash-after-hmac
    test_interrupt(&mut hw_hasher);

    #[cfg(feature = "hash-v34")]
    test_sizes(&mut hw_hasher);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
