#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::dma::copy;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // Check `u8` copy
    {
        let data: [u8; 2] = [0xC0, 0xDE];
        let mut buf = [0; 2];
        unsafe { copy(p.DMA_CH0, &data, &mut buf).await };
        assert_eq!(buf, data);
    }

    // Check `u16` copy
    {
        let data: [u16; 2] = [0xC0BE, 0xDEAD];
        let mut buf = [0; 2];
        unsafe { copy(p.DMA_CH1, &data, &mut buf).await };
        assert_eq!(buf, data);
    }

    // Check `u32` copy
    {
        let data: [u32; 2] = [0xC0BEDEAD, 0xDEADAAFF];
        let mut buf = [0; 2];
        unsafe { copy(p.DMA_CH2, &data, &mut buf).await };
        assert_eq!(buf, data);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
