// required-features: hash
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::hash::*;
use embassy_stm32::mode::Blocking;
use embassy_stm32::{bind_interrupts, hash, peripherals};
use hmac::{Hmac as SoftwareHmac, KeyInit, Mac};
use panic_probe as _;
use sha2::{Digest, Sha224 as SoftwareSha224, Sha256 as SoftwareSha256};

type HmacSha256 = SoftwareHmac<SoftwareSha256>;

#[cfg(any(feature = "stm32l4a6zg", feature = "stm32h755zi", feature = "stm32h753zi"))]
bind_interrupts!(struct Irqs {
   HASH_RNG => hash::InterruptHandler<peripherals::HASH>;
});

#[cfg(any(
    feature = "stm32wba52cg",
    feature = "stm32wba65ri",
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

    info!("Hardware start");

    // Start an SHA-256 digest.
    let mut sha256context = hw_hasher.start::<Sha256, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut sha256context, test_1);

    // Interrupt the SHA-256 digest to compute an SHA-224 digest.
    let mut sha224context = hw_hasher.start::<Sha224, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut sha224context, test_3);
    let mut sha224_digest_buffer: [u8; 28] = [0; 28];
    let _ = hw_hasher.finish_blocking(sha224context, &mut sha224_digest_buffer);

    // Finish the SHA-256 digest.
    hw_hasher.update_blocking(&mut sha256context, test_2);
    let mut sha256_digest_buffer: [u8; 32] = [0; 32];
    let _ = hw_hasher.finish_blocking(sha256context, &mut sha256_digest_buffer);

    info!("Hardware stop");
    info!("Software start");

    // Compute the SHA-256 digest in software.
    let mut sw_sha256_hasher = SoftwareSha256::new();
    sw_sha256_hasher.update(test_1);
    sw_sha256_hasher.update(test_2);
    let sw_sha256_digest = sw_sha256_hasher.finalize();

    //Compute the SHA-224 digest in software.
    let mut sw_sha224_hasher = SoftwareSha224::new();
    sw_sha224_hasher.update(test_3);
    let sw_sha224_digest = sw_sha224_hasher.finalize();

    info!("Software stop");

    // Compare the SHA-256 digests.
    info!("Hardware SHA-256 Digest: {:?}", sha256_digest_buffer);
    info!("Software SHA-256 Digest: {:?}", sw_sha256_digest[..]);
    defmt::assert!(sha256_digest_buffer == sw_sha256_digest[..]);

    // Compare the SHA-224 digests.
    info!("Hardware SHA-256 Digest: {:?}", sha224_digest_buffer);
    info!("Software SHA-256 Digest: {:?}", sw_sha224_digest[..]);
    defmt::assert!(sha224_digest_buffer == sw_sha224_digest[..]);

    let hmac_key: [u8; 64] = [0x55; 64];

    info!("Hardware start");

    // Compute HMAC in hardware.
    let mut sha256hmac_context = hw_hasher.start::<Sha256, Hmac>(DataType::Width8, Some(&hmac_key));
    hw_hasher.update_blocking(&mut sha256hmac_context, test_1);
    hw_hasher.update_blocking(&mut sha256hmac_context, test_2);
    let mut hw_hmac: [u8; 32] = [0; 32];
    hw_hasher.finish_blocking(sha256hmac_context, &mut hw_hmac);

    info!("Hardware stop");
    info!("Software start");

    // Compute HMAC in software.
    let mut sw_mac = HmacSha256::new_from_slice(&hmac_key).unwrap();
    sw_mac.update(test_1);
    sw_mac.update(test_2);
    let sw_hmac = sw_mac.finalize().into_bytes();

    info!("Software stop");

    info!("Hardware HMAC: {:?}", hw_hmac);
    info!("Software HMAC: {:?}", sw_hmac[..]);
    defmt::assert!(hw_hmac == sw_hmac[..]);

    let long_hmac_key = [0x37; 100];
    let mut sha256hmac_context = hw_hasher.start::<Sha256, Hmac>(DataType::Width8, Some(&long_hmac_key));
    hw_hasher.update_blocking(&mut sha256hmac_context, test_1);
    hw_hasher.update_blocking(&mut sha256hmac_context, test_2);
    let mut hw_hmac: [u8; 32] = [0; 32];
    hw_hasher.finish_blocking(sha256hmac_context, &mut hw_hmac);

    let mut sw_mac = HmacSha256::new_from_slice(&long_hmac_key).unwrap();
    sw_mac.update(test_1);
    sw_mac.update(test_2);
    let sw_hmac = sw_mac.finalize().into_bytes();
    defmt::assert!(hw_hmac == sw_hmac[..]);
}

