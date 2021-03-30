#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::{panic, *};

use bxcan::filter::Mask32;
use cortex_m_rt::entry;
use embassy::executor::Executor;
use embassy::util::Forever;
use embassy_stm32::{can, interrupt};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{can::Can, stm32};

#[embassy::task]
async fn run(dp: stm32::Peripherals, _cp: cortex_m::Peripherals) {
    let gpioa = dp.GPIOA.split();

    let rx = gpioa.pa11.into_alternate_af9();
    let tx = gpioa.pa12.into_alternate_af9();
    let mut can = bxcan::Can::new(Can::new(dp.CAN1, (tx, rx)));

    // APB1 (PCLK1): 24MHz, Bit rate: 20kBit/s, Sample Point 87.5%
    // Value was calculated with http://www.bittiming.can-wiki.info/
    can.modify_config().set_bit_timing(0x001c_004a);
    // Configure filters so that can frames can be received.
    can.modify_filters().enable_bank(0, Mask32::accept_all());

    let mut can = can::Can::new(can, interrupt::take!(CAN1_TX), interrupt::take!(CAN1_RX0));

    let _frame = can.receive().await;
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run(dp, cp)));
    });
}
