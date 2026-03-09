#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::config::Config;
use hal::dma::DmaChannel;
use hal::sgi::hash::{HashSize, SGIHasher, hash_blocking, hash_dma_start};
use hal::sgi::{Config as SgiConfig, InterruptHandler as SgiInterruptHandler, Sgi};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        SGI => SgiInterruptHandler;
    }
);

// Note: SGI is identical among all MCXA MCUs.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    defmt::info!("SGI example");

    // Take DMA channel ownership, will use for DMA copy.
    let mut dma_ch0 = DmaChannel::new(p.DMA_CH0.reborrow());

    let mut sgi = Sgi::new(
        // Use reborrowed token so sgi can be explicitly dropped and recreated
        // within the same scope (pattern used in the Embassy TRNG example).
        p.SGI0.reborrow(),
        Irqs,
        SgiConfig {
            enable_interrupt: false,
        },
    )
    .unwrap();
    const MAX_LEN: usize = 32 * 1024;
    let mut input_data = [0u8; MAX_LEN];

    for i in 0..input_data.len() {
        input_data[i] = i as u8;
    }

    let mut hash_result: [u8; 64] = [0u8; 64];

    // Hash blocking will block until complete, max input size limited to 512 bytes.
    match hash_blocking(&mut sgi, HashSize::Sha384, &input_data[..308], &mut hash_result) {
        Ok(_) => defmt::info!("Hash output blocking: {=[u8]:x}", &hash_result[..48]), // SHA-384 is 48 bytes or 384 bits
        Err(_) => defmt::error!("Block Hashing failed"),
    }

    match hash_blocking(&mut sgi, HashSize::Sha512, &input_data[..308], &mut hash_result) {
        Ok(_) => defmt::info!("Hash output blocking: {=[u8]:x}", &hash_result[..64]), // SHA-512 is 64 bytes or 512 bits
        Err(_) => defmt::error!("Block Hashing failed"),
    }

    drop(sgi);

    let mut sgi = Sgi::new(
        // Use reborrowed token so sgi can be explicitly dropped and recreated
        // within the same scope (pattern used in the Embassy TRNG example).
        p.SGI0.reborrow(),
        Irqs,
        SgiConfig {
            enable_interrupt: false,
        },
    )
    .unwrap();

    //streaming mode:
    let mut hasher = SGIHasher::new();
    // Init takes two inputs, size and mode. Unless there are specific concerns about memory/performance, leave mode as None.
    let _ = hasher.init(HashSize::Sha512, None).unwrap();
    let mut index = 0;
    // hasher.update has a max input size of 512 bytes per update call.
    for _ in 0..3 {
        let _ = hasher.update(&mut sgi, &input_data[index..index + 128]).unwrap();
        index += 128; // doing one full block at a time.
    }

    for _ in 0..3 {
        let _ = hasher.update(&mut sgi, &input_data[index..index + 50]).unwrap();
        index += 50; // doing less than a full block.
    }

    let _ = hasher.update(&mut sgi, &input_data[index..index + 512]).unwrap(); //max size.
    //index += 512;

    // Call hasher.finalize to get the final hash result
    let _ = hasher.finalize(&mut sgi, &mut hash_result[..64]).unwrap();

    defmt::info!("Hash output streaming: {=[u8]:x}", &hash_result[..64]);

    drop(sgi);

    let mut sgi = Sgi::new(
        // Use reborrowed token so sgi can be explicitly dropped and recreated
        // within the same scope (pattern used in the Embassy TRNG example).
        p.SGI0.reborrow(),
        Irqs,
        SgiConfig {
            enable_interrupt: true, // enable for DMA mode.
        },
    )
    .unwrap();

    // Async mode with DMA transfer. no upper size limit, except 2^32 bytes as imposed by SGI itself.
    // Takes an initialized and acquired SGI instance and DMA channel instance, size and data to be hashed.
    // Returns a DmaHasher instance which can be awaited completion asynchronously (interrupts)
    match hash_dma_start(&mut sgi, &mut dma_ch0, HashSize::Sha384, &input_data) {
        Ok(dma_hasher) => {
            match dma_hasher.finalize(&mut hash_result[..48]).await {
                // Call finalize with output buffer.
                Ok(()) => defmt::info!("DMA Async Hash output: {=[u8]:x}", &hash_result[..48]),
                Err(e) => defmt::error!("DMA Hashing failed in finalize(): {:?}", defmt::Debug2Format(&e)),
            }
        }
        Err(_) => defmt::error!("Hash DMA start failed"),
    }
}
