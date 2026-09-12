#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_silabs::eusart::{self, Config, Uart, UartRx};
use embassy_silabs::mode::Async;
use embassy_silabs::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
    EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
});

static CHANNEL: Channel<ThreadModeRawMutex, [u8; 8], 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_silabs::init({
        use embassy_silabs::rcc::*;
        let mut cfg = embassy_silabs::Config::default();
        // 40 MHz HFXO crystal (brd2713a/MGM260P); the EUSART runs off EM01GRPCCLK.
        cfg.hfxo = Some(HfxoConfig {
            freq: Hertz::mhz(40),
            mode: HfxoMode::Xtal,
            ctune: HfxoCtune::Auto { default: 140 },
        });
        cfg.sysclk = SysclkSource::Hfxo;
        cfg.em01grpaclk = Em01GrpAClkSource::Hfxo;
        cfg.em01grpcclk = Em01GrpCClkSource::Hfxo;
        cfg
    });
    info!("Hello World!");

    let mut uart = unwrap!(Uart::new(p.EUSART0, p.PA05, p.PA04, Irqs, Config::default()));
    unwrap!(uart.blocking_write(b"Type 8 chars to echo!\r\n"));

    let (mut tx, rx) = uart.split();

    spawner.spawn(unwrap!(reader(rx)));

    loop {
        let buf = CHANNEL.receive().await;
        info!("writing...");
        unwrap!(tx.write(&buf).await);
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, Async>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        CHANNEL.send(buf).await;
    }
}
