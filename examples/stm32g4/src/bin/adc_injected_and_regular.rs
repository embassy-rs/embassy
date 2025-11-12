//! adc injected and regular conversions
//!
//! This example both regular and injected ADC conversions at the same time
//! p:pa0 n:pa2

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::info;
use embassy_stm32::adc::{
    Adc, AdcChannel as _, ConversionTrigger, Exten, InjectedAdc, RegularConversionMode, SampleTime,
};
use embassy_stm32::interrupt::typelevel::{ADC1_2, Interrupt};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, Mms2};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::{Config, interrupt};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use {defmt_rtt as _, panic_probe as _};

static ADC1_HANDLE: CriticalSectionMutex<RefCell<Option<InjectedAdc<ADC1, 1>>>> =
    CriticalSectionMutex::new(RefCell::new(None));

/// This example showcases how to use both regular ADC conversions with DMA and injected ADC
/// conversions with ADC interrupt simultaneously. Both conversion types can be configured with
/// different triggers and thanks to DMA it is possible to use the measurements in different task
/// without needing to access the ADC peripheral.
///
/// If you don't need both regular and injected conversions the example code can easily be reworked
/// to only include one of the ADC conversion types.
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    // See Table 166 and 167 in RM0440 Rev 9 for ADC1/2 External triggers
    // Note: Regular and Injected channels use different tables!!
    const ADC1_INJECTED_TRIGGER_TIM1_TRGO2: u8 = 8;
    const ADC1_REGULAR_TRIGGER_TIM1_TRGO2: u8 = 10;

    // --- RCC config ---
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);

    // In this example we use tim1_trgo2 event to trigger the ADC conversions
    let tim1 = p.TIM1;
    let pwm_freq = 1;
    let mut pwm = ComplementaryPwm::new(
        tim1,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Hertz::hz(pwm_freq),
        CountingMode::EdgeAlignedUp,
    );
    pwm.set_master_output_enable(false);
    // Mms2 is used to configure which timer event that is connected to tim1_trgo2.
    // In this case we use the update event of the timer.
    pwm.set_mms2(Mms2::UPDATE);

    // Configure regular conversions with DMA
    let adc1 = Adc::new(p.ADC1, Default::default());

    let vrefint_channel = adc1.enable_vrefint().degrade_adc();
    let pa0 = p.PC1.degrade_adc();
    let regular_sequence = [
        (vrefint_channel, SampleTime::CYCLES247_5),
        (pa0, SampleTime::CYCLES247_5),
    ]
    .into_iter();

    // Configurations of Injected ADC measurements
    let pa2 = p.PA2.degrade_adc();
    let injected_sequence = [(pa2, SampleTime::CYCLES247_5)];

    // Configure DMA for retrieving regular ADC measurements
    let dma1_ch1 = p.DMA1_CH1;
    // Using buffer of double size means the half-full interrupts will generate at the expected rate
    let mut readings = [0u16; 4];

    let injected_trigger = ConversionTrigger {
        channel: ADC1_INJECTED_TRIGGER_TIM1_TRGO2,
        edge: Exten::RISING_EDGE,
    };
    let regular_trigger = ConversionTrigger {
        channel: ADC1_REGULAR_TRIGGER_TIM1_TRGO2,
        edge: Exten::RISING_EDGE,
    };

    let (mut ring_buffered_adc, injected_adc) = adc1.into_ring_buffered_and_injected(
        dma1_ch1,
        &mut readings,
        regular_sequence,
        RegularConversionMode::Triggered(regular_trigger),
        injected_sequence,
        injected_trigger,
        true,
    );

    // Store ADC globally to allow access from ADC interrupt
    critical_section::with(|cs| {
        ADC1_HANDLE.borrow(cs).replace(Some(injected_adc));
    });
    // Enable interrupt for ADC1_2
    unsafe { ADC1_2::enable() };

    // Main loop for reading regular ADC measurements periodically
    let mut data = [0u16; 2];
    loop {
        {
            match ring_buffered_adc.read(&mut data).await {
                Ok(n) => {
                    defmt::info!("Regular ADC reading, VrefInt: {}, PA0: {}", data[0], data[1]);
                    defmt::info!("Remaining samples: {}", n,);
                }
                Err(e) => {
                    defmt::error!("DMA error: {:?}", e);
                    ring_buffered_adc.clear();
                }
            }
        }
    }
}

/// Use ADC1_2 interrupt to retrieve injected ADC measurements
/// Interrupt must be unsafe as hardware can invoke it any-time. Critical sections ensure safety
/// within the interrupt.
#[interrupt]
unsafe fn ADC1_2() {
    critical_section::with(|cs| {
        if let Some(injected_adc) = ADC1_HANDLE.borrow(cs).borrow_mut().as_mut() {
            let injected_data = injected_adc.read_injected_samples();
            info!("Injected reading of PA2: {}", injected_data[0]);
        }
    });
}
