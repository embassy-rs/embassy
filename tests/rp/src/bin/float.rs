#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pac;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    embassy_rp::init(Default::default());
    info!("Hello World!");

    const PI_F: f32 = 3.1415926535f32;
    const PI_D: f64 = 3.14159265358979323846f64;

    pac::BUSCTRL
        .perfsel(0)
        .write(|r| r.set_perfsel(pac::busctrl::vals::Perfsel::ROM));

    for i in 0..=360 {
        let rad_f = (i as f32) * PI_F / 180.0;
        info!(
            "{}° float: {=f32} / {=f32} / {=f32} / {=f32}",
            i,
            rad_f,
            rad_f - PI_F,
            rad_f + PI_F,
            rad_f % PI_F
        );
        let rad_d = (i as f64) * PI_D / 180.0;
        info!(
            "{}° double: {=f64} / {=f64} / {=f64} / {=f64}",
            i,
            rad_d,
            rad_d - PI_D,
            rad_d + PI_D,
            rad_d % PI_D
        );
        Timer::after_millis(10).await;
    }

    let rom_accesses = pac::BUSCTRL.perfctr(0).read().perfctr();
    // every float operation used here uses at least 10 cycles
    defmt::assert!(rom_accesses >= 360 * 12 * 10);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
