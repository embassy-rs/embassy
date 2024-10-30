#![no_std]
#![no_main]

use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_stm32::usart::{self, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

/*--- Main ---------------------------------------------------------------------------------------*/

bind_interrupts!(
    struct Irqs {
        USART3 => usart::InterruptHandler<peripherals::USART3>;
    }
);

/// this test will print all the bytes received in it's serial port
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut config = usart::Config::default();
    config.baudrate = 115_200;
    info!("debug uart baudrate: {}", config.baudrate);

    let port = Uart::new(
        p.USART3,     // periph
        p.PD9,        // rx,
        p.PD8,        // tx,
        Irqs,         // prove irq bound
        p.GPDMA2_CH0, // tx_dma
        p.GPDMA2_CH1, // rx_dma
        config,
    )
    .unwrap();

    const S: usize = 2000;
    static BUFF: StaticCell<[u8; S]> = StaticCell::new();
    let b = BUFF.init_with(|| [0_u8; S]);

    let (mut port_tx, port_rx) = port.split();
    let mut port_rx = port_rx.into_ring_buffered(b);

    _ = port_tx.write(b"hello there\n").await;

    let mut read_buff = [0_u8; 256];
    let mut count = 0;

    loop {
        if let Ok(c) = port_rx.read(&mut read_buff).await.map_err(|e| {
            error!("read error: {}", e);
        }) {
            if let Ok(s) = core::str::from_utf8(&read_buff).map_err(|e| {
                error!("str parse error: {}", defmt::Debug2Format(&e));
            }) {
                count += c;

                debug!("total_rx: {} delta: {} -- rx: {}", count, c, s);
            }
        }

        // prove the dma is reading in the background by pausing the current read for a while
        Timer::after_millis(500).await;
    }
}
