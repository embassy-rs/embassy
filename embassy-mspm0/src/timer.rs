#![macro_use]

use embassy_hal_internal::{Peri, PeripheralType};

use crate::common::hillclimb;

/// Amount of bits of a timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerBits {
    /// 16 bits.
    Bits16,
    /// 32 bits.
    Bits32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerPrescaler {
    /// 8 bits.
    Bits8,
    // No prescaler
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerClockSource {
    /// Bus Clock; typically 24-80MHz
    BusClock,
    // Middle Frequency; Fixed 4MHz clock
    MFClock,
    // Low Frequency; 32kHz
    LFClock,
}

#[allow(private_bounds)]
pub trait Timer: PeripheralType + SealedTimer + 'static {
    /// Amount of bits this timer has.
    const BITS: TimerBits;

    /// Availability & Size of prescaler
    const PRESCALER: TimerPrescaler;
}

pub(crate) trait SealedTimer {
    /// Registers for this timer.
    ///
    /// This is a raw pointer to the register block. The actual available registers will depends on the timer type.
    fn regs() -> crate::pac::tim::Tim;

    /// Enable the interrupt corresponding to this timer.
    unsafe fn enable_interrupt();
}

/// Basic low-lever interface into a timer
pub struct LLTimer<'d, T: Timer> {
    #[allow(unused)]
    inner: Peri<'d, T>,
}

impl<'d, T: Timer> LLTimer<'d, T> {
    pub fn new(tim: Peri<'d, T>) -> Self {
        Self { inner: tim }
    }

    pub fn set_clk_source(&self, src: TimerClockSource) {
        let regs = T::regs();
        regs.clksel().write_value(mspm0_metapac::tim::regs::Clksel(match src {
            TimerClockSource::BusClock => 0b1000,
            TimerClockSource::MFClock => 0b0100,
            TimerClockSource::LFClock => 0b0010,
        }));
    }

    pub fn set_clk_enable(&self, enable: bool) {
        let regs = T::regs();
        regs.commonregs(0).cclkctl().write(|w| {
            w.set_clken(enable);
        })
    }

    // Automatically configures clock frequency & tmr.load to give a periodic interrupt
    // Frequency is at best-effort basis, actual frequency is returned
    //
    // This uses the ZeroEvent
    //
    // SAFETY: this requires the user to setup an interrupt handler which clears the zero interrupt
    //  failure to do so will block the CPU
    pub unsafe fn start_periodic_timer(&self, freq: u32) -> u32 {
        let regs = T::regs();

        // Reset timer regs
        regs.gprcm(0).rstctl().write(|w| {
            w.set_resetassert(true);
            w.set_key(mspm0_metapac::tim::vals::ResetKey::KEY);
            w.set_resetstkyclr(true);
        });
        regs.gprcm(0).pwren().modify(|w| {
            w.set_enable(true);
            w.set_key(mspm0_metapac::tim::vals::PwrenKey::KEY);
        });
        // FIXME: assuming busclk for now
        regs.clksel().modify(|w| w.set_busclk_sel(true));

        let ctr = regs.counterregs(0);

        // Frequency guess
        let actual_freq = self.set_clk_freq(freq * 100);
        let load = actual_freq.div_ceil(freq).min(u16::MAX as u32);

        #[cfg(feature = "defmt")]
        defmt::trace!("Timer LOAD={}", load);

        ctr.load().write_value(load);
        ctr.ctr().write_value(load);
        ctr.ctrctl().write(|w| {
            w.set_cm(mspm0_metapac::tim::vals::Cm::DOWN);
            w.set_repeat(mspm0_metapac::tim::vals::Repeat::REPEAT_1);
            w.set_cvae(mspm0_metapac::tim::vals::Cvae::LDVAL);
            w.set_en(true);
        });

        regs.cpu_int(0).imask().write(|w| {
            w.set_l(true);
        });

        regs.commonregs(0).cclkctl().modify(|w| {
            w.set_clken(true);
        });

        unsafe { T::enable_interrupt() };

        actual_freq / load
    }

    pub fn stop_timer(&self) {
        let regs = T::regs();
        regs.counterregs(0).ctrctl().write(|w| {
            w.set_en(false);
        })
    }

    // Set clock rate (*not interrupt-rate*) at best-effort
    //
    // WARN: currently assumes BusClock with MCLK source
    pub fn set_clk_freq(&self, frequency: u32) -> u32 {
        let regs = T::regs();
        // Frequency is chip-specific & based on power-domain;

        // FIXME: usually BusClock is MCLK, but e.g. TIMG0 on G310x is PD0->ULPCLK, currently there is no way to distinguish
        let clk_freq = crate::clocks::CLOCKS.m_clk.load(core::sync::atomic::Ordering::Relaxed);

        // TODO: use mathacl for div?
        // NOTE: could also use `FEATUREVER` to find the available features
        if matches!(T::PRESCALER, TimerPrescaler::Bits8) {
            let div_range = 0..8u32;
            // Should be optimal value for this clock
            let divs = hillclimb([0u32; 2], |divs| {
                if !div_range.contains(&divs[0]) || !(0..0xff).contains(&divs[1]) {
                    i32::MAX
                } else {
                    clk_freq as i32 - (frequency * (divs[0] + 1) * (divs[1] + 1)) as i32
                }
            });
            regs.clkdiv().write_value(mspm0_metapac::tim::regs::Clkdiv(divs[0]));
            regs.commonregs(0)
                .cps()
                .write_value(mspm0_metapac::tim::regs::Cps(divs[1]));

            #[cfg(feature = "defmt")]
            defmt::trace!("clk={}, divs[0]={} divs[1]={}", clk_freq, divs[0], divs[1]);
            clk_freq / ((divs[0] + 1) * (divs[1] + 1))
        } else {
            let divider = (frequency / clk_freq).saturating_sub(1);
            let actual_div = divider.min(7);
            regs.clkdiv().write_value(mspm0_metapac::tim::regs::Clkdiv(actual_div));
            regs.commonregs(0).cps().write_value(mspm0_metapac::tim::regs::Cps(0));

            clk_freq / (actual_div + 1)
        }
    }
}

macro_rules! impl_timer {
    ($name: ident, $bits: ident, $prescaler: ident) => {
        impl crate::timer::SealedTimer for crate::peripherals::$name {
            fn regs() -> crate::pac::tim::Tim {
                unsafe { crate::pac::tim::Tim::from_ptr(crate::pac::$name.as_ptr()) }
            }

            unsafe fn enable_interrupt() {
                use embassy_hal_internal::interrupt::InterruptExt;
                crate::interrupt::$name.unpend();
                crate::interrupt::$name.enable();
            }
        }

        impl crate::timer::Timer for crate::peripherals::$name {
            const BITS: crate::timer::TimerBits = crate::timer::TimerBits::$bits;
            const PRESCALER: crate::timer::TimerPrescaler = crate::timer::TimerPrescaler::$prescaler;
        }
    };
}