/// Regression test for the DINIS stall seen in the embedded-tls handshake on
/// STM32H563: interleaved contexts where the old feed pattern left an
/// untriggered block in the IN buffer, so `store_context` spun on DINIS
/// forever (deterministic, because TLS record sizes are fixed). The exact
/// update-size pattern from the hang is updates of 6, then 367, then 79
/// bytes on one context, interleaved with unrelated digests. On the old
/// driver this either stalls or (if words were silently lost) fails the
/// digest comparison; both are caught here.
fn test_dinis_stall_regression(hw_hasher: &mut Hash<'_, peripherals::HASH, Blocking>) {
    let mut data = [0u8; 512];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i * 31 + 7) as u8;
    }

    // Context A: the TLS transcript-hash pattern.
    let mut sw_a = SoftwareSha256::new();
    sw_a.update(&data[..6]);
    sw_a.update(&data[100..467]); // 367 bytes
    sw_a.update(&data[200..279]); // 79 bytes
    let expected_a = sw_a.finalize();

    let mut ctx_a = hw_hasher.start::<Sha256, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut ctx_a, &data[..6]);

    // Interleave a complete unrelated digest between updates of ctx_a.
    let mut sw_b = SoftwareSha224::new();
    sw_b.update(&data[300..413]); // 113 bytes
    let expected_b = sw_b.finalize();

    let mut ctx_b = hw_hasher.start::<Sha224, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut ctx_b, &data[300..413]);
    let mut b_digest = [0u8; 28];
    hw_hasher.finish_blocking(ctx_b, &mut b_digest);
    defmt::assert!(b_digest == expected_b[..]);

    // This is the update whose store left the stuck IN buffer on the old driver.
    hw_hasher.update_blocking(&mut ctx_a, &data[100..467]);

    // Interleave again, this time a SHA-256 context fed exactly one quantum
    // (68 bytes) followed by one full block (64 bytes).
    let mut sw_c = SoftwareSha256::new();
    sw_c.update(&data[400..468]);
    sw_c.update(&data[0..64]);
    let expected_c = sw_c.finalize();

    let mut ctx_c = hw_hasher.start::<Sha256, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut ctx_c, &data[400..468]);
    hw_hasher.update_blocking(&mut ctx_c, &data[0..64]);
    let mut c_digest = [0u8; 32];
    hw_hasher.finish_blocking(ctx_c, &mut c_digest);
    defmt::assert!(c_digest == expected_c[..]);

    // Final update of ctx_a: used to hang in store_context here.
    hw_hasher.update_blocking(&mut ctx_a, &data[200..279]);
    let mut a_digest = [0u8; 32];
    hw_hasher.finish_blocking(ctx_a, &mut a_digest);
    defmt::assert!(a_digest == expected_a[..]);
}

