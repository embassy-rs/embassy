#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::flexcomm::spi::Spi;
use embassy_imxrt::gpio;
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use is31fl3743b_driver::{CSy, Is31fl3743b, SWx};
use {defmt_rtt as _, embassy_imxrt_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing SPI");

    let cs = gpio::Output::new(
        p.PIO1_6,
        gpio::Level::Low,
        gpio::DriveMode::PushPull,
        gpio::DriveStrength::Normal,
        gpio::SlewRate::Standard,
    );

    let spi = Spi::new_blocking(p.FLEXCOMM5, p.PIO1_3, p.PIO1_5, p.PIO1_4, Default::default());
    let delay = Delay;

    // One SPI device only on the SPI bus
    let spi_dev = ExclusiveDevice::new(spi, cs, delay).unwrap();

    // Instantiate IS31FL3743B device
    let mut driver = Is31fl3743b::new(spi_dev).unwrap();

    // Enable phase delay to help reduce power noise
    let _ = driver.enable_phase_delay();
    // Set global current, check method documentation for more info
    let _ = driver.set_global_current(90);

    let _ = driver.set_led_peak_current_bulk(SWx::SW1, CSy::CS1, &[100; 11 * 18]);

    // Driver is fully set up, we can now start turning on LEDs!
    // Create a white breathing effect
    loop {
        for brightness in (0..=255_u8).chain((0..=255).rev()) {
            let _ = driver.set_led_brightness_bulk(SWx::SW1, CSy::CS1, &[brightness; 11 * 18]);
            Timer::after_micros(1).await;
        }
    }
}
