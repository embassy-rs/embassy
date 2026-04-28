#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");
use embassy_executor::Spawner;
use embassy_mcxa::adc::{Async, Command, CommandConfig, CommandId, Trigger};
use embassy_mcxa::bind_interrupts;
use hal::adc::{self, Adc};
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::dac::Dac;
use hal::pac::adc::Mode;
use hal::peripherals::ADC0;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const MAX_ERR: f32 = 0.2;

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<ADC0>;
});

async fn test_value<'p>(value: u16, dac: &Dac, adc: &mut Adc<'p, Async>) {
    dac.write(value);
    let expected = value as f32 / 4095.0 * 3.3;
    defmt::unwrap!(adc.do_software_trigger(1));
    let measured = adc.wait_get_conversion().await.unwrap().conv_value as f32 / 65535.0 * 3.3;
    assert!(
        measured > (expected - MAX_ERR),
        "Measured value was too low, expected: {}, measured: {}.",
        expected,
        measured
    );
    assert!(
        measured < (expected + MAX_ERR),
        "Measured value was too high, expected: {}, measured: {}.",
        expected,
        measured
    );
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    let dac = defmt::unwrap!(Dac::new(p.DAC0, p.P2_2));

    let commands = &[Command::new_single(
        p.P2_7,
        CommandConfig {
            resolution: Mode::Data16Bits,
            ..Default::default()
        },
    )];

    let mut adc_config = adc::Config::default();
    adc_config.reference_voltage_source = adc::ReferenceVoltage::VddaAnaPin;
    let mut adc = Adc::new_async(
        p.ADC0,
        Irqs,
        commands,
        &[Trigger {
            target_command_id: CommandId::Cmd1,
            ..Default::default()
        }],
        adc_config,
    )
    .unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    test_value(4095, &dac, &mut adc).await;
    test_value(3072, &dac, &mut adc).await;
    test_value(2048, &dac, &mut adc).await;
    test_value(1024, &dac, &mut adc).await;
    test_value(512, &dac, &mut adc).await;
    test_value(0, &dac, &mut adc).await;

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
