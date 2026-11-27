//! ADC diagnostic: compare external ADC_IN2 (GPIO05) with internal
//! DAC_CORE_AOUT (raw sample channel 14).

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::adc::{Adc, AdcChannel, Config as AdcConfig, RawSampleChannel};
use embassy_asr::dac::{Config as DacConfig, Dac};
use embassy_asr6601_verify::{enable_debug_in_low_power, settle_analog};
use panic_halt as _;

const WORDS: usize = 48;
const MAGIC: u32 = 0x4144_4941; // "ADIA"
const DONE: u32 = 0x444f_4e45;

#[unsafe(no_mangle)]
pub static mut ADC_RESULTS: [u32; WORDS] = [0; WORDS];

fn word(index: usize) -> *mut u32 {
    unsafe { addr_of_mut!(ADC_RESULTS).cast::<u32>().add(index) }
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
    let external0 = RawSampleChannel::new(1).unwrap();
    let external2 = AdcChannel::new(p.PA5);
    let internal = RawSampleChannel::new(14).unwrap();

    let codes = [0u16, 128, 256, 512, 768, 1024, 1280, 1536];
    for (i, code) in codes.iter().enumerate() {
        put(2, i as u32);
        let _ = dac.set(*code);
        settle_analog();
        let ext0 = adc.blocking_read(external0).unwrap_or(0xffff);
        let ext2 = adc.blocking_read_channel(&external2).unwrap_or(0xffff);
        let int = adc.blocking_read(internal).unwrap_or(0xffff);
        put(3 + i * 4, u32::from(*code));
        put(4 + i * 4, u32::from(ext0));
        put(5 + i * 4, u32::from(ext2));
        put(6 + i * 4, u32::from(int));
    }

    put(1, DONE);
    loop {
        cortex_m::asm::wfi();
    }
}
