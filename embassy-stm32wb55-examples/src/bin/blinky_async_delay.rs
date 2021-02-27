//! Async I2C example with AXP173 chip.
#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::panic;

use cortex_m_rt::entry;

use async_embedded_traits::delay::AsyncDelayMs;

use embassy::executor::{task, Executor};
use embassy::util::Forever;

use embassy_stm32wb55::interrupt;

use stm32wb_hal::flash::FlashExt;
use stm32wb_hal::lptim::{lptim1::LpTimer as LpTimer1, lptim2::LpTimer as LpTimer2};
use stm32wb_hal::prelude::*;
use stm32wb_hal::rcc::{
    ApbDivider, Config, HDivider, HseDivider, LptimClkSrc, PllConfig, PllSrc, RfWakeupClock,
    RtcClkSrc, StopWakeupClock, SysClkSrc,
};
use stm32wb_hal::stm32;

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
        .rf_wkp_sel(RfWakeupClock::Lse)
        .lptim1_src(LptimClkSrc::Lse)
        .lptim2_src(LptimClkSrc::Lse);

    let mut rcc = rcc.apply_clock_config(clock_config, &mut dp.FLASH.constrain().acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut green_led = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let lptim = LpTimer2::init_oneshot(dp.LPTIM2, &mut rcc);
    let mut delay =
        embassy_stm32wb55::delay::lptim2::LptimDelay::new(lptim, interrupt::take!(LPTIM2));
    loop {
        green_led.set_low().unwrap();
        defmt::info!("Blink");
        delay.async_delay_ms(1000_u32).await;

        green_led.set_high().unwrap();
        defmt::info!("Blonk");
        delay.async_delay_ms(1000_u32).await;
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
