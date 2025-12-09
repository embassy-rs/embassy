#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::adc::{Adc, InterruptHandler, LpadcConfig, TriggerPriorityPolicy};
use hal::bind_interrupts;
use hal::clocks::PoweredClock;
use hal::clocks::config::Div8;
use hal::clocks::periph_helpers::{AdcClockSel, Div4};
use hal::config::Config;
use hal::pac::adc1::cfg::{Pwrsel, Refsel};
use hal::pac::adc1::cmdl1::{Adch, Mode};
use hal::pac::adc1::ctrl::CalAvgs;
use hal::pac::adc1::tctrl::Tcmd;
use hal::peripherals::ADC1;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1 => InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("ADC interrupt Example");

    let adc_config = LpadcConfig {
        enable_in_doze_mode: true,
        conversion_average_mode: CalAvgs::Average128,
        enable_analog_preliminary: true,
        power_up_delay: 0x80,
        reference_voltage_source: Refsel::Option3,
        power_level_mode: Pwrsel::Lowest,
        trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
        enable_conv_pause: false,
        conv_pause_delay: 0,
        fifo_watermark: 0,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        source: AdcClockSel::FroLfDiv,
        div: Div4::no_div(),
    };
    let mut adc = Adc::new_async(p.ADC1, p.P1_10, Irqs, adc_config).unwrap();

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    let mut conv_command_config = adc.get_default_conv_command_config();
    conv_command_config.channel_number = Adch::SelectCorrespondingChannel8;
    conv_command_config.conversion_resolution_mode = Mode::Data16Bits;
    adc.set_conv_command_config(1, &conv_command_config).unwrap();

    let mut conv_trigger_config = adc.get_default_conv_trigger_config();
    conv_trigger_config.target_command_id = Tcmd::ExecuteCmd1;
    conv_trigger_config.enable_hardware_trigger = false;
    adc.set_conv_trigger_config(0, &conv_trigger_config);

    defmt::info!("ADC configuration done...");

    loop {
        match adc.read().await {
            Ok(value) => {
                defmt::info!("*** ADC interrupt TRIGGERED! *** -- value: {}", value);
            }
            Err(e) => {
                defmt::error!("ADC read error: {:?}", e);
            }
        }
    }
}
