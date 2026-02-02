#![no_std]
#![no_main]

// TODO: Test doesn't work yet and is not proven to work ever
// TODO: Also test ringbuffered uart
// NOTE: Blocking uart is hard to test, so maybe todo?

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::LpuartDma;
use embassy_mcxa::lpuart::buffered::BufferedLpuart;
use embedded_io_async::{Read, Write};
use hal::config::Config;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const MESSAGE_SIZE: usize = 69;
const MESSAGE: [u8; MESSAGE_SIZE] = *b"You've found the HIL tests for MCXA! Hope you have a wonderful day :)";
const MESSAGE_PLUS_1: [u8; MESSAGE_SIZE] = {
    let mut m = MESSAGE;
    let mut i = 0;
    while i < m.len() {
        m[i] = m[i].wrapping_add(1);
        i += 1;
    }
    m
};

bind_interrupts!(struct Irqs {
    LPUART3 => hal::lpuart::buffered::BufferedInterruptHandler::<hal::peripherals::LPUART3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_12m_enabled = true;
    config.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());

    let p = hal::init(config);

    defmt::info!("lpuart test");

    let config = hal::lpuart::Config {
        baudrate_bps: 115_200,
        rx_fifo_watermark: 0,
        tx_fifo_watermark: 0,
        ..Default::default()
    };

    static TX_BUF: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0u8; 256]);
    static RX_BUF: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0u8; 256]);

    let echo_uart = BufferedLpuart::new(
        p.LPUART3,
        p.P4_5, // TX pin
        p.P4_2, // RX pin
        Irqs,
        TX_BUF.take(),
        RX_BUF.take(),
        config,
    )
    .unwrap();

    spawner.spawn(echo_plus_1(echo_uart).unwrap());

    let mut dma_uart = LpuartDma::new(
        p.LPUART2, // Peripheral
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        p.DMA_CH0, // TX DMA channel
        p.DMA_CH1, // RX DMA channel
        config,
    )
    .unwrap();

    let mut rx_buffer = [0; MESSAGE_SIZE];
    embassy_time::Timer::after_millis(10).await;

    defmt::info!("Sending message");
    dma_uart.write_dma(&MESSAGE).await.unwrap();
    defmt::info!("Done, waiting for response");
    dma_uart.read_dma(&mut rx_buffer).await.unwrap();
    assert_eq!(rx_buffer, MESSAGE_PLUS_1);
}

#[embassy_executor::task]
async fn echo_plus_1(mut uart: BufferedLpuart<'static>) {
    // Echo loop
    let mut buf = [0u8; MESSAGE_SIZE];

    defmt::info!("Waiting on echo task");

    let used = uart.read_exact(&mut buf).await.unwrap();

    defmt::info!("Received {} bytes", used);
    embassy_time::Timer::after_millis(100).await;

    for byte in buf.iter_mut() {
        *byte = byte.wrapping_add(1);
    }
    uart.write_all(&buf).await.unwrap();
    defmt::warn!("Done sending");
}
