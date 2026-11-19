#![no_std]
#![no_main]

//! STM32H7 dual-core HSEM notification example, CM7 side.
//!
//! The CM7 notifies HSEM channel 5 every 500 ms and toggles the green LED (LD1, PB0).
//! The CM4 (`stm32h755cm4/src/bin/hsem_listen.rs`) waits on the same channel with `listen()` and
//! toggles the yellow LED (LD2, PE1) on every notification.
//!
//! Usage: flash the CM4 binary (`hsem_listen`), then this one.

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::hsem::{HardwareSemaphore, HardwareSemaphoreInterruptHandler};
use embassy_stm32::{Config, SharedData, bind_interrupts, peripherals};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    HSEM1 => HardwareSemaphoreInterruptHandler<peripherals::HSEM>;
});

#[unsafe(link_section = ".ram_d3")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default());
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            fracn: None,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8),
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }
    let p = embassy_stm32::init_primary(config, &SHARED_DATA);

    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);
    let [_, _, _, _, mut channel5, _] = hsem.split();
    let mut led = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut count = 0u32;
    loop {
        Timer::after_millis(500).await;
        channel5.blocking_notify();
        led.toggle();
        count = count.wrapping_add(1);
        info!("notified channel 5 ({})", count);
    }
}
