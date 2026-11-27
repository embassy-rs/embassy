//! ADC/DAC and GPIO loopback verification.
//!
//! Wiring:
//! * DAC GPIO09 (PA9) -> ADC GPIO05 (PA5 / ADC_IN2)
//! * GPIO08 (PA8) output -> GPIO04 (PA4) input
//!
//! Results are written to `LOOP_RESULTS` in SRAM.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::adc::{Adc, AdcChannel, Config as AdcConfig};
use embassy_asr::dac::{Config as DacConfig, Dac};
use embassy_asr::gpio::{Flex, Level, Output, Pull};
use embassy_asr::pac;
use embassy_asr6601_verify::{enable_debug_in_low_power, settle_analog};
use panic_halt as _;

const WORDS: usize = 64;
const MAGIC: u32 = 0x4c4f_4f50; // "LOOP"
const DONE: u32 = 0x444f_4e45;

#[unsafe(no_mangle)]
pub static mut LOOP_RESULTS: [u32; WORDS] = [0; WORDS];

fn word(index: usize) -> *mut u32 {
    unsafe { addr_of_mut!(LOOP_RESULTS).cast::<u32>().add(index) }
}

fn put(index: usize, value: u32) {
    unsafe { write_volatile(word(index), value) }
}

#[entry]
fn main() -> ! {
    put(0, MAGIC);
    put(1, 1);

    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();

    let mut dac = Dac::new(p.DAC, p.PA9, DacConfig::default());
    let mut adc = Adc::new_blocking(p.ADC, AdcConfig::default()).unwrap();
    let channel = AdcChannel::new(p.PA5);

    let codes = [0u16, 128, 256, 512, 768, 1024, 1280, 1536];
    for (i, code) in codes.iter().enumerate() {
        put(2, i as u32);
        let _ = dac.set(*code);
        settle_analog();
        let value = adc.blocking_read_channel(&channel).unwrap_or(0xffff);
        put(3 + i * 2, u32::from(*code));
        put(4 + i * 2, u32::from(value));
    }

    put(2, 0x100);
    let mut out = Output::new(p.PA8, Level::Low);
    let mut input = Flex::new(p.PA4);
    input.set_as_input();
    input.set_pull(Pull::None);

    let gpio = unsafe { pac::Gpioa::steal() };
    unsafe {
        gpio.fr().write_with_zero(|w| w.bits(0xffff_ffff));
    }
    gpio.int_cr().modify(|_, w| {
        w.pos_int_en4().set_bit();
        w.neg_int_en4().clear_bit();
        w
    });

    out.set_low();
    settle_analog();
    let low = u32::from(input.is_high());

    out.set_high();
    settle_analog();
    let high = u32::from(input.is_high());
    let pos_flag = u32::from(gpio.fr().read().pos_f4().bit_is_set());

    out.set_low();
    settle_analog();
    let low2 = u32::from(input.is_high());

    gpio.int_cr().modify(|_, w| {
        w.pos_int_en4().clear_bit();
        w
    });
    unsafe {
        gpio.fr().write_with_zero(|w| w.bits(0xffff_ffff));
    }

    put(20, 0x200);
    put(21, low);
    put(22, high);
    put(23, pos_flag);
    put(24, low2);
    put(25, u32::from(input.is_high()));
    put(26, gpio.idr().read().bits());
    put(27, gpio.int_cr().read().bits());

    put(1, DONE);
    loop {
        cortex_m::asm::wfi();
    }
}
