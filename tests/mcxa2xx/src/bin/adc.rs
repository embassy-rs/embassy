#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::adc::{Command, CommandConfig, CommandId, Trigger};
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::gpio::Output;
use hal::adc::{self, Adc};
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::pac::adc::vals::Mode;
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

    let commands = &[Command::new_single(
        p.P2_4,
        CommandConfig {
            resolution: Mode::DATA_16_BITS,
            ..Default::default()
        },
    )];
    let mut adc = Adc::new_async(
        p.ADC0,
        Irqs,
        commands,
        &[
            Trigger {
                target_command_id: CommandId::Cmd1,
                ..Default::default()
            },
            Trigger {
                target_command_id: CommandId::Cmd1,
                ..Default::default()
            },
        ],
        adc::Config::default(),
    )
    .unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    // Set output low. ADC should measure (close to) GND

    output.set_low();
    embassy_time::Timer::after_millis(10).await;

    adc.do_software_trigger(0b0001).unwrap();
    let val = adc.wait_get_conversion().await.unwrap();
    assert!(val.conv_value < 0x1000);
    assert_eq!(val.command, CommandId::Cmd1);
    assert_eq!(val.loop_channel_index, 0);
    assert_eq!(val.trigger_id_source, 0);

    // Set output high, so ADC should measure (close to) VDD

    output.set_high();
    embassy_time::Timer::after_millis(10).await;

    adc.do_software_trigger(0b0010).unwrap();
    let val = adc.wait_get_conversion().await.unwrap();
    assert!(val.conv_value > 0xE000);
    assert_eq!(val.command, CommandId::Cmd1);
    assert_eq!(val.loop_channel_index, 0);
    assert_eq!(val.trigger_id_source, 1);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
