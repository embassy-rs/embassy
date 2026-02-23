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

    let mut p = hal::init(config);

    let mut output = Output::new(
        p.P1_8,
        embassy_mcxa::gpio::Level::Low,
        embassy_mcxa::gpio::DriveStrength::Normal,
        embassy_mcxa::gpio::SlewRate::Slow,
    );

    {
        defmt::info!("Store if");

        let commands = &[Command::new_single(
            p.P2_4.reborrow(),
            CommandConfig {
                resolution: Mode::DATA_16_BITS,
                compare: adc::Compare::StoreIf(adc::CompareFunction::GreaterThan(0x8000)),
                ..Default::default()
            },
        )];
        let mut adc = Adc::new_async(
            p.ADC0.reborrow(),
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

        defmt::info!("Low, None");
        // Output is low, so a trigger should not result in a store to fifo
        adc.do_software_trigger(0b0001).unwrap();
        assert!(adc.wait_get_conversion().await.is_none());

        output.set_high();

        defmt::info!("High, Some");
        // Output is high, so a trigger should result in a store to fifo
        adc.do_software_trigger(0b0001).unwrap();
        assert!(adc.wait_get_conversion().await.is_some());
    }

    output.set_low();

    {
        defmt::info!("Skip until");

        let commands = &[Command::new_single(
            p.P2_4.reborrow(),
            CommandConfig {
                resolution: Mode::DATA_16_BITS,
                compare: adc::Compare::SkipUntil(adc::CompareFunction::GreaterThan(0x8000)),
                ..Default::default()
            },
        )];
        let mut adc = Adc::new_async(
            p.ADC0.reborrow(),
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

        defmt::info!("Low, Error");
        // Output is low, so we're never getting a result
        adc.do_software_trigger(0b0001).unwrap();
        embassy_time::Timer::after_millis(100).await;
        assert_eq!(adc.try_get_conversion(), Err(adc::Error::FifoPending));

        defmt::info!("High, Some");
        // Set output high, and we should get a value without new trigger
        output.set_high();
        assert!(adc.wait_get_conversion().await.is_some());
    }

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
