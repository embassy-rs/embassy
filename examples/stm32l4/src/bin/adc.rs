#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy_stm32::gpio::{Input, Level, NoPin, Output, Pull};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use example_common::*;

use cortex_m_rt::entry;
//use stm32f4::stm32f429 as pac;
use cortex_m::delay::Delay;
use embassy_stm32::adc::{Adc, Resolution};
use embassy_stm32::dac::{Channel, Dac, Value};
use embassy_stm32::spi::{ByteOrder, Config, Spi, MODE_0};
use embassy_stm32::time::Hertz;
use embedded_hal::blocking::spi::Transfer;
use micromath::F32Ext;
use stm32l4::stm32l4x5 as pac;
use stm32l4xx_hal::gpio::PA4;
use stm32l4xx_hal::rcc::PllSource;
use stm32l4xx_hal::{prelude::*, rcc};

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");
    //let pp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let pp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();
    let mut flash = pp.FLASH.constrain();
    let mut rcc = pp.RCC.constrain();
    let mut pwr = pp.PWR.constrain(&mut rcc.apb1r1);

    let mut delay = Delay::new(cp.SYST, 80_000_000);

    // TRY the other clock configuration
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .pll_source(PllSource::HSI16)
        .freeze(&mut flash.acr, &mut pwr);

    let pp = unsafe { pac::Peripherals::steal() };

    pp.RCC.ccipr.modify(|_, w| {
        unsafe {
            w.adcsel().bits(0b11);
        }
        w
    });

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    pp.RCC.ahb2enr.modify(|_, w| {
        w.adcen().set_bit();
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w.gpiofen().set_bit();
        w
    });

    let p = embassy_stm32::init(Default::default());

    let (mut adc, mut delay) = Adc::new(p.ADC1, delay);
    //adc.enable_vref();
    adc.set_resolution(Resolution::EightBit);
    let mut channel = p.PC0;

    loop {
        let v = adc.read(&mut channel);
        info!("--> {}", v);
    }
}

fn to_sine_wave(v: u8) -> u8 {
    if v >= 128 {
        // top half
        let r = 3.14 * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}
