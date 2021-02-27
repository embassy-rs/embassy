//! Async I2C example with AXP173 chip.
#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::panic;

use cortex_m::singleton;
use cortex_m_rt::entry;

use embassy::executor::{task, Executor};
use embassy::util::Forever;

use embassy_stm32wb55::i2c::i2c1::AsyncI2c as AsyncI2c1;
use embassy_stm32wb55::interrupt;

use axp173::*;

use stm32wb_hal::dma::DmaExt;
use stm32wb_hal::flash::FlashExt;
use stm32wb_hal::prelude::*;
use stm32wb_hal::rcc::{
    ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, RfWakeupClock, RtcClkSrc,
    StopWakeupClock, SysClkSrc,
};
use stm32wb_hal::{i2c::I2c, stm32};

#[task]
async fn run(dp: stm32::Peripherals, _cp: cortex_m::Peripherals) {
    // Allow using debugger and RTT during WFI/WFE (sleep)
    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    let mut rcc = dp.RCC.constrain();
    rcc.set_stop_wakeup_clock(StopWakeupClock::HSI16);

    // Fastest clock configuration.
    // * External low-speed crystal is used (LSE)
    // * 32 MHz HSE with PLL
    // * 64 MHz CPU1, 32 MHz CPU2
    // * 64 MHz for APB1, APB2
    // * HSI as a clock source after wake-up from low-power mode
    let clock_config = Config::new(SysClkSrc::Pll(PllSrc::Hse(HseDivider::NotDivided)))
        .with_lse()
        .cpu1_hdiv(HDivider::NotDivided)
        .cpu2_hdiv(HDivider::Div2)
        .apb1_div(ApbDivider::NotDivided)
        .apb2_div(ApbDivider::NotDivided)
        .pll_cfg(PllConfig {
            m: 2,
            n: 12,
            r: 3,
            q: Some(4),
            p: Some(3),
        })
        .rtc_src(RtcClkSrc::Lse)
        .rf_wkp_sel(RfWakeupClock::Lse);

    let mut rcc = rcc.apply_clock_config(clock_config, &mut dp.FLASH.constrain().acr);

    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut pull_ups = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let _ = pull_ups.set_high();

    let scl = gpiob
        .pb6
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob
        .pb7
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), &mut rcc);

    let dma_channels = dp.DMA1.split(&mut rcc, dp.DMAMUX1);
    let dma_c1 = dma_channels.ch1;

    let buf = singleton!(: [u8; 128] = [0; 128]).unwrap();

    let async_i2c = AsyncI2c1::new(buf, i2c, interrupt::take!(DMA1_CHANNEL1), dma_c1);

    let mut axp173 = axp173::Axp173::new(async_i2c);
    axp173.init().await.unwrap();

    // Set charging current to 100mA
    axp173
        .set_charging_current(ChargingCurrent::CURRENT_100MA)
        .await
        .unwrap();

    // 25Hz sample rate, Disable TS, enable current sensing ADC
    axp173
        .set_adc_settings(
            AdcSettings::default()
                .set_adc_sample_rate(AdcSampleRate::RATE_25HZ)
                .ts_adc(false)
                .set_ts_pin_mode(TsPinMode::SHUT_DOWN)
                .vbus_voltage_adc(true)
                .vbus_current_adc(true)
                .batt_voltage_adc(true)
                .batt_current_adc(true),
        )
        .await
        .unwrap();

    axp173.set_coulomb_counter(true).await.unwrap();
    axp173.resume_coulomb_counter().await.unwrap();

    axp173
        .set_shutdown_long_press_time(ShutdownLongPressTime::SEC_4)
        .await
        .unwrap();
    axp173.set_shutdown_long_press(false).await.unwrap();

    axp173
        .enable_ldo(&Ldo::ldo3_with_voltage(10, true))
        .await
        .unwrap();
    axp173.disable_ldo(&LdoKind::LDO4).await.unwrap();

    // Is the device connected to the USB power supply?
    if axp173.vbus_present().await.unwrap() {
        defmt::info!("VBUS is present");
    } else {
        defmt::info!("VBUS is not present");
    }

    // Is the battery connected to the device?
    if axp173.battery_present().await.unwrap() {
        defmt::info!("Battery is present");
    } else {
        defmt::info!("Battery is not present");
    }

    // Is the battery currently being charged?
    if axp173.battery_charging().await.unwrap() {
        defmt::info!("Battery is charging");
    } else {
        defmt::info!("Battery is not charging");
    }

    let vbus = axp173.vbus_voltage().await.unwrap();
    defmt::info!("VBUS: {}", vbus.as_volts());

    let vbus = axp173.vbus_current().await.unwrap();
    defmt::info!("VBUS current: {} mA", vbus.as_milliamps());

    let batt = axp173.batt_voltage().await.unwrap();
    defmt::info!("Batt: {} V", batt.as_volts());

    let batt_charge = axp173.batt_charge_current().await.unwrap();
    let batt_discharge = axp173.batt_discharge_current().await.unwrap();
    defmt::info!(
        "Batt: ^ {} mA | v {} mA",
        batt_charge.as_milliamps(),
        batt_discharge.as_milliamps()
    );

    defmt::info!(
        "Charge coulombs: {}",
        axp173.read_charge_coulomb_counter().await.unwrap()
    );
    defmt::info!(
        "Discharge coulombs: {}",
        axp173.read_discharge_coulomb_counter().await.unwrap()
    );

    let ldo = axp173.read_ldo(LdoKind::LDO2).await.unwrap();
    defmt::info!(
        "LDO2: enabled: {}, voltage: {} V",
        ldo.enabled(),
        ldo.voltage().0 as f32 / 1000.0,
    );

    let ldo = axp173.read_ldo(LdoKind::LDO3).await.unwrap();
    defmt::info!(
        "LDO3: enabled: {}, voltage: {} V",
        ldo.enabled(),
        ldo.voltage().0 as f32 / 1000.0,
    );

    let ldo = axp173.read_ldo(LdoKind::LDO4).await.unwrap();
    defmt::info!(
        "LDO4: enabled: {}, voltage: {} V",
        ldo.enabled(),
        ldo.voltage().0 as f32 / 1000.0,
    );

    loop {
        cortex_m::asm::nop();
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    defmt::info!("Starting");

    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run(dp, cp)).unwrap();
    });
}
