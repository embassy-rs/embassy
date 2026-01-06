#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::dma::copy;
use {defmt_rtt as _, panic_probe as _};

const BUFLEN: usize = 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Test memory-to-memory DMA transfers");

    let src = [0x55u8; BUFLEN];
    let mut dst = [0u8; BUFLEN];

    unsafe { copy(p.DMA0_CH0, &src, &mut dst) }.await;
    assert!(dst == src);

    info!("DMA copy succeeded");
}
