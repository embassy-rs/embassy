#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::eusart::{self, Config, Uart};
use embassy_silabs::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
    EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init({
        use embassy_silabs::cmu::*;
        let mut cfg = embassy_silabs::Config::default();
        // brd2713a (MGM260P) has a 40 MHz HFXO crystal.
        cfg.cmu.hfxo = Some(HfxoConfig {
            freq: Hertz::mhz(40),
            mode: HfxoMode::Xtal,
            ctune: HfxoCtune::Auto { default: 140 },
        });
        cfg.cmu.sysclk = SysclkSource::Hfxo;
        cfg.cmu.mux.em01grpaclk = Some(mux::Em01grpaclkctrlClksel::Hfxo);
        // EUSART0 runs on EM01GRPCCLK.
        cfg.cmu.mux.em01grpcclk = Some(mux::Em01grpcclkctrlClksel::Hfxo);
        cfg
    });
    let mut uart = unwrap!(Uart::new(p.EUSART0, p.PA04, p.PA05, Irqs, Config::default()));

    uart.write(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        uart.read(&mut buf).await.unwrap();
        uart.write(&buf).await.unwrap();
    }
}
