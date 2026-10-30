//! Safe, output-free hardware verification for the ASR6601SE.
//!
//! Results are written to `VERIFY_RESULTS` in SRAM. The host reads them with
//! `probe-rs read` after the firmware reaches the final WFI loop.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::adc::{Adc, AdcChannel, Config as AdcConfig, SampleChannel};
use embassy_asr::crc::{Config as CrcConfig, Crc};
use embassy_asr::dac::{Config as DacConfig, Dac};
use embassy_asr::dma::{Channel, TransferOptions};
use embassy_asr::gpio::{Flex, Level, Output, Pull};
use embassy_asr::pac;
use embassy_asr::syscfg::Syscfg;
use embassy_asr::timer::{Config as TimerConfig, InterruptFlags as TimerFlags, Timer};
use embassy_asr6601_verify::{enable_debug_in_low_power, settle_gpio};
use panic_halt as _;

const MAX_TESTS: usize = 24;
const RESULT_WORDS: usize = 3 + MAX_TESTS * 4;

const MAGIC: u32 = 0x4153_5256; // "ASRV"
const DONE: u32 = 0x444f_4e45; // "DONE"

const RUNNING: u32 = 0;
const PASS: u32 = 1;
const FAIL: u32 = 2;
const SKIP: u32 = 3;

#[unsafe(no_mangle)]
pub static mut VERIFY_RESULTS: [u32; RESULT_WORDS] = [0; RESULT_WORDS];

fn word(index: usize) -> *mut u32 {
    // SAFETY: `index` is always bounded by the callers below.
    unsafe { addr_of_mut!(VERIFY_RESULTS).cast::<u32>().add(index) }
}

fn write_word(index: usize, value: u32) {
    unsafe { write_volatile(word(index), value) }
}

fn read_word(index: usize) -> u32 {
    unsafe { read_volatile(word(index)) }
}

fn begin(index: usize, id: u32) {
    let slot = 3 + index * 4;
    write_word(1, index as u32 + 1);
    write_word(slot, id);
    write_word(slot + 1, RUNNING);
    write_word(slot + 2, 0);
    write_word(slot + 3, 0);
}

fn finish(index: usize, status: u32, detail0: u32, detail1: u32) {
    let slot = 3 + index * 4;
    write_word(slot + 2, detail0);
    write_word(slot + 3, detail1);
    write_word(slot + 1, status);
}

macro_rules! test {
    ($index:expr, $id:expr, $body:block) => {{
        begin($index, $id);
        let (status, d0, d1) = $body;
        finish($index, status, d0, d1);
    }};
}

fn or_probe(ptr: *const pac::gptim0::RegisterBlock, expected: u32) -> (u32, u32) {
    let regs = unsafe { &*ptr };
    let original = regs.or().read().bits();
    unsafe {
        regs.or().write_with_zero(|w| w.bits(0x5a5));
    }
    let readback = regs.or().read().bits();
    unsafe {
        regs.or().write_with_zero(|w| w.bits(original));
    }
    (u32::from(readback == expected), readback)
}

fn crc_stable(crc: &mut Crc<'_>) -> (u32, u32, u32) {
    let bytes = [0x31u8, 0x32, 0x33, 0x34];
    let first = crc.feed_u8(&bytes);
    crc.reset();
    let second = crc.feed_u8(&bytes);
    crc.reset();
    let words = [0x3433_3231u32];
    let word = crc.feed_u32(&words);
    (first, second, word)
}

