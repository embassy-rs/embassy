// required-features: hash
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::hash::*;
use embassy_stm32::{bind_interrupts, hash, peripherals};
use sha2::{Digest, Sha224, Sha256};
use {defmt_rtt as _, panic_probe as _};

#[cfg(any(
    feature = "stm32l4a6zg",
    feature = "stm32h755zi",
    feature = "stm32h753zi"
))]
bind_interrupts!(struct Irqs {
   HASH_RNG => hash::InterruptHandler<peripherals::HASH>;
});

#[cfg(any(
    feature = "stm32wba52cg",
    feature = "stm32l552ze",
    feature = "stm32h563zi",
    feature = "stm32u5a5zj",
    feature = "stm32u585ai"
))]
bind_interrupts!(struct Irqs {
    HASH => hash::InterruptHandler<peripherals::HASH>;
 });

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config());
    let dma = peri!(p, HASH_DMA);
    let mut hw_hasher = Hash::new(p.HASH, dma);

    let test_1: &[u8] = b"as;dfhaslfhas;oifvnasd;nifvnhasd;nifvhndlkfghsd;nvfnahssdfgsdafgsasdfasdfasdfasdfasdfghjklmnbvcalskdjghalskdjgfbaslkdjfgbalskdjgbalskdjbdfhsdfhsfghsfghfgh";
    let test_2: &[u8] = b"fdhalksdjfhlasdjkfhalskdjfhgal;skdjfgalskdhfjgalskdjfglafgadfgdfgdafgaadsfgfgdfgadrgsyfthxfgjfhklhjkfgukhulkvhlvhukgfhfsrghzdhxyfufynufyuszeradrtydyytserr";
    let test_3: &[u8] = b"a.ewtkluGWEBR.KAJRBTA,RMNRBG,FDMGB.kger.tkasjrbt.akrjtba.krjtba.ktmyna,nmbvtyliasd;gdrtba,sfvs.kgjzshd.gkbsr.tksejb.SDkfBSE.gkfgb>ESkfbSE>gkJSBESE>kbSE>fk";

    // Start an SHA-256 digest.
    let mut sha256context = hw_hasher.start(Algorithm::SHA256, DataType::Width8).await;
    hw_hasher.update(&mut sha256context, test_1).await;

    // Interrupt the SHA-256 digest to compute an SHA-224 digest.
    let mut sha224context = hw_hasher.start(Algorithm::SHA224, DataType::Width8).await;
    hw_hasher.update(&mut sha224context, test_3).await;
    let mut sha224_digest_buffer: [u8; 64] = [0; 64];
    let sha224_digest = hw_hasher.finish(sha224context, &mut sha224_digest_buffer).await;

    // Finish the SHA-256 digest.
    hw_hasher.update(&mut sha256context, test_2).await;
    let mut sha_256_digest_buffer: [u8; 64] = [0; 64];
    let sha256_digest = hw_hasher.finish(sha256context, &mut sha_256_digest_buffer).await;

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
    info!("Hardware SHA-256 Digest: {:?}", sha256_digest);
    info!("Software SHA-256 Digest: {:?}", sw_sha256_digest[..]);
    defmt::assert!(*sha256_digest == sw_sha256_digest[..]);

    // Compare the SHA-224 digests.
    info!("Hardware SHA-256 Digest: {:?}", sha224_digest);
    info!("Software SHA-256 Digest: {:?}", sw_sha224_digest[..]);
    defmt::assert!(*sha224_digest == sw_sha224_digest[..]);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
