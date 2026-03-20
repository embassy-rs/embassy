#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::adc::{AnyAdcPin, Command, CommandConfig, CommandId, Trigger};
use embassy_mcxa::pac::adc::vals::{Avgs, Mode, Sts};
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::{Duration, Ticker};
use hal::adc::{self, Adc};
use hal::clocks::config::Div8;
use hal::config::Config;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1 => adc::InterruptHandler<peripherals::ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("=== ADC temperature Example ===");

    let commands = &[Command::new_looping(
        AnyAdcPin::temperature(), // Use the temperature channel
        2,
        CommandConfig {
            chained_command: None,
            averaging: Avgs::AVERAGE_1024,  // Max average
            sample_time: Sts::SAMPLE_131P5, // Max sample time
            compare: adc::Compare::Disabled,
            resolution: Mode::DATA_16_BITS,
            wait_for_trigger: false,
        },
    )
    .unwrap()];

    let mut adc = Adc::new_async(
        p.ADC1,
        Irqs,
        commands,
        &[Trigger {
            target_command_id: CommandId::Cmd1,
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
        let conversion1 = adc.wait_get_conversion().await.unwrap();
        let conversion2 = adc.wait_get_conversion().await.unwrap();

        let celsius = calculate_temperature(conversion1.conv_value, conversion2.conv_value);
        defmt::info!("Current temperature: {=f32}", celsius);
    }
}

fn calculate_temperature(conv1: u16, conv2: u16) -> f32 {
    // Constants from the datasheet. May differ per device/family
    const A: f32 = 738.0;
    const B: f32 = 287.5;
    const ALPHA: f32 = 10.06;

    A * (ALPHA * (conv2 as f32 - conv1 as f32) / (conv2 as f32 + (ALPHA * (conv2 as f32 - conv1 as f32)))) - B
}
