#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART1;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUartTx, Config};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    UART1_IRQ => BufferedInterruptHandler<UART1>;
});

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    static TX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let mut tx = BufferedUartTx::new(p.UART1, p.PIN_4, Irqs, &mut TX_BUF.init([0; 64])[..], Config::default());
    info!("BufferedUartTx created");

    let mut i = 0u32;
    loop {
        info!("still alive {}", i);
        tx.write(b"hello\r\n").await.unwrap();
        tx.flush().await.unwrap();
        i += 1;
        Timer::after_millis(500).await;
    }
}
