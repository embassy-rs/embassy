#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::adc::{ConvCommandConfig, ConvTriggerConfig};
use embassy_time::{Duration, Ticker};
use hal::adc::{self, Adc, TriggerPriorityPolicy};
use hal::clocks::PoweredClock;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::{AdcClockSel, Div4};
use hal::config::Config;
use hal::pac::adc::vals::{CalAvgs, Mode, Pwrsel, Refsel, Tcmd};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const G_LPADC_RESULT_SHIFT: u32 = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("=== ADC polling Example ===");

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
    let adc = Adc::new_blocking(p.ADC1, p.P1_10, adc_config).unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    let conv_command_config = ConvCommandConfig {
        conversion_resolution_mode: Mode::DATA_16_BITS,
        ..ConvCommandConfig::default()
    };
    adc.set_conv_command_config(1, &conv_command_config).unwrap();

    let conv_trigger_config: ConvTriggerConfig = ConvTriggerConfig {
        target_command_id: Tcmd::EXECUTE_CMD1,
        enable_hardware_trigger: false,
        ..Default::default()
    };
    adc.set_conv_trigger_config(0, &conv_trigger_config).unwrap();

    defmt::info!("=== ADC configuration done... ===");
    let mut tick = Ticker::every(Duration::from_millis(100));

    loop {
        tick.next().await;
        adc.do_software_trigger(1).unwrap();
        let result = loop {
            match adc.get_conv_result() {
                Ok(res) => break res,
                Err(_) => {
                    // Conversion not ready, continue polling
                }
            }
        };
        let value = result.conv_value >> G_LPADC_RESULT_SHIFT;
        defmt::info!("ADC value: {=u16}", value);
    }
}
