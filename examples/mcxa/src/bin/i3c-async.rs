#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i3c::InterruptHandler;
use hal::i3c::controller::{self, BusType, I3c};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I3C0 => InterruptHandler;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I3C example");

    let config = controller::Config::default();
    let mut i3c = I3c::new_async(p.I3C0, p.P1_9, p.P1_8, Irqs, config).unwrap();
    let mut buf = [0u8; 2];

    loop {
        // ~~~~~~~~ //
        // I2C mode //
        // ~~~~~~~~ //

        // RSTDAA: reset first to make sure device responds to I2c requests.
        i3c.async_write(0x7e, &[0x06], BusType::I3cSdr).await.unwrap();

        i3c.async_write_read(0x48, &[0x00], &mut buf, BusType::I2c)
            .await
            .unwrap();
        let raw = f32::from(i16::from_be_bytes(buf) / 16);
        let temp_i2c = raw * 0.0625;
        defmt::info!("P3T1755 via I2C: {}C", temp_i2c);
        Timer::after_secs(1).await;

        // ~~~~~~~~~~~~ //
        // I3C SDR mode //
        // ~~~~~~~~~~~~ //

        // RSTDAA
        i3c.async_write(0x7e, &[0x06], BusType::I3cSdr).await.unwrap();

        Timer::after_micros(100).await;

        // ENTDAA
        i3c.async_write(0x7e, &[0x07], BusType::I3cSdr).await.unwrap();

        // P3T1755 temperature register = 0x00
        i3c.async_write_read(0x48, &[0x00], &mut buf, BusType::I3cSdr)
            .await
            .unwrap();
        let raw = f32::from(i16::from_be_bytes(buf) / 16);
        let temp_i3c_sdr = raw * 0.0625;
        defmt::info!("P3T1755 via I3C SDR: {}C", temp_i3c_sdr);
        Timer::after_secs(1).await;
    }
}
