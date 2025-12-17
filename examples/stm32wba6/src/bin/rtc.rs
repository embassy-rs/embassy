#![no_std]
#![no_main]

use defmt::*;
use embassy_examples_common::rtc_api::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::*;
use embassy_stm32::rtc::{DateTime as EmbassyDateTime, DayOfWeek, Rtc as EmbassyRtc, RtcConfig, RtcTimeProvider};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

pub fn pll_init(config: &mut Config) {
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        // divq: Some(PllDiv::DIV10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (USBOTG)
        frac: Some(0),             // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;

    // voltage scale for max performance
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    // route PLL1_P into the USB‐OTG‐HS block
    config.rcc.sys = Sysclk::PLL1_R;
}

// STM32-specific wrapper that implements RtcInstance
pub struct Stm32Rtc {
    rtc_tuple: (EmbassyRtc, RtcTimeProvider),
}

impl Stm32Rtc {
    pub fn new(rtc_tuple: (EmbassyRtc, RtcTimeProvider)) -> Self {
        Self { rtc_tuple }
    }

    fn embassy_to_api_datetime(embassy_dt: &EmbassyDateTime) -> DateTime {
        DateTime {
            year: embassy_dt.year(),
            month: embassy_dt.month(),
            day: embassy_dt.day(),
            week_day: embassy_dt.day_of_week() as u8,
            hour: embassy_dt.hour(),
            minute: embassy_dt.minute(),
            second: embassy_dt.second(),
        }
    }

    fn api_to_embassy_datetime_impl(api_dt: &DateTime) -> Result<EmbassyDateTime, RtcError> {
        let day_of_week = match api_dt.week_day {
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            6 => DayOfWeek::Saturday,
            7 => DayOfWeek::Sunday,
            _ => return Err(RtcError::InvalidInput),
        };

        EmbassyDateTime::from(
            api_dt.year,
            api_dt.month,
            api_dt.day,
            day_of_week,
            api_dt.hour,
            api_dt.minute,
            api_dt.second,
            0, // microseconds
        )
        .map_err(|_| RtcError::InvalidInput)
    }
}

impl RtcInstance for Stm32Rtc {
    fn set_date_time(&mut self, new_date_time: DateTime) -> Result<(), RtcError> {
        println!("set_date_time ...");
        let embassy_dt = Self::api_to_embassy_datetime_impl(&new_date_time)?;
        println!("setting {:?} ", embassy_dt);
        self.rtc_tuple
            .0
            .set_datetime(embassy_dt)
            .map_err(|_| RtcError::HardwareError)
    }

    fn get_date_time(&mut self) -> Result<DateTime, RtcError> {
        self.rtc_tuple
            .1
            .now()
            .map(|dt| Self::embassy_to_api_datetime(&dt))
            .map_err(|_| RtcError::HardwareError)
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    pll_init(&mut config);

    let p = embassy_stm32::init(config);

    let rtc_tuple = EmbassyRtc::new(p.RTC, RtcConfig::default());

    let stm32_rtc = Stm32Rtc::new(rtc_tuple);
    let mut my_rtc = Rtc::new(stm32_rtc);

    // Setting datetime using API format
    let initial_datetime = DateTime {
        year: 2022,
        month: 12,
        day: 18,
        week_day: 7, // Saturday
        hour: 0,
        minute: 0,
        second: 0,
    };

    match my_rtc.set_date_time(initial_datetime) {
        Ok(()) => info!("RTC set successfully."),
        Err(e) => error!("Failed to set RTC date/time: {:?}", e),
    }

    // Reading datetime every 1s
    loop {
        match my_rtc.get_date_time() {
            Ok(result) => info!(
                "Date: {} {}/{}/{} Time: {}:{}:{}",
                result.week_day, result.year, result.month, result.day, result.hour, result.minute, result.second
            ),
            Err(e) => error!("Failed to get RTC date/time: {:?}", e),
        }
        Timer::after_millis(1000).await;
    }
}
