//! Run SAADC on multiple pins only every 3rd time, to show anomaly 241 workaround.
//!
//! To correctly measure the MCU current on the NRF52DK follow the instructions
//! <https://docs.nordicsemi.com/bundle/ug_nrf52832_dk/page/UG/dk/prepare_board.html>
//! otherwise you will measure the whole board, including the segger j-link chip for example

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::saadc::{Oversample, Saadc};
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());

    // For PPK2 digital channel plot to track when SAADC is on/off.
    let mut ppk2_d0 = Output::new(p.P0_27, Level::Low, OutputDrive::Standard);
    let mut num_loops: usize = 0;
    loop {
        num_loops += 1;
        if num_loops.is_multiple_of(3) {
            ppk2_d0.set_high();
            let battery_pin = p.P0_02.reborrow();
            let sensor1_pin = p.P0_03.reborrow();
            let mut adc_config = saadc::Config::default();
            adc_config.oversample = Oversample::OVER4X;
            let battery = saadc::ChannelConfig::single_ended(battery_pin);
            let sensor1 = saadc::ChannelConfig::single_ended(sensor1_pin);
            let mut saadc = Saadc::new(p.SAADC.reborrow(), Irqs, adc_config, [battery, sensor1]);
            // Indicated: wait for ADC calibration.
            saadc.calibrate().await;
            let mut buf = [0; 2];
            info!("sampling...");
            saadc.sample(&mut buf).await;
            info!("data: {:x}", buf);

            // Sleep to show the high power usage on the plot, even though sampling is done.
            Timer::after_millis(100).await;
            ppk2_d0.set_low();
            // disable the following line to show the anomaly on the power profiler plot.
            core::mem::drop(saadc);
            // Sleep to show the power usage when drop did not happen.
            Timer::after_millis(100).await;
            // worst case drop happens here
        } else {
            info!("waiting");
        }
        // Sleep for 1 second. The executor ensures the core sleeps with a WFE when it has nothing to do.
        // During this sleep, the nRF chip should only use ~3uA
        Timer::after_secs(1).await;
    }
}