#[entry]
fn main() -> ! {
    write_word(0, MAGIC);

    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();

    test!(0, 1, {
        let addr = word(0) as u32;
        write_word(7, 0xa55a_1234);
        let back = read_word(7);
        let status = if back == 0xa55a_1234 && addr != 0 { PASS } else { FAIL };
        (status, addr, back)
    });

    let pa4 = p.PA4;
    test!(1, 2, {
        let mut pin = Flex::new(pa4);
        pin.set_as_input();
        pin.set_pull(Pull::Up);
        settle_gpio();
        let up = u32::from(pin.is_high());
        pin.set_pull(Pull::Down);
        settle_gpio();
        let down = u32::from(pin.is_high());
        pin.set_pull(Pull::None);
        (if up == 1 && down == 0 { PASS } else { FAIL }, up, down)
    });

    let pa8 = p.PA8;
    test!(2, 3, {
        let mut out = Output::new(pa8, Level::Low);
        out.set_high();
        let high = u32::from(out.is_set_high());
        out.set_low();
        let low = u32::from(out.is_set_low());
        (if high == 1 && low == 1 { PASS } else { FAIL }, high, low)
    });

    let mut t0 = Timer::new_blocking(p.TIMER0, TimerConfig::new()).unwrap();

    test!(3, 4, {
        let (ok, readback) = or_probe(pac::Gptim0::PTR.cast(), 0x5a5);
        (if ok == 1 { PASS } else { FAIL }, 0, readback)
    });

    {
        let _t1 = Timer::new_blocking(p.TIMER1, TimerConfig::new()).unwrap();
        test!(4, 5, {
            let (ok, readback) = or_probe(pac::Gptim1::PTR.cast(), 0x001);
            (if ok == 1 { PASS } else { FAIL }, 0, readback)
        });
    }

    {
        let _t2 = Timer::new_blocking(p.TIMER2, TimerConfig::new()).unwrap();
        test!(5, 6, {
            let (ok, readback) = or_probe(pac::Gptim2::PTR.cast(), 0x1a5);
            (if ok == 1 { PASS } else { FAIL }, 0, readback)
        });
    }

    {
        let _t3 = Timer::new_blocking(p.TIMER3, TimerConfig::new()).unwrap();
        test!(6, 7, {
            let (ok, readback) = or_probe(pac::Gptim3::PTR.cast(), 0x025);
            (if ok == 1 { PASS } else { FAIL }, 0, readback)
        });
    }

    test!(7, 8, {
        t0.set_period(100);
        t0.set_enabled(true);
        let mut set = false;
        for _ in 0..2_000_000 {
            if t0.interrupt_status().contains(TimerFlags::UPDATE) {
                set = true;
                break;
            }
        }
        t0.set_enabled(false);
        let cleared = if set {
            t0.clear_interrupt(TimerFlags::UPDATE);
            !t0.interrupt_status().contains(TimerFlags::UPDATE)
        } else {
            false
        };
        (
            if set && cleared { PASS } else { FAIL },
            u32::from(set),
            u32::from(cleared),
        )
    });

    drop(t0);

    if let Ok(mut adc) = Adc::new_blocking(p.ADC, AdcConfig::default()) {
        let channel = AdcChannel::new(p.PA5);

        test!(8, 9, {
            match adc.blocking_read_channel(&channel) {
                Ok(value) => (PASS, u32::from(value), 0),
                Err(_) => (FAIL, 0, 0),
            }
        });

        test!(9, 10, {
            let regs = unsafe { pac::Adc::steal() };
            let started = adc.start_continuous_channels(&[SampleChannel::In2]).is_ok();
            let mut set = false;
            if started {
                for _ in 0..2_000_000 {
                    if regs.isr().read().eoc().bit_is_set() {
                        set = true;
                        break;
                    }
                }
            }
            if set {
                regs.cr().modify(|_, w| w.stop().set_bit());
                for _ in 0..1_000_000 {
                    if regs.cr().read().start().bit_is_clear() {
                        break;
                    }
                }
                regs.cr().modify(|_, w| w.dis().set_bit());
            }
            let before = u32::from(regs.isr().read().eoc().bit_is_set());
            let after = if before == 1 {
                unsafe {
                    regs.isr().write_with_zero(|w| w.eoc().clear_bit_by_one());
                }
                u32::from(regs.isr().read().eoc().bit_is_set())
            } else {
                1
            };
            adc.stop_conversion();
            (
                if started && set && before == 1 && after == 0 {
                    PASS
                } else {
                    FAIL
                },
                u32::from(started) | u32::from(set) << 1,
                before | after << 1,
            )
        });

        test!(10, 11, {
            adc.set_vbat31_enabled(true);
            let on = u32::from(
                unsafe { pac::Afec::steal() }
                    .analog_2c()
                    .read()
                    .vbat_measure_enable()
                    .bit_is_set(),
            );
            adc.set_vbat31_enabled(false);
            let off = u32::from(
                unsafe { pac::Afec::steal() }
                    .analog_2c()
                    .read()
                    .vbat_measure_enable()
                    .bit_is_set(),
            );
            let iref = u32::from(
                unsafe { pac::Afec::steal() }
                    .analog_12()
                    .read()
                    .adc_internal_ref()
                    .bit_is_set(),
            );
            (
                if on == 1 && off == 0 && iref == 1 { PASS } else { FAIL },
                on | off << 1,
                iref,
            )
        });
    } else {
        test!(8, 9, { (FAIL, 0, 0) });
        test!(9, 10, { (SKIP, 0, 0) });
        test!(10, 11, { (SKIP, 0, 0) });
    }

    test!(11, 12, {
        let mut crc = Crc::new(p.CRC, CrcConfig::default());
        let (first, second, word) = crc_stable(&mut crc);
        crc.reset();
        let (first2, second2, _) = crc_stable(&mut crc);
        let status = if first == second && first == first2 && second == second2 {
            PASS
        } else {
            FAIL
        };
        (status, first ^ word, second ^ word)
    });

    let mut dma = Channel::new_blocking(p.DMA0_CH0);
    test!(12, 13, {
        let source = [0x1122_3344u32, 0x5566_7788, 0x99aa_bbcc, 0xddee_ff00];
        let mut destination = [0u32; 4];
        let status = match dma.blocking_copy(&source, &mut destination, TransferOptions::default()) {
            Ok(()) if destination == source => PASS,
            Ok(()) => FAIL,
            Err(_) => FAIL,
        };
        (status, destination[0], destination[3])
    });

    let mut dac = Dac::new(p.DAC, p.PA9, DacConfig::default());
    test!(13, 14, {
        let afec = unsafe { pac::Afec::steal() };
        let on = u32::from(afec.analog_11().read().dac_power_down().is_enabled())
            | u32::from(afec.analog_12().read().dac_power_down().bit_is_clear()) << 1
            | u32::from(afec.analog_27().read().dac_output_enable().is_enabled()) << 2;
        let set_ok = u32::from(dac.set(2048).is_ok());
        dac.disable();
        let off = u32::from(afec.analog_11().read().dac_power_down().is_powered_down())
            | u32::from(afec.analog_12().read().dac_power_down().bit_is_set()) << 1
            | u32::from(afec.analog_27().read().dac_output_enable().is_disabled()) << 2;
        (
            if on == 7 && set_ok == 1 && off == 7 { PASS } else { FAIL },
            on | off << 3,
            set_ok,
        )
    });

    test!(14, 15, {
        let rcc = unsafe { pac::Rcc::steal() };
        let before = rcc.rst_sr().read();
        let cpu = u32::from(before.cpu_reset_sr().bit_is_set());
        let cleared = if cpu == 1 {
            unsafe {
                rcc.rst_sr().write_with_zero(|w| w.cpu_reset_sr().clear_bit_by_one());
            }
            u32::from(rcc.rst_sr().read().cpu_reset_sr().bit_is_clear())
        } else {
            0
        };
        (
            if cpu == 1 && cleared == 1 {
                PASS
            } else if cpu == 0 {
                SKIP
            } else {
                FAIL
            },
            cpu,
            cleared,
        )
    });

    let _syscfg = Syscfg::new(p.SYSCFG);
    test!(15, 16, {
        let syscfg = unsafe { pac::Syscfg::steal() };
        let original = syscfg.cr5().read().bits();
        unsafe { syscfg.cr5().write_with_zero(|w| w.bits(0x5a5a_5a5a)) };
        let readback = syscfg.cr5().read().bits();
        unsafe { syscfg.cr5().write_with_zero(|w| w.bits(original)) };
        (if readback == 0x5a5a_5a5a { PASS } else { FAIL }, original, readback)
    });

    test!(16, 17, {
        let rcc = unsafe { pac::Rcc::steal() };
        rcc.cgr1()
            .modify(|_, w| w.wwdg_clk_en().set_bit().wwdg_cnt_clk_en().set_bit());
        rcc.rst0().modify(|_, w| w.wwdg_rst_n().clear_bit());
        settle_gpio();
        rcc.rst0().modify(|_, w| w.wwdg_rst_n().set_bit());
        settle_gpio();

        let wdg = unsafe { pac::Wwdg::steal() };
        unsafe {
            wdg.lock().write_with_zero(|w| w.bits(0x1acc_e551));
            wdg.load().write_with_zero(|w| w.bits(0x1000));
        }
        wdg.control().modify(|_, w| {
            w.inten().set_bit();
            w.resen().clear_bit();
            w
        });
        unsafe {
            wdg.lock().write_with_zero(|w| w.bits(!0x1acc_e551));
        }

        let mut ris = 0;
        let mut mis = 0;
        for _ in 0..2_000_000 {
            ris = wdg.ris().read().bits();
            mis = wdg.mis().read().bits();
            if ris & 1 != 0 || mis & 1 != 0 {
                break;
            }
        }
        unsafe {
            wdg.lock().write_with_zero(|w| w.bits(0x1acc_e551));
            wdg.intclr().write_with_zero(|w| w.bits(1));
            wdg.control().modify(|_, w| w.inten().clear_bit());
            wdg.load().write_with_zero(|w| w.bits(0xffff_ffff));
            wdg.lock().write_with_zero(|w| w.bits(!0x1acc_e551));
        }
        let cleared = wdg.mis().read().bits() & 1;

        (
            if mis & 1 == 1 && cleared == 0 { PASS } else { FAIL },
            ris | mis << 1,
            cleared,
        )
    });

    test!(17, 18, {
        let rcc = unsafe { pac::Rcc::steal() };
        rcc.cgr1().modify(|_, w| w.iwdg_clk_en().set_bit());
        rcc.rst0().modify(|_, w| w.iwdg_rst_n().clear_bit());
        settle_gpio();
        rcc.rst0().modify(|_, w| w.iwdg_rst_n().set_bit());
        settle_gpio();

        let iwdg = unsafe { pac::Iwdg::steal() };
        let mut ready = false;
        for _ in 0..2_000_000 {
            if iwdg.sr().read().bits() & 0xf == 0xf {
                ready = true;
                break;
            }
        }
        let original_max = iwdg.max().read().bits();
        let original_win = iwdg.win().read().bits();
        if ready {
            unsafe {
                iwdg.max().write_with_zero(|w| w.bits(0xabc));
            }
            for _ in 0..2_000_000 {
                if iwdg.sr().read().bits() & 0xf == 0xf {
                    break;
                }
            }
            unsafe {
                iwdg.win().write_with_zero(|w| w.bits(0x123));
            }
            for _ in 0..2_000_000 {
                if iwdg.sr().read().bits() & 0xf == 0xf {
                    break;
                }
            }
        }
        let max = iwdg.max().read().bits();
        let win = iwdg.win().read().bits();
        unsafe {
            iwdg.max().write_with_zero(|w| w.bits(original_max));
            iwdg.win().write_with_zero(|w| w.bits(original_win));
        }
        (
            if ready && max == 0xabc && win == 0x123 {
                PASS
            } else {
                FAIL
            },
            max,
            win,
        )
    });

    // Re-enable the DAC and drive a known voltage for an optional meter check.
    dac.enable();
    let _ = dac.set(2048);

    write_word(2, DONE);
    loop {
        cortex_m::asm::wfi();
    }
}
