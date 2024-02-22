#![no_std]
#![no_main]
teleprobe_meta::target!(b"nrf52840-dk");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::buffered_uarte::{self, BufferedUarteRx, BufferedUarteTx};
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;
    UARTE1 => buffered_uarte::InterruptHandler<peripherals::UARTE1>;
});

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

        let mut tx = BufferedUarteTx::new(&mut p.UARTE1, Irqs, &mut p.P1_02, config.clone(), &mut tx_buffer);

        let mut rx = BufferedUarteRx::new(
            &mut p.UARTE0,
            &mut p.TIMER0,
            &mut p.PPI_CH0,
            &mut p.PPI_CH1,
            &mut p.PPI_GROUP0,
            Irqs,
            &mut p.P1_03,
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
