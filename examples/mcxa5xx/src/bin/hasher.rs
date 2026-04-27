#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::config::Config;
use hal::dma::DmaChannel;
use hal::peripherals::SGI0;
use hal::sgi::hash::{BlockingHasher, DmaHasher, HashSize, StreamingHasher};
use hal::sgi::{InterruptHandler, Sgi};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Note: SGI is identical among all MCXA MCUs.

bind_interrupts!(struct Irqs {
    SGI => InterruptHandler<SGI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    defmt::info!("SGI example");

    // Take DMA channel ownership, will use for DMA copy.
    let mut dma_ch0 = DmaChannel::new(p.DMA0_CH0.reborrow());

    const MAX_LEN: usize = 32 * 1024;
    let mut input_data = [0u8; MAX_LEN];

    for i in 0..input_data.len() {
        input_data[i] = i as u8;
    }

    let mut hash_result: [u8; 64] = [0u8; 64];

    let mut blocking_hasher = BlockingHasher::new(Sgi::new_blocking(p.SGI0.reborrow()).unwrap());

    // Hash blocking will block until complete, max input size limited to 512 bytes.
    match blocking_hasher.hash_blocking(HashSize::Sha384, &input_data[..308], &mut hash_result) {
        Ok(_) => defmt::info!("Hash output blocking: {=[u8]:x}", &hash_result[..48]), // SHA-384 is 48 bytes or 384 bits
        Err(_) => defmt::error!("Block Hashing failed"),
    }

    match blocking_hasher.hash_blocking(HashSize::Sha512, &input_data[..308], &mut hash_result) {
        Ok(_) => defmt::info!("Hash output blocking: {=[u8]:x}", &hash_result[..64]), // SHA-512 is 64 bytes or 512 bits
        Err(_) => defmt::error!("Block Hashing failed"),
    }

    drop(blocking_hasher);

    //streaming mode:
    let mut hasher = StreamingHasher::new(HashSize::Sha512, None).unwrap();
    let mut index = 0;
    // hasher.update has a max input size of 512 bytes per update call.
    for _ in 0..3 {
        let _ = hasher
            .update(p.SGI0.reborrow(), &input_data[index..index + 128])
            .unwrap();
        index += 128; // doing one full block at a time.
    }

    for _ in 0..3 {
        let _ = hasher
            .update(p.SGI0.reborrow(), &input_data[index..index + 50])
            .unwrap();
        index += 50; // doing less than a full block.
    }

    let _ = hasher
        .update(p.SGI0.reborrow(), &input_data[index..index + 512])
        .unwrap(); //max size.
    //index += 512;

    // Call hasher.finalize to get the final hash result
    let _ = hasher.finalize(p.SGI0.reborrow(), &mut hash_result[..64]).unwrap();

    defmt::info!("Hash output streaming: {=[u8]:x}", &hash_result[..64]);

    let sgi = Sgi::new(p.SGI0.reborrow(), Irqs).unwrap();
    match DmaHasher::start_and_finalize(sgi, &mut dma_ch0, HashSize::Sha384, &input_data, &mut hash_result[..48]).await
    {
        Ok(()) => defmt::info!("DMA Async Hash output: {=[u8]:x}", &hash_result[..48]),
        Err(e) => defmt::error!("DMA Hashing failed: {:?}", defmt::Debug2Format(&e)),
    }
}
