//! Timer (TIM) drivers.

#![macro_use]

use embassy_hal_internal::{Peri, PeripheralType};
use mspm0_metapac::tim::Tim;

use crate::common::hillclimb;
use crate::gpio::Pin;
use crate::interrupt;

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: interrupt::typelevel::Interrupt;

    #[inline]
    fn width() -> CounterWidth {
        Self::info().width
    }
}

/// A timer instance with 2 compare and capture channels.
pub trait General2ChannelInstance: Instance {}

/// A timer instance with 4 compare and capture channels.
pub trait General4ChannelInstance: General2ChannelInstance {}

/// A timer instance with a 32-bit counter.
pub trait General32BitInstance: Instance {}

/// An advanced timer instance with complementary channel outputs and fault detection.
pub trait AdvancedInstance: Instance {}

/// Marker trait describing the type used for counting.
#[allow(private_bounds)]
pub trait TimerBits: SealedTimerBits {}
impl TimerBits for u16 {}
impl TimerBits for u32 {}

/// Width of counter in a timer instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CounterWidth {
    /// 16-bit counter width.
    Bits16,

    /// 32-bit counter width.
    Bits32,
}

impl CounterWidth {
    /// Counter width in bits.
    pub const fn bits(&self) -> u8 {
        match self {
            CounterWidth::Bits16 => 16,
            CounterWidth::Bits32 => 32,
        }
    }
}

/// A timer pin.
#[allow(private_bounds)]
pub trait TimerPin<T: Instance, Channel: TimerChannel>: Pin + PeripheralType {
    /// Get the PF number needed to use this pin as a timer pin for the specified [`Channel`].
    fn pf_num(&self) -> u8;
}

/// A timer channel
#[allow(private_bounds)]
pub trait TimerChannel: SealedChannel {}

/// Marker type for channel 0.
pub enum Ch0 {}
impl TimerChannel for Ch0 {}

/// Marker type for channel 1.
pub enum Ch1 {}
impl TimerChannel for Ch1 {}

/// Marker type for channel 2.
pub enum Ch2 {}
impl TimerChannel for Ch2 {}

/// Marker type for channel 3.
pub enum Ch3 {}
impl TimerChannel for Ch3 {}

/// Marker type for channel 0 complementary output.
pub enum CompCh0 {}
impl TimerChannel for CompCh0 {}

/// Marker type for channel 1 complementary output.
pub enum CompCh1 {}
impl TimerChannel for CompCh1 {}

/// Marker type for channel 2 complementary output.
pub enum CompCh2 {}
impl TimerChannel for CompCh2 {}

/// Marker type for channel 3 complementary output.
pub enum CompCh3 {}
impl TimerChannel for CompCh3 {}

/// Clock source for the timer.
///
/// The actual clock speed used depends on power domain and system clock config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSel {
    /// Use the low frequency clock.
    ///
    /// The LFCLK runs at 32.768 kHz.
    LfClk,

    /// Use the middle frequency clock.
    ///
    /// The MFCLK runs at 4 MHz.
    MfClk,

    /// Use the common Bus frequency (post PLL)
    ///
    /// Bus Clock; typically 24-80MHz
    BusClk,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CountingMode {
    /// The timer counts up to the reload value and then resets back at 0.
    #[default]
    EdgeAlignedUp,

    /// The timer counts down to 0 and then resets back to the load value.
    EdgeAlignedDown,

    /// The timer counts up to the load value and then counts back to 0.
    CenterAligned,
}

/// Basic low-lever interface into a timer
pub struct LLTimer<'d, T: Instance> {
    #[allow(unused)]
    inner: Peri<'d, T>,
}

impl<'d, T: Instance> LLTimer<'d, T> {
    pub fn new(tim: Peri<'d, T>) -> Self {
        Self { inner: tim }
    }

    pub fn set_clk_source(&self, src: ClockSel) {
        let regs = T::info().regs;
        regs.clksel().write_value(mspm0_metapac::tim::regs::Clksel(match src {
            ClockSel::BusClk => 0b1000,
            ClockSel::MfClk => 0b0100,
            ClockSel::LfClk => 0b0010,
        }));
    }

    pub fn set_clk_enable(&self, enable: bool) {
        let regs = T::info().regs;
        regs.commonregs(0).cclkctl().write(|w| {
            w.set_clken(enable);
        })
    }

