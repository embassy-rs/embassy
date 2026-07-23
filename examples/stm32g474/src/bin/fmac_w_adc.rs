#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, Exten, RegularAdcTrigger, SampleTime};
use embassy_stm32::fmac::{self, Q16};
use embassy_stm32::hrtim::stm32_hrtim::{HrControltExt, HrPwmBuilderExt, Parts};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, triggers};
use stm32_hrtim::HrPwmAdvExt;
use stm32_hrtim::compare_register::HrCompareRegister;
use stm32_hrtim::output::NoPin;
use stm32_hrtim::timer::HrTimer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        // Set system frequency to 16MHz * 15/1/2 = 120MHz
        // This would lead to HrTim running at 120MHz * 32 = 3.84GHz...
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::Div2),
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul15,
        });
        config.rcc.mux.adc12sel = mux::Adcsel::Sys;
        config.rcc.sys = Sysclk::Pll1R;
    }

    let mut p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, Default::default());

    let mut temperature = adc.enable_temperature();

    let one_third = Q16::from_f32(1.0 / 3.0);

    let feedforward_weights = [one_third, one_third, one_third];

    // Create a second order FIR filter
    //
    // This will calculate
    // result = input_history[0] * latest_input +
    //     input_history[1] * older_input +
    //     input_history[2] * oldest_input
    let mut fmac = fmac::Fmac::fir(
        p.FMAC.reborrow(),
        fmac::Config {
            output_mode: fmac::OutputMode::Saturating,
            read_method: fmac::AccessMethod::Poll,
            write_method: fmac::AccessMethod::Poll,
        },
        None,
        &feedforward_weights,
        fmac::Gain::X1,
    );
    let trigger = RegularAdcTrigger::from(triggers::HRTIM_ADC_TRG1, Exten::RisingEdge).unwrap();

    let mut from_adc = fmac::FromAdc::new(
        &mut fmac,
        &mut adc,
        [(temperature.reborrow_adc(), SampleTime::Cycles6405)].into_iter(),
        trigger,
        p.DMA1_CH1,
        Irqs,
    );

    // ...with a prescaler of 4 this gives us a HrTimer with a tick rate of 30MHz
    // With max the max period set, this would be 30MHz/2^16 ~= 458Hz...
    let prescaler = stm32_hrtim::Pscl128;
    let period = 0xFFFF;

    // ... and with an adc trigger postscaler of 32, we get adc triggers at a rate of about 458Hz/32=14Hz
    let Parts { control, tima, .. } = p.HRTIM1.hr_control();
    let (control, ..) = control
        .set_adc1_trigger_psc(stm32_hrtim::control::AdcTriggerPostscaler::Div32)
        .wait_for_calibration();
    let mut control = control.constrain();

    let mut tim = tima
        .pwm_advanced(NoPin, NoPin)
        .prescaler(prescaler)
        .period(period)
        .finalize(&mut control);

    tim.cr3.set_duty(period / 2);
    control.adc_trigger1.enable_source(&tim.cr3);
    tim.timer.start(&mut control.control);

    defmt::assert!(from_adc.read().is_none()); // <-- Not enough data yet
    defmt::println!("Running!");
    loop {
        if let Some(raw) = from_adc.read() {
            // raw is in Q1.15 format, convert back to u16
            let filtered_value = raw.into_bits();
            defmt::println!("reading: {}", filtered_value);
        }
    }
}
