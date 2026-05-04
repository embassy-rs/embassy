#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2};
use embassy_rp::{bind_interrupts, dma};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // Check `u8` copy
    {
        let data: [u8; 2] = [0xC0, 0xDE];
        let mut buf = [0; 2];
        let mut ch = dma::Channel::new(p.DMA_CH0, Irqs);
        unsafe { ch.copy(&data, &mut buf).await };
        assert_eq!(buf, data);
    }

    // Check `u16` copy
    {
        let data: [u16; 2] = [0xC0BE, 0xDEAD];
        let mut buf = [0; 2];
        let mut ch = dma::Channel::new(p.DMA_CH1, Irqs);
        unsafe { ch.copy(&data, &mut buf).await };
        assert_eq!(buf, data);
    }

    // Check `u32` copy
    {
        let data: [u32; 2] = [0xC0BEDEAD, 0xDEADAAFF];
        let mut buf = [0; 2];
        let mut ch = dma::Channel::new(p.DMA_CH2, Irqs);
        unsafe { ch.copy(&data, &mut buf).await };
        assert_eq!(buf, data);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
