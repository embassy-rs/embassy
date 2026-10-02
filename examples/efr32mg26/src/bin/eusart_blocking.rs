#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::eusart::{Config, Uart};
use panic_probe as _;

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
    info!("Hello World!");

    let mut uart = unwrap!(Uart::new_blocking(p.EUSART0, p.PA04, p.PA05, Config::default()));

    unwrap!(uart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(uart.blocking_read(&mut buf));
        unwrap!(uart.blocking_write(&buf));
    }
}
