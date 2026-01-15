#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::gpio::{Input, Pull};
use hal::i3c::controller::{self, I3c};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I3C example");

    // Note: P0_2 is connected to P1_8 on the FRDM_MCXA276 via a resistor, and
    // defaults to SWO on the debug peripheral. Explicitly make it a high-z
    // input.
    let _pin = Input::new(p.P0_2, Pull::Disabled);

    let config = controller::Config::default();
    let mut i3c = I3c::new_blocking(p.I3C0, p.P1_9, p.P1_8, config).unwrap();
    let mut buf = [0u8; 2];

    loop {
        // ~~~~~~~~ //
        // I2C mode //
        // ~~~~~~~~ //

        // RSTDAA
        i3c.blocking_write(0x7e, &[0x06]).unwrap();

        i3c.i2c_blocking_write_read(0x48, &[0x00], &mut buf).unwrap();
        let raw = f32::from(i16::from_be_bytes(buf) / 16);
        let temp_i2c = raw * 0.0625;
        defmt::info!("P3T1755 via I2C: {}C", temp_i2c);
        Timer::after_secs(1).await;

        // ~~~~~~~~ //
        // I3C mode //
        // ~~~~~~~~ //

        // RSTDAA
        i3c.blocking_write(0x7e, &[0x06]).unwrap();

        Timer::after_micros(100).await;

        // ENTDAA
        i3c.blocking_write(0x7e, &[0x07]).unwrap();

        // P3T1755 temperature register = 0x00
        i3c.blocking_write_read(0x48, &[0x00], &mut buf).unwrap();
        let raw = f32::from(i16::from_be_bytes(buf) / 16);
        let temp_i3c = raw * 0.0625;
        defmt::info!("P3T1755 via I3C: {}C", temp_i3c);
        Timer::after_secs(1).await;
    }
}
