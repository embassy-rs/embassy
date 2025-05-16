//! This example shows how to use `zerocopy_channel` from `embassy_sync` for
//! sending large values between two tasks without copying.
//! The example also shows how to use the RP2040 ADC with DMA.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU16, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{self, Adc, Async, Config, InterruptHandler};
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::DMA_CH0;
use embassy_rp::{bind_interrupts, Peri};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Ticker, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type SampleBuffer = [u16; 512];

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

const BLOCK_SIZE: usize = 512;
const NUM_BLOCKS: usize = 2;
static MAX: AtomicU16 = AtomicU16::new(0);

struct AdcParts {
    adc: Adc<'static, Async>,
    pin: adc::Channel<'static>,
    dma: Peri<'static, DMA_CH0>,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Here we go!");

    let adc_parts = AdcParts {
        adc: Adc::new(p.ADC, Irqs, Config::default()),
        pin: adc::Channel::new_pin(p.PIN_29, Pull::None),
        dma: p.DMA_CH0,
    };

    static BUF: StaticCell<[SampleBuffer; NUM_BLOCKS]> = StaticCell::new();
    let buf = BUF.init([[0; BLOCK_SIZE]; NUM_BLOCKS]);

    static CHANNEL: StaticCell<Channel<'_, NoopRawMutex, SampleBuffer>> = StaticCell::new();
    let channel = CHANNEL.init(Channel::new(buf));
    let (sender, receiver) = channel.split();

    spawner.must_spawn(consumer(receiver));
    spawner.must_spawn(producer(sender, adc_parts));

    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;
        let max = MAX.load(Ordering::Relaxed);
        info!("latest block's max value: {:?}", max);
    }
}

#[embassy_executor::task]
async fn producer(mut sender: Sender<'static, NoopRawMutex, SampleBuffer>, mut adc: AdcParts) {
    loop {
        // Obtain a free buffer from the channel
        let buf = sender.send().await;

        // Fill it with data
        adc.adc
            .read_many(&mut adc.pin, buf, 1, adc.dma.reborrow())
            .await
            .unwrap();

        // Notify the channel that the buffer is now ready to be received
        sender.send_done();
    }
}

#[embassy_executor::task]
async fn consumer(mut receiver: Receiver<'static, NoopRawMutex, SampleBuffer>) {
    loop {
        // Receive a buffer from the channel
        let buf = receiver.receive().await;

        // Simulate using the data, while the producer is filling up the next buffer
        Timer::after_micros(1000).await;
        let max = buf.iter().max().unwrap();
        MAX.store(*max, Ordering::Relaxed);

        // Notify the channel that the buffer is now ready to be reused
        receiver.receive_done();
    }
}
