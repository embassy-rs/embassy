#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::adc::{Command, CommandConfig, CommandId, Trigger};
use embassy_time::{Duration, Ticker};
use hal::adc::{self, Adc};
use hal::clocks::config::Div8;
use hal::config::Config;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("=== ADC polling Example ===");

    let commands = &[
        Command::new_single(
            p.P1_10,
            CommandConfig {
                chained_command: Some(CommandId::Cmd2), // Command 2 is executed after this command is done
                ..Default::default()
            },
        ),
        Command::new_looping(
            p.P1_11,
            3, // Command is run 3 times
            CommandConfig {
                chained_command: None, // Terminate the conversion after command is done
                ..Default::default()
            },
        )
        .unwrap(),
    ];

    let mut adc = Adc::new_blocking(
        p.ADC1,
        commands,
        &[Trigger {
            target_command_id: CommandId::Cmd1,
            enable_hardware_trigger: false,
            ..Default::default()
        }],
        adc::Config::default(),
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
            match adc.try_get_conversion() {
                Ok(res) => {
                    defmt::info!("ADC result: {}", res);
                }
                Err(adc::Error::FifoPending) => {
                    // Conversion not ready, continue polling
                }
                Err(_) => {
                    // We're done
                    break;
                }
            }
        }
    }
}
