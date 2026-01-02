//! This example shows how to use the RP2040 ADC with DMA, both single- and multichannel reads.
//! For multichannel, the samples are interleaved in the buffer:
//! `[ch1, ch2, ch3, ch4, ch1, ch2, ch3, ch4, ...]`
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::Pull;
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Here we go!");

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut dma = p.DMA_CH0;
    let mut pin = Channel::new_pin(p.PIN_26, Pull::Up);
    let mut pins = [
        Channel::new_pin(p.PIN_27, Pull::Down),
        Channel::new_pin(p.PIN_28, Pull::None),
        Channel::new_pin(p.PIN_29, Pull::Up),
        Channel::new_temp_sensor(p.ADC_TEMP_SENSOR),
    ];

    const BLOCK_SIZE: usize = 100;
    const NUM_CHANNELS: usize = 4;
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        // Read 100 samples from a single channel
        let mut buf = [0_u16; BLOCK_SIZE];
        let div = 479; // 100kHz sample rate (48Mhz / 100kHz - 1)
        adc.read_many(&mut pin, &mut buf, div, dma.reborrow()).await.unwrap();
        info!("single: {:?} ...etc", buf[..8]);

        // Read 100 samples from 4 channels interleaved
        let mut buf = [0_u16; { BLOCK_SIZE * NUM_CHANNELS }];
        let div = 119; // 100kHz sample rate (48Mhz / 100kHz * 4ch - 1)
        adc.read_many_multichannel(&mut pins, &mut buf, div, dma.reborrow())
            .await
            .unwrap();
        info!("multi:  {:?} ...etc", buf[..NUM_CHANNELS * 2]);

        ticker.next().await;
    }
}
