#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::adc::{Command, CommandConfig, CommandId, Trigger};
use embassy_time::{Duration, Ticker};
use hal::adc::{self, Adc, TriggerPriorityPolicy};
use hal::clocks::PoweredClock;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::{AdcClockSel, Div4};
use hal::config::Config;
use hal::pac::adc::vals::{CalAvgs, Pwrsel, Refsel};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("=== ADC polling Example ===");

    let adc_config = adc::Config {
        enable_in_doze_mode: true,
        calibration_average_mode: CalAvgs::AVERAGE_128,
        power_pre_enabled: true,
        power_up_delay: 0x80,
        reference_voltage_source: Refsel::OPTION_3,
        power_level_mode: Pwrsel::LOWEST,
        trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
        conv_pause_delay: None,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        source: AdcClockSel::FroLfDiv,
        div: Div4::no_div(),
    };

    let commands = &[
        Command::new_single(
            p.P1_10,
            CommandConfig {
                chained_command: Some(CommandId::Cmd2),
                ..Default::default()
            },
        ),
        Command::new_looping(
            p.P1_11,
            3,
            CommandConfig {
                chained_command: None,
                ..Default::default()
            },
        )
        .unwrap(),
    ];

    let adc = Adc::new(
        p.ADC1,
        commands,
        &[Trigger {
            target_command_id: CommandId::Cmd1,
            enable_hardware_trigger: false,
            ..Default::default()
        }],
        adc_config,
    )
    .unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    defmt::info!("=== ADC configuration done... ===");
    let mut tick = Ticker::every(Duration::from_millis(1000));

    loop {
        tick.next().await;
        adc.do_software_trigger(0b0001).unwrap();

        loop {
            match adc.try_get_conv_result() {
                Ok(res) => {
                    defmt::info!("ADC result: {}", res);
                },
                Err(adc::Error::FifoPending) => {
                    // Conversion not ready, continue polling
                }
                Err(_) => {
                    // We're done
                    break;
                }
            }
        };
    }
}