    /// Automatically configures clock frequency & tmr.load to give a periodic interrupt
    /// Frequency is at best-effort basis, actual frequency is returned
    ///
    /// This uses the LoadEvent, which should be cleared in the interrupt handler
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// #[interrupt]
    /// fn TIMA0() {
    ///   pac::TIMA0.cpu_int(0).iclr().write(|w| {
    ///     w.set_l(true);
    ///   });
    ///
    ///   ...
    /// }
    ///
    /// let tima0 = LLTimer::new(p.TIMA0);
    /// let actual_freq = unsafe { tima0.start_periodic_timer(10) };
    /// ```
    ///
    /// SAFETY: this requires the user to setup an interrupt handler which clears the zero interrupt
    ///  failure to do so will block the CPU
    pub unsafe fn start_periodic_timer(&self, freq: u32) -> u32 {
        let regs = T::info().regs;

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
        let regs = T::info().regs;
        regs.counterregs(0).ctrctl().write(|w| {
            w.set_en(false);
        })
    }

    // Set clock rate (*not interrupt-rate*) at best-effort
    //
    // WARN: currently assumes BusClock with MCLK source
    pub fn set_clk_freq(&self, frequency: u32) -> u32 {
        let info = T::info();
        let regs = info.regs;
        // Frequency is chip-specific & based on power-domain;

        // FIXME: usually BusClock is MCLK, but e.g. TIMG0 on G310x is PD0->ULPCLK, currently there is no way to distinguish
        let clk_freq = crate::clocks::CLOCKS.m_clk.load(core::sync::atomic::Ordering::Relaxed);

        // TODO: use mathacl for div?
        // NOTE: could also use `FEATUREVER` to find the available features
        if info.prescaler {
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

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    unsafe fn enable_interrupt();
}

trait SealedTimerBits: Into<u32> {}
impl SealedTimerBits for u16 {}
impl SealedTimerBits for u32 {}

trait SealedChannel {}
impl SealedChannel for Ch0 {}
impl SealedChannel for Ch1 {}
impl SealedChannel for Ch2 {}
impl SealedChannel for Ch3 {}
impl SealedChannel for CompCh0 {}
impl SealedChannel for CompCh1 {}
impl SealedChannel for CompCh2 {}
impl SealedChannel for CompCh3 {}

pub(crate) struct Info {
    pub(crate) regs: Tim,
    #[allow(unused)]
    pub(crate) prescaler: bool,
    pub(crate) width: CounterWidth,
    #[allow(unused)]
    pub(crate) channels: u8,
}

macro_rules! impl_tim_instance {
    (
        $instance: ident,
        prescaler: $prescaler: expr,
        width: $width: ident,
        channels: $channels: expr
    ) => {
        impl crate::tim::SealedInstance for crate::peripherals::$instance {
            #[inline]
            fn info() -> &'static crate::tim::Info {
                const INFO: crate::tim::Info = crate::tim::Info {
                    regs: crate::pac::$instance,
                    prescaler: $prescaler,
                    width: crate::tim::CounterWidth::$width,
                    channels: $channels,
                };

                &INFO
            }

            unsafe fn enable_interrupt() {
                use embassy_hal_internal::interrupt::InterruptExt;
                crate::interrupt::$instance.unpend();
                crate::interrupt::$instance.enable();
            }
        }

        impl crate::tim::Instance for crate::peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$instance;
        }
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_32bit {
    ($instance: ident) => {
        impl crate::tim::General32BitInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_2ch {
    ($instance: ident) => {
        impl crate::tim::General2ChannelInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_general_4ch {
    ($instance: ident) => {
        impl crate::tim::General4ChannelInstance for crate::peripherals::$instance {}
    };
}

#[allow(unused)]
macro_rules! impl_tim_instance_advanced {
    ($instance: ident) => {
        impl crate::tim::AdvancedInstance for crate::peripherals::$instance {}
    };
}

macro_rules! impl_tim_pin {
    (
        $instance: ident,
        $pin: ident,
        $pf: expr,
        $channel: ident
    ) => {
        impl crate::tim::TimerPin<crate::peripherals::$instance, crate::tim::$channel> for crate::peripherals::$pin {
            #[inline]
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}
