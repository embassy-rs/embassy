#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use defmt::info;
use core::cell::RefCell;

use embassy_stm32::{
    adc::{Adc, AdcChannel as _, Exten, SampleTime},
    interrupt,
    interrupt::typelevel::ADC1_2,
    peripherals::*,
    time::Hertz,
    timer::Channel,
    timer::complementary_pwm::{ComplementaryPwm, Mms2},
    timer::low_level::CountingMode,
    Config,
};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use critical_section;
use critical_section::Mutex;
use embassy_stm32::interrupt::typelevel::Interrupt;
use embassy_stm32::dma::Transfer;
use embassy_stm32::adc::RxDma;
use embassy_stm32::sai::word::WordSize;
use embassy_stm32::dma::Dir;
use embassy_stm32::dma::ReadableRingBuffer;

static ADC1_HANDLE: CriticalSectionMutex<RefCell<Option<Adc<'static, ADC1>>>> =
    CriticalSectionMutex::new(RefCell::new(None));

static COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));


#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    info!("init");

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

    // --- TIM1 setup ---
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
        CountingMode::CenterAlignedUpInterrupts,
    );
    pwm.set_master_output_enable(false);
    let max_duty = pwm.get_max_duty();
    pwm.set_duty(Channel::Ch4, max_duty - 1);
    pwm.set_mms2(Mms2::COMPARE_OC4);

    // --- ADC setup ---
    let mut adc1 = Adc::new(p.ADC1);
    let mut pa2 = p.PA2.degrade_adc();

    let injected_seq = [(&mut pa2, SampleTime::CYCLES247_5)].into_iter();
    adc1.configure_injected_sequence(injected_seq);
    let trigger = 8; // tim1_trgo2 for injected channels
    adc1.set_injected_conversion_trigger(trigger, Exten::RISING_EDGE);

    // Configure regular conversions with DMA
    let mut vrefint_channel = adc1.enable_vrefint().degrade_adc();
    let mut pa0 = p.PC1.degrade_adc();
    let regular_sequence = [
        (&mut vrefint_channel, SampleTime::CYCLES247_5),
        (&mut pa0, SampleTime::CYCLES247_5),
    ].into_iter();
    adc1.configure_regular_sequence(regular_sequence);

    let trigger_regular = 10; // Regular and Injected channels use different trigger definitions
    adc1.set_regular_conversion_trigger(trigger_regular, Exten::RISING_EDGE);
    let adc1_addr = adc1.get_dma_data_pointer();

    let mut dma1_ch1 = p.DMA1_CH1;
    let mut readings = [0u16; 256];
    let request = <DMA1_CH1 as embassy_stm32::adc::RxDma<embassy_stm32::peripherals::ADC1>>::request(&dma1_ch1);

    let mut transfer = unsafe { ReadableRingBuffer::new(
        dma1_ch1.reborrow(),
        request,
        adc1_addr,
        &mut readings,
        Default::default(),
    )};
    transfer.start();

    adc1.start_regular_conversion();
    // Store ADC globally so IRQ can access it
    critical_section::with(|cs| {
        ADC1_HANDLE.borrow(cs).replace(Some(adc1));
    });
    unsafe { ADC1_2::enable() };

    // Let main task idle (could do something else)
    let mut data = [0u16; 2];
    loop {
        {
            match transfer.read(&mut data) {
                Ok((0, _)) => {},
                Ok((n, m)) => {
                    defmt::info!("data: {:?}, n: {}, m: {}", &data, n, m);
                },
                res => {defmt::info!("res: {:?}", res);}
            }
            //transfer.clear();

        }

        embassy_time::Timer::after_millis(120).await;
    }
}


#[interrupt]
unsafe fn ADC1_2() {
    critical_section::with(|cs| {
        if let Some(adc) = ADC1_HANDLE.borrow(cs).borrow_mut().as_mut() {
            let injected_data = adc.clear_injected_eos();

            let mut count_ref = COUNT.borrow(cs).borrow_mut();
            *count_ref += 1;
            if *count_ref > 500 {
                *count_ref = 0;
                info!("u_dc: {}", injected_data[0]);
            }
        }
    });
}

