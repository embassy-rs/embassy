#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::i3c::controller::{self, BusType, I3c};
use embassy_time::Timer;
use hal::config::Config;
use hal::peripherals::I3C0;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I3C0 => embassy_mcxa::i3c::InterruptHandler<I3C0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I3C test");

    let config = controller::Config::default();
    let mut i3c = I3c::new_async(p.I3C0, p.P1_9, p.P1_8, Irqs, config).unwrap();
    let mut buf = [0u8; 2];

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
    Timer::after_millis(10).await;

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

    assert!((-40.0..120.0).contains(&temp_i2c));
    assert!((-40.0..120.0).contains(&temp_i3c_sdr));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