/// Sweep first/second update sizes across the NBWE-quantum (block + 4 bytes)
/// and block boundaries, interleaving an unrelated SHA-224 digest between
/// the two updates of the SHA-256 context. Catches both the DINIS stall and
/// silent word loss from overrunning the IN buffer.
fn test_boundary_sizes(hw_hasher: &mut Hash<'_, peripherals::HASH, Blocking>) {
    let mut data = [0u8; 512];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i * 13 + 5) as u8;
    }

    // Sizes straddling 63/64/65 (block), 67/68/69 (quantum = block + 4) and
    // larger multiples: 132 = 2 quanta, 136 = 2 quanta + block, etc.
    let sizes = [
        1usize, 2, 3, 4, 5, 6, 15, 16, 17, 31, 32, 33, 47, 48, 49, 63, 64, 65, 66, 67, 68, 69, 70, 79, 96, 127, 128,
        129, 130, 131, 132, 133, 134, 135, 136, 137, 255, 256, 257, 319, 367, 511,
    ];
    let splits = [1usize, 63, 64, 68, 79, 132];

    for &n in &sizes {
        for &m in &splits {
            info!("boundary n={} m={}", n, m);

            let mut ctx_a = hw_hasher.start::<Sha256, NonHmac>(DataType::Width8, None);
            hw_hasher.update_blocking(&mut ctx_a, &data[..n]);

            let b_off = 256 - m / 2;
            let mut ctx_b = hw_hasher.start::<Sha224, NonHmac>(DataType::Width8, None);
            hw_hasher.update_blocking(&mut ctx_b, &data[b_off..b_off + m]);
            let mut b_digest = [0u8; 28];
            hw_hasher.finish_blocking(ctx_b, &mut b_digest);

            hw_hasher.update_blocking(&mut ctx_a, &data[100..100 + m]);

            let mut a_digest = [0u8; 32];
            hw_hasher.finish_blocking(ctx_a, &mut a_digest);

            let mut sw_a = SoftwareSha256::new();
            sw_a.update(&data[..n]);
            sw_a.update(&data[100..100 + m]);
            let mut sw_b = SoftwareSha224::new();
            sw_b.update(&data[b_off..b_off + m]);

            defmt::assert!(a_digest == sw_a.finalize()[..]);
            defmt::assert!(b_digest == sw_b.finalize()[..]);
        }
    }
}

// This uses sha512, so only supported on hash_v3 and up
// The HASH of the STM32H503 has no SHA-384/512.
#[cfg(all(feature = "hash-v34", not(feature = "stm32h503rb")))]
fn test_sizes(hw_hasher: &mut Hash<'_, peripherals::HASH, Blocking>) {
    let in1 = b"4BPuGudaDK";
    let in2 = b"cfFIGf0XSNhFBQ5LaIqzjnRKDRkoWweJI06HLUcicIUGjpuDNfOTQNSrRxDoveDPlazeZtt06SIYO5CvHvsJ98XSfO9yJEMHoDpDAmNQtwZOPlKmdiagRXsJ7w7IjdKpQH6I2t";

    for i in 1..10 {
        // sha512 block size is 128, so test around there
        for j in [1, 1, 2, 3, 4, 5, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133] {
            info!("test_sizes i {} j {}", i, j);
            let mut sw = sha2::Sha512::new();
            let mut ctx = hw_hasher.start::<Sha512, NonHmac>(DataType::Width8, None);

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

    let boundary_input = [0u8; 255];
    let mut sw = sha2::Sha512::new();
    let mut ctx = hw_hasher.start::<Sha512, NonHmac>(DataType::Width8, None);
    sw.update(&boundary_input);
    hw_hasher.update_blocking(&mut ctx, &boundary_input);
    let sw_digest = sw.finalize();
    let mut hw_digest = [0u8; 64];
    hw_hasher.finish_blocking(ctx, &mut hw_digest);
    defmt::assert!(hw_digest == *sw_digest);
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();
    let mut hw_hasher = Hash::new_blocking(p.HASH, Irqs);

    test_interrupt(&mut hw_hasher);
    // Run it a second time to check hash-after-hmac
    test_interrupt(&mut hw_hasher);

    test_dinis_stall_regression(&mut hw_hasher);
    test_boundary_sizes(&mut hw_hasher);

    // The HASH of the STM32H503 has no SHA-384/512.
    #[cfg(all(feature = "hash-v34", not(feature = "stm32h503rb")))]
    test_sizes(&mut hw_hasher);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
