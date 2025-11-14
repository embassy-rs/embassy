#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276::clocks::periph_helpers::{AdcClockSel, Div4};
use embassy_mcxa276::clocks::PoweredClock;
use embassy_mcxa276::lpuart::{Config, Lpuart};
use embassy_mcxa276 as hal;
use hal::adc::{ConvResult, LpadcConfig, TriggerPriorityPolicy};
use mcxa_pac::adc1::cfg::{Pwrsel, Refsel};
use mcxa_pac::adc1::cmdl1::{Adch, Mode};
use mcxa_pac::adc1::ctrl::CalAvgs;
use mcxa_pac::adc1::tctrl::Tcmd;

mod common;

use core::fmt::Write;

use heapless::String;
use {defmt_rtt as _, panic_probe as _};

const G_LPADC_RESULT_SHIFT: u32 = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    unsafe {
        common::init_uart2(hal::pac());
    }

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with PIO2_2 as TX and PIO2_3 as RX
    unsafe {
        common::init_uart2(hal::pac());
    }
    let mut uart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.PIO2_2,  // TX pin
        p.PIO2_3,  // RX pin
        config,
    )
    .unwrap();

    uart.blocking_write(b"\r\n=== ADC polling Example ===\r\n").unwrap();

    unsafe {
        common::init_adc(hal::pac());
    }

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
    let adc = hal::adc::Adc::<hal::adc::Adc1>::new(p.ADC1, adc_config);

    adc.do_offset_calibration();
    adc.do_auto_calibration();

    let mut conv_command_config = adc.get_default_conv_command_config();
    conv_command_config.channel_number = Adch::SelectCorrespondingChannel8;
    conv_command_config.conversion_resolution_mode = Mode::Data16Bits;
    adc.set_conv_command_config(1, &conv_command_config);

    let mut conv_trigger_config = adc.get_default_conv_trigger_config();
    conv_trigger_config.target_command_id = Tcmd::ExecuteCmd1;
    conv_trigger_config.enable_hardware_trigger = false;
    adc.set_conv_trigger_config(0, &conv_trigger_config);

    uart.write_str_blocking("\r\n=== ADC configuration done... ===\r\n");

    loop {
        adc.do_software_trigger(1);
        let mut result: Option<ConvResult> = None;
        while result.is_none() {
            result = hal::adc::get_conv_result();
        }
        let value = result.unwrap().conv_value >> G_LPADC_RESULT_SHIFT;
        let mut buf: String<16> = String::new(); // adjust size as needed
        write!(buf, "\r\nvalue: {}\r\n", value).unwrap();
        uart.write_str_blocking(&buf);
    }
}
