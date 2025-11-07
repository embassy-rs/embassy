#![no_std]
#![no_main]

use cortex_m;
use embassy_executor::Spawner;
use embassy_mcxa276 as hal;

use hal::adc::{LpadcConfig, TriggerPriorityPolicy};
use hal::pac::adc1::cfg::{Pwrsel, Refsel};
use hal::pac::adc1::cmdl1::{Adch, Mode};
use hal::pac::adc1::ctrl::CalAvgs;
use hal::pac::adc1::tctrl::Tcmd;

use hal::uart;
mod common;

use {defmt_rtt as _, panic_probe as _};

use hal::InterruptExt;
use hal::bind_interrupts;

bind_interrupts!(struct Irqs {
    ADC1 => hal::adc::AdcHandler;
});

#[used]
#[no_mangle]
static KEEP_ADC: unsafe extern "C" fn() = ADC1;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    unsafe {
        common::init_uart2(hal::pac());
    }

    let src = unsafe { hal::clocks::uart2_src_hz(hal::pac()) };
    let uart = uart::Uart::<uart::Lpuart2>::new(p.LPUART2, uart::Config::new(src));

    uart.write_str_blocking("\r\n=== ADC interrupt Example ===\r\n");

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

    adc.enable_interrupt(0x1);

    unsafe {
        hal::interrupt::ADC1.enable();
    }

    unsafe {
        cortex_m::interrupt::enable();
    }

    loop {
        adc.do_software_trigger(1);
        while !adc.is_interrupt_triggered() {
            // Wait until the interrupt is triggered
        }
        uart.write_str_blocking("\r\n*** ADC interrupt TRIGGERED! ***\r\n");
        //TBD need to print the value
    }
}
