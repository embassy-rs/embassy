//! SimplePWM example with STOP mode using SimplePwm and a
//! Advance Control timer.
//!
//! The MCU enters STOP2 mode between LED toggles (5 s intervals).
//! My board draws about 160µA while sleeping, which is due to hardware
//! limitations. Actual MCU current will be measure once i test on
//! NUCLEO board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

use embassy_stm32::gpio::{Flex, Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::time;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};

const DEBUG_DURING_SLEEP: bool = false;

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using simplePwm!");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;

        // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: Some(0x0C),
        });
        // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
        config.rcc.ls = LsConfig {
            rtc: RtcClockSource::Lse,
            lsi: false,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumLow),
                peripherals_clocked: true,
            }),
        };
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),             // Fractional part (disabled)
        });

        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div1;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.apb7_pre = APBPrescaler::Div1;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;

        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
        config.rcc.mux.lptim2sel = mux::Lptim2sel::Hsi;
        config.rcc.mux.rngsel = mux::Rngsel::Hsi;
        config.rcc.sys = Sysclk::Pll1R;

        config.enable_debug_during_sleep = DEBUG_DURING_SLEEP;
        config.min_stop_pause = embassy_time::Duration::from_millis(10);
    }

    let mut p = embassy_stm32::init(config);

    info!("initializing unused GPIOs for minimum current draw ...");
    let _gpio_pd5 = Output::new(p.PD5, Level::Low, Speed::Low);
    let _gpio_pb10 = Output::new(p.PB10, Level::High, Speed::Low);
    let _gpio_pa6 = Input::new(p.PA6, Pull::Up);
    let _gpio_pe1 = Output::new(p.PE1, Level::High, Speed::VeryHigh);
    let _gpio_pe3 = Output::new(p.PE3, Level::High, Speed::VeryHigh);
    let _gpio_pe0 = Output::new(p.PE0, Level::Low, Speed::VeryHigh);
    let _gpio_pd14 = Output::new(p.PD14, Level::Low, Speed::VeryHigh);
    let mut flex_pd8 = Flex::new(p.PD8);
    flex_pd8.set_as_analog();
    let _gpio_ph3 = Output::new(p.PH3, Level::Low, Speed::Low);

    info!("initializing simplePwm and power rail");
    let mut power_rail = Output::new(p.PB15, Level::Low, Speed::Low);

    // STM32WBA65 has 3 types of timers:
    //  - General Purpose(TIM2/3/4/16/17)
    //  - AdvancedControl(TIM1)
    //  - LowPower(LPTIM1/2)

    loop {
        {
            info!("begin");
            info!("initializing SimplePwm with timer TIM1");
            let led_pwm_pin = PwmPin::new(p.PB8.reborrow(), OutputType::PushPull);
            let mut led_pwm = SimplePwm::new(
                p.TIM1.reborrow(),
                Some(led_pwm_pin),
                None,
                None,
                None,
                time::khz(100),
                CountingMode::EdgeAlignedUp,
            );
            // blocking STOP mode entry ... why is this needed?
            // let _stay_awake = WakeGuard::new(StopMode::Stop1);

            power_rail.set_high();
            led_pwm.ch1().enable();
            led_pwm.ch1().set_duty_cycle_percent(30);
            info!("led on — staying awake so PWM keeps toggling");

            Timer::after_millis(2000).await;

            info!("led off — sleeping 5 s");
            led_pwm.ch1().disable();
            led_pwm.ch1().set_duty_cycle_percent(0);
            drop(led_pwm);
            power_rail.set_low();
            info!("end");
        }
        Timer::after_millis(2000).await; // MCU enters STOP here
    }
}
