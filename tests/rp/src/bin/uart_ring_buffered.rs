#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::dma_ring_tx::TxInterruptHandler;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH2, UART0};
use embassy_rp::uart::{Config, Error, InterruptHandler, RingBufferedUart, RingBufferedUartRx};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use embedded_io_async::Write;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, TxInterruptHandler<DMA_CH2>;
});

/// Split into 4 DMA segments of 64 bytes.
const RX_POOL_LEN: usize = 256;
/// Deliberately small, so the streams below wrap it and have to wait for space.
const TX_POOL_LEN: usize = 64;

fn pattern(i: usize, seed: u8) -> u8 {
    (i as u8).wrapping_mul(31).wrapping_add(seed)
}

/// Read `len` bytes and check them against `pattern`.
async fn read_pattern(rx: &mut RingBufferedUartRx<'_>, len: usize, seed: u8) -> Result<(), Error> {
    let mut got = 0;
    let mut buf = [0u8; 64];
    while got < len {
        let want = (len - got).min(buf.len());
        let n = rx.read(&mut buf[..want]).await?;
        for (j, b) in buf[..n].iter().enumerate() {
            assert_eq!(*b, pattern(got + j, seed), "mismatch at byte {}", got + j);
        }
        got += n;
    }
    Ok(())
}

fn fill(buf: &mut [u8], seed: u8) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = pattern(i, seed);
    }
}

fn drain(rx: &mut RingBufferedUartRx<'_>) {
    let mut scratch = [0u8; RX_POOL_LEN];
    while !rx.is_empty() {
        let _ = rx.blocking_read(&mut scratch);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let mut config = Config::default();
    config.baudrate = 1_000_000;

    static RX_POOL: StaticCell<[u8; RX_POOL_LEN]> = StaticCell::new();
    static TX_POOL: StaticCell<[u8; TX_POOL_LEN]> = StaticCell::new();
    let rx_pool = &mut RX_POOL.init([0; RX_POOL_LEN])[..];
    let tx_pool = &mut TX_POOL.init([0; TX_POOL_LEN])[..];

    // GP0 (UART0 TX) is wired to GP1 (UART0 RX).
    let mut uart = RingBufferedUart::new(
        p.UART0, p.PIN_0, p.PIN_1, p.DMA_CH0, p.DMA_CH1, p.DMA_CH2, Irqs, rx_pool, tx_pool, config,
    );
    let (tx, rx) = uart.split_ref();

    // Several pool sizes through both rings at once, so the TX ring wraps and waits for space
    // many times while the RX ring keeps up.
    {
        let mut data = [0u8; 1500];
        fill(&mut data, 1);
        let ((), r) = join(
            async {
                unwrap!(tx.write_all(&data).await);
                tx.flush().await;
            },
            read_pattern(rx, data.len(), 1),
        )
        .await;
        unwrap!(r);
    }

    // Each write size from 1 to 13 from an idle channel, so every one starts a fresh run.
    for size in 1..=13 {
        let mut data = [0u8; 13];
        fill(&mut data[..size], 2);
        unwrap!(tx.write_all(&data[..size]).await);
        tx.flush().await;
        unwrap!(read_pattern(rx, size, 2).await);
    }

    // `flush` returns once the last stop bit is out, so everything has been received a moment later.
    {
        let mut data = [0u8; 40];
        fill(&mut data, 3);
        unwrap!(tx.write_all(&data).await);
        tx.flush().await;
        assert_eq!(tx.pending(), 0);
        Timer::after_micros(50).await;
        assert_eq!(rx.len(), data.len());
        drain(rx);
    }

    // A burst that fits in the RX pool is captured whole while nothing reads.
    {
        let mut data = [0u8; RX_POOL_LEN / 2];
        fill(&mut data, 4);
        unwrap!(tx.write_all(&data).await);
        tx.flush().await;
        Timer::after_millis(1).await;
        unwrap!(read_pattern(rx, data.len(), 4).await);
    }

    // More than the RX pool can hold while nothing reads is reported as an overrun, not as data.
    // It stays under one full pool: a reader a whole pool or more behind cannot be detected.
    {
        let mut data = [0u8; RX_POOL_LEN - RX_POOL_LEN / 16];
        fill(&mut data, 5);
        unwrap!(tx.write_all(&data).await);
        tx.flush().await;
        Timer::after_millis(1).await;
        let mut buf = [0u8; RX_POOL_LEN];
        assert!(matches!(rx.read(&mut buf).await, Err(Error::Overrun)));
        drain(rx);
    }

    // And reception recovers afterwards.
    {
        let mut data = [0u8; 100];
        fill(&mut data, 6);
        unwrap!(tx.write_all(&data).await);
        tx.flush().await;
        unwrap!(read_pattern(rx, data.len(), 6).await);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
