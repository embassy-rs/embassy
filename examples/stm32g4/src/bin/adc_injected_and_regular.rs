//! adc injected and regular conversions
//!
//! This example both regular and injected ADC conversions at the same time
//! p:pa0 n:pa2

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::info;
use embassy_stm32::adc::{Adc, AdcChannel as _, Exten, RxDma, SampleTime};
use embassy_stm32::dma::ReadableRingBuffer;
use embassy_stm32::interrupt::typelevel::{ADC1_2, Interrupt};
use embassy_stm32::peripherals::{ADC1, DMA1_CH1};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, Mms2};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::{Config, interrupt};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use {critical_section, defmt_rtt as _, panic_probe as _};

static ADC1_HANDLE: CriticalSectionMutex<RefCell<Option<Adc<'static, ADC1>>>> =
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
    let mut adc1 = Adc::new(p.ADC1);
    let mut vrefint_channel = adc1.enable_vrefint().degrade_adc();
    let mut pa0 = p.PC1.degrade_adc();
    let regular_sequence = [
        (&mut vrefint_channel, SampleTime::CYCLES247_5),
        (&mut pa0, SampleTime::CYCLES247_5),
    ]
    .into_iter();
    adc1.configure_regular_sequence(regular_sequence);
    adc1.set_regular_conversion_trigger(ADC1_REGULAR_TRIGGER_TIM1_TRGO2, Exten::RISING_EDGE);

    // Configure DMA for retrieving regular ADC measurements
    let mut dma1_ch1 = p.DMA1_CH1;
    // Using a buffer larger than amount of samples gives more margin for timing of
    // main loop iterations and DMA transfers.
    let mut readings = [0u16; 4];
    let dma_request = <DMA1_CH1 as RxDma<ADC1>>::request(&dma1_ch1);
    let mut transfer = unsafe {
        ReadableRingBuffer::new(
            dma1_ch1.reborrow(),
            dma_request,
            adc1.get_data_register_address(),
            &mut readings,
            Default::default(),
        )
    };
    transfer.start();

    // Configurations of Injected ADC measurements
    let mut pa2 = p.PA2.degrade_adc();
    let injected_seq = [(&mut pa2, SampleTime::CYCLES247_5)].into_iter();
    adc1.configure_injected_sequence(injected_seq);
    adc1.set_injected_conversion_trigger(ADC1_INJECTED_TRIGGER_TIM1_TRGO2, Exten::RISING_EDGE);

    // ADC must be started after all configurations are completed
    adc1.start_regular_conversion();
    adc1.start_injected_conversion();

    // Enable interrupt at end of injected ADC conversion
    adc1.enable_injected_eos_interrupt(true);

    // Store ADC globally to allow access from ADC interrupt
    critical_section::with(|cs| {
        ADC1_HANDLE.borrow(cs).replace(Some(adc1));
    });
    // Enable interrupt for ADC1_2
    unsafe { ADC1_2::enable() };

    // Main loop for reading regular ADC measurements periodically
    let mut data = [0u16; 2];
    loop {
        {
            // No need to access the ADC object here, instead we get the readings from the DMA
            match transfer.read_exact(&mut data).await {
                Ok(n) => {
                    defmt::info!("Regular ADC reading, VrefInt: {}, PA0: {}", data[0], data[1]);
                    defmt::info!("Remaining samples: {}", n,);
                }
                Err(e) => {
                    defmt::error!("DMA error: {:?}", e);
                    transfer.clear();
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
        if let Some(adc) = ADC1_HANDLE.borrow(cs).borrow_mut().as_mut() {
            let injected_data = adc.clear_injected_eos();
            info!("Injected reading of PA2: {}", injected_data[0]);
        }
    });
}
