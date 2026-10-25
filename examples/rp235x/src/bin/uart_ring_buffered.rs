//! This example shows how to use the ring-buffered DMA UART in the RP235x chip.
//!
//! [`RingBufferedUart`] receives continuously into a DMA ring and sends from another one, with no
//! per-byte interrupt in either direction. Reception keeps running between reads, so bytes that
//! arrive while the reader is busy are not lost as long as they fit in the RX pool.
//!
//! No specific hardware is specified in this example. If you connect pin 0 and 1 you should get the same data back.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::dma_ring_tx::TxInterruptHandler;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH2, UART0};
use embassy_rp::uart::{Config, InterruptHandler, RingBufferedUart, RingBufferedUartRx};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use embedded_io_async::{Read, Write};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
    // The TX channel is restarted from its own interrupt handler, not `dma::InterruptHandler`.
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, TxInterruptHandler<DMA_CH2>;
});

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // The RX pool is split into 4 DMA segments, and a blocked reader is only woken when one fills.
    // Each 32-byte message below fills exactly one 32-byte segment, so it is read as soon as it lands.
    static RX_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 128])[..];
    static TX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 64])[..];

    let uart = RingBufferedUart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        p.DMA_CH0, // RX data channel
        p.DMA_CH1, // RX control channel, no interrupt needed
        p.DMA_CH2, // TX channel
        Irqs,
        rx_buf,
        tx_buf,
        Config::default(),
    );
    let (mut tx, rx) = uart.split();

    spawner.spawn(unwrap!(reader(rx)));

    info!("Writing...");
    loop {
        let data: [u8; 32] = core::array::from_fn(|i| i as u8 + 1);
        info!("TX {:?}", data);
        // Queues into the TX pool and returns; the DMA sends it in the background.
        tx.write_all(&data).await.unwrap();
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: RingBufferedUartRx<'static>) {
    info!("Reading...");
    loop {
        let mut buf = [0; 32];
        rx.read_exact(&mut buf).await.unwrap();
        info!("RX {:?}", buf);
    }
}
