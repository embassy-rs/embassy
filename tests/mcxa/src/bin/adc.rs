#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::gpio::Output;
use hal::adc::{self, Adc, TriggerPriorityPolicy};
use hal::clocks::PoweredClock;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::{AdcClockSel, Div4};
use hal::config::Config;
use hal::pac::adc::vals::{CalAvgs, Mode, Pwrsel, Refsel};
use hal::peripherals::ADC0;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<ADC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    let mut output = Output::new(
        p.P1_8,
        embassy_mcxa::gpio::Level::Low,
        embassy_mcxa::gpio::DriveStrength::Normal,
        embassy_mcxa::gpio::SlewRate::Slow,
    );

    let adc_config = adc::Config {
        enable_in_doze_mode: true,
        conversion_average_mode: CalAvgs::AVERAGE_128,
        enable_analog_preliminary: true,
        power_up_delay: 0x80,
        reference_voltage_source: Refsel::OPTION_3,
        power_level_mode: Pwrsel::LOWEST,
        trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
        enable_conv_pause: false,
        conv_pause_delay: 0,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        source: AdcClockSel::FroLfDiv,
        div: Div4::no_div(),
    };
    let mut adc = Adc::new_async(p.ADC0, p.P2_4, Irqs, adc_config).unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();
    adc.set_resolution(Mode::DATA_16_BITS);

    // Set output low. ADC should measure (close to) GND

    output.set_low();
    embassy_time::Timer::after_millis(10).await;

    let val = adc.read().await.unwrap();
    assert!(val < 0x1000);

    // Set output high, so ADC should measure (close to) VDD

    output.set_high();
    embassy_time::Timer::after_millis(10).await;

    let val = adc.read().await.unwrap();
    assert!(val > 0xE000);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
