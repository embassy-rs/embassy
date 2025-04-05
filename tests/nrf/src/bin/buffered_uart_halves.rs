// required-features: two-uarts
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::buffered_uarte::{self, BufferedUarteRx, BufferedUarteTx};
use embassy_nrf::{peripherals, uarte};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD1M;

    let mut tx_buffer = [0u8; 1024];
    let mut rx_buffer = [0u8; 1024];

    // test teardown + recreate of the buffereduarte works fine.
    for _ in 0..2 {
        const COUNT: usize = 40_000;

        let mut tx = BufferedUarteTx::new(
            peri!(p, UART1).reborrow(),
            peri!(p, PIN_A).reborrow(),
            irqs!(UART1_BUFFERED),
            config.clone(),
            &mut tx_buffer,
        );

        let mut rx = BufferedUarteRx::new(
            peri!(p, UART0).reborrow(),
            p.TIMER0.reborrow(),
            p.PPI_CH0.reborrow(),
            p.PPI_CH1.reborrow(),
            p.PPI_GROUP0.reborrow(),
            irqs!(UART0_BUFFERED),
            peri!(p, PIN_B).reborrow(),
            config.clone(),
            &mut rx_buffer,
        );

        let tx_fut = async {
            info!("tx initialized!");

            let mut tx_buf = [0; 215];
            let mut i = 0;
            while i < COUNT {
                let n = tx_buf.len().min(COUNT - i);
                let tx_buf = &mut tx_buf[..n];
                for (j, b) in tx_buf.iter_mut().enumerate() {
                    *b = (i + j) as u8;
                }
                let n = unwrap!(tx.write(tx_buf).await);
                i += n;
            }
        };
        let rx_fut = async {
            info!("rx initialized!");

            let mut i = 0;
            while i < COUNT {
                let buf = unwrap!(rx.fill_buf().await);

                for &b in buf {
                    assert_eq!(b, i as u8);
                    i = i + 1;
                }

                let n = buf.len();
                rx.consume(n);
            }
        };

        join(rx_fut, tx_fut).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
