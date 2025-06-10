#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = embassy_nrf, peripherals = false, dispatchers = [EGU0_SWI0, EGU1_SWI1])]
mod app {
    use defmt::info;
    use embassy_nrf::gpio::{Level, Output, OutputDrive};
    use embassy_nrf::{peripherals, Peri};
    use embassy_time::Timer;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        info!("Hello World!");

        let p = embassy_nrf::init(Default::default());
        blink::spawn(p.P0_13).map_err(|_| ()).unwrap();

        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn blink(_cx: blink::Context, pin: Peri<'static, peripherals::P0_13>) {
        let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);

        loop {
            info!("off!");
            led.set_high();
            Timer::after_millis(300).await;
            info!("on!");
            led.set_low();
            Timer::after_millis(300).await;
        }
    }
}
