//! The `v1` ADC (legacy): `SR`/`CR1`/`CR2`, a 16 (or 28) entry regular sequence, a 4-entry injected
//! sequence, per-channel sample times, a single analog watchdog and no calibration registers
//! beyond the F1 self-calibration.
//!
//! Chips: F1/F37x (`adc_v1_f1`), F2/F4/F7 (`adc_v1_f4`), L1 (`adc_v1_l1`).

use core::sync::atomic::Ordering;

#[cfg(not(adc_v1_f1))]
use super::Prescaler;
use super::injected::InjectedRegs;
use super::{
    AdcRegs, Clock, Config, ConversionMode, Exten, InternalChannel, Resolution, SampleTimes, State, WatchdogChannels,
};
use crate::pac::adc::Adc as Regs;
#[cfg(adc_v1_l1)]
use crate::pac::adc::vals::Adcpre;
#[cfg(not(adc_v1_f1))]
use crate::pac::adc::vals::Res;
use crate::pac::adc::vals::SampleTime;
#[cfg(adc_v1_f4)]
use crate::pac::adccommon::AdcCommon;
#[cfg(adc_v1_f4)]
use crate::pac::adccommon::vals::Adcpre;
use crate::time::Hertz;
use crate::wait::block_for_us;

/// Maximum ADC clock frequency.
#[cfg(adc_v1_f1)]
const MAX_CLOCK: Hertz = Hertz::mhz(14);
#[cfg(stm32f2)]
const MAX_CLOCK: Hertz = Hertz::mhz(30);
#[cfg(any(stm32f4, stm32f7))]
const MAX_CLOCK: Hertz = Hertz::mhz(36);
#[cfg(adc_v1_l1)]
const MAX_CLOCK: Hertz = Hertz::mhz(16);

/// Sample times as half ADC clock cycles, indexed by the `SMP` field value.
#[cfg(adc_v1_f1)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 15, 27, 57, 83, 111, 143, 479];
#[cfg(adc_v1_f4)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [6, 30, 56, 112, 168, 224, 288, 960];
#[cfg(adc_v1_l1)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [8, 18, 32, 48, 96, 192, 384, 768];

/// Number of channels.
#[cfg(adc_v1_f1)]
const CHANNELS: usize = 18;
#[cfg(adc_v1_f4)]
const CHANNELS: usize = 19;
#[cfg(adc_v1_l1)]
const CHANNELS: usize = 32;

/// `EXTSEL` value for a software start on the F1.
#[cfg(adc_v1_f1)]
const EXTSEL_SWSTART: u8 = 0b111;

#[cfg(adc_v1_f1)]
type Common = ();
#[cfg(adc_v1_f4)]
type Common = AdcCommon;
#[cfg(adc_v1_l1)]
type Common = ();

#[cfg(not(adc_v1_f1))]
fn to_res(res: Resolution) -> Res {
    match res {
        Resolution::Bits12 => Res::Bits12,
        Resolution::Bits10 => Res::Bits10,
        Resolution::Bits8 => Res::Bits8,
        Resolution::Bits6 => Res::Bits6,
    }
}

#[cfg(not(adc_v1_f1))]
fn from_res(res: Res) -> Resolution {
    match res {
        Res::Bits12 => Resolution::Bits12,
        Res::Bits10 => Resolution::Bits10,
        Res::Bits8 => Resolution::Bits8,
        Res::Bits6 => Resolution::Bits6,
    }
}

/// The prescaler values of the `ADCPRE` field, in order.
#[cfg(adc_v1_f4)]
const PRESCALERS: [Prescaler; 4] = [Prescaler::Div2, Prescaler::Div4, Prescaler::Div6, Prescaler::Div8];
#[cfg(adc_v1_l1)]
const PRESCALERS: [Prescaler; 3] = [Prescaler::Div1, Prescaler::Div2, Prescaler::Div4];

#[cfg(adc_v1_f4)]
fn to_presc(presc: Prescaler) -> Adcpre {
    match presc {
        Prescaler::Div2 => Adcpre::Div2,
        Prescaler::Div4 => Adcpre::Div4,
        Prescaler::Div6 => Adcpre::Div6,
        Prescaler::Div8 => Adcpre::Div8,
    }
}

#[cfg(adc_v1_l1)]
fn to_presc(presc: Prescaler) -> Adcpre {
    match presc {
        Prescaler::Div1 => Adcpre::Div1,
        Prescaler::Div2 => Adcpre::Div2,
        Prescaler::Div4 => Adcpre::Div4,
    }
}

/// The clock the ADC prescaler divides. On the L1 the ADC is always clocked from the HSI.
fn kernel_clock(kernel_clock: Hertz) -> Hertz {
    #[cfg(adc_v1_l1)]
    {
        let _ = kernel_clock;
        crate::rcc::HSI_FREQ
    }
    #[cfg(not(adc_v1_l1))]
    kernel_clock
}

#[cfg(not(adc_v1_f1))]
#[allow(unused_variables)]
fn set_presc(regs: Regs, common: Common, presc: Adcpre) {
    #[cfg(adc_v1_f4)]
    critical_section::with(|_| common.ccr().modify(|w| w.set_adcpre(presc)));
    #[cfg(adc_v1_l1)]
    regs.ccr().modify(|w| w.set_adcpre(presc));
}

impl super::BasicAdcRegs for Regs {
    type SampleTime = SampleTime;
}

impl AdcRegs for Regs {
    type Common = Common;

    const AWD_COUNT: usize = 1;
    #[cfg(adc_v1_l1)]
    const MAX_SEQUENCE_LEN: usize = 28;
    #[cfg(not(adc_v1_l1))]
    const MAX_SEQUENCE_LEN: usize = 16;
    const INJECTED_RANKS: usize = 4;

    fn init(self, common: Common, kernel_clock: Hertz, config: &Config) {
        // Clock. The F1 has no prescaler of its own: the RCC (`config.rcc.adc_pre`) sets it.
        match config.clock {
            Clock::Auto => {
                #[cfg(not(adc_v1_f1))]
                {
                    let kernel_clock = self::kernel_clock(kernel_clock);
                    let presc = PRESCALERS
                        .iter()
                        .copied()
                        .find(|p| kernel_clock / p.divisor() <= MAX_CLOCK)
                        .expect("ADC kernel clock too fast, use a slower APB2 clock");
                    set_presc(self, common, to_presc(presc));
                }
            }
            #[cfg(not(adc_v1_f1))]
            Clock::Async(presc) => set_presc(self, common, to_presc(presc)),
        }
        let clock = self.clock(common, kernel_clock);
        assert!(
            clock <= MAX_CLOCK,
            "ADC clock {} exceeds the maximum of {}",
            clock,
            MAX_CLOCK
        );

        self.enable();

        // F1: self-calibration, run once after power-up.
        #[cfg(adc_v1_f1)]
        {
            self.cr2().modify(|w| w.set_rstcal(true));
            while self.cr2().read().rstcal() {}
            self.cr2().modify(|w| w.set_cal(true));
            while self.cr2().read().cal() {}
        }

        // Single conversion mode, software trigger, scan mode (so multi-channel sequences work).
        self.cr1().modify(|w| {
            w.set_scan(true);
            w.set_discen(false);
        });
        self.cr2().modify(|w| {
            w.set_cont(false);
            #[cfg(adc_v1_f1)]
            {
                w.set_extsel(EXTSEL_SWSTART);
                w.set_exttrig(true);
            }
            #[cfg(not(adc_v1_f1))]
            w.set_exten(Exten::Disabled);
        });

        #[cfg(not(adc_v1_f1))]
        if let Some(res) = config.resolution {
            self.set_resolution(res);
        }
    }

    #[cfg_attr(not(adc_v1_f4), allow(unused_variables))]
    fn clock(self, common: Common, kernel_clock: Hertz) -> Hertz {
        let kernel_clock = self::kernel_clock(kernel_clock);
        #[cfg(adc_v1_f4)]
        {
            let _ = self;
            kernel_clock / PRESCALERS[common.ccr().read().adcpre().to_bits() as usize].divisor()
        }
        #[cfg(adc_v1_l1)]
        {
            kernel_clock / PRESCALERS[self.ccr().read().adcpre().to_bits() as usize].divisor()
        }
        #[cfg(adc_v1_f1)]
        {
            // The RCC driver already applies the ADC prescaler.
            let _ = (self, common);
            kernel_clock
        }
    }

    fn power_down(self) {
        self.cr2().modify(|w| w.set_adon(false));
        #[cfg(adc_v1_l1)]
        while self.sr().read().adons() {}
    }

    fn enable(self) {
        if !self.cr2().read().adon() {
            self.cr2().modify(|w| w.set_adon(true));
            // Stabilization time.
            block_for_us(3);
            #[cfg(adc_v1_l1)]
            while !self.sr().read().adons() {}
        }
    }

    #[cfg(not(adc_v1_f1))]
    fn set_resolution(self, res: Resolution) {
        self.cr1().modify(|w| w.set_res(to_res(res)));
    }

    fn resolution(self) -> Resolution {
        #[cfg(adc_v1_f1)]
        return Resolution::Bits12;
        #[cfg(not(adc_v1_f1))]
        from_res(self.cr1().read().res())
    }

    fn configure_sequence(self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>, injected: bool) {
        let len = sequence.len();
        assert!(len != 0, "sequence cannot be empty");
        if injected {
            assert!(len <= Self::INJECTED_RANKS, "injected sequence too long");
        } else {
            assert!(len <= Self::MAX_SEQUENCE_LEN, "sequence too long");
        }

        const SMPR_COUNT: usize = CHANNELS.div_ceil(10);
        let mut smpr = [crate::pac::adc::regs::Smpr::default(); SMPR_COUNT];
        for (i, s) in smpr.iter_mut().enumerate() {
            *s = self.smpr(i).read();
        }
        let mut sqr1 = crate::pac::adc::regs::Sqr1::default();
        let mut sqr2 = crate::pac::adc::regs::Sqr2::default();
        let mut sqr3 = crate::pac::adc::regs::Sqr3::default();
        #[cfg(adc_v1_l1)]
        let mut sqr4 = crate::pac::adc::regs::Sqr4::default();
        #[cfg(adc_v1_l1)]
        let mut sqr5 = crate::pac::adc::regs::Sqr5::default();
        let mut jsqr = self.jsqr().read();

        if injected {
            jsqr.set_jl(len as u8 - 1);
        } else {
            sqr1.set_l(len as u8 - 1);
        }

        for (i, ((channel, differential), sample_time)) in sequence.enumerate() {
            let channel = channel as usize;
            assert!(channel < CHANNELS, "channel {} does not exist", channel);
            assert!(!differential, "this ADC has no differential inputs");
            smpr[channel / 10].set_smp(channel % 10, sample_time);

            if injected {
                // The injected sequence is right-aligned: with JL + 1 conversions the ranks used
                // are JSQ[4 - (JL + 1)] to JSQ[3].
                jsqr.set_jsq(4 - len + i, channel as u8);
            } else {
                #[cfg(not(adc_v1_l1))]
                match i {
                    0..=5 => sqr3.set_sq(i, channel as u8),
                    6..=11 => sqr2.set_sq(i - 6, channel as u8),
                    _ => sqr1.set_sq(i - 12, channel as u8),
                }
                #[cfg(adc_v1_l1)]
                match i {
                    0..=5 => sqr5.set_sq(i, channel as u8),
                    6..=11 => sqr4.set_sq(i - 6, channel as u8),
                    12..=17 => sqr3.set_sq(i - 12, channel as u8),
                    18..=23 => sqr2.set_sq(i - 18, channel as u8),
                    _ => sqr1.set_sq(i - 24, channel as u8),
                }
            }
        }

        for (i, s) in smpr.iter().enumerate() {
            self.smpr(i).write_value(*s);
        }
        if injected {
            self.jsqr().write_value(jsqr);
        } else {
            self.sqr1().write_value(sqr1);
            self.sqr2().write_value(sqr2);
            self.sqr3().write_value(sqr3);
            #[cfg(adc_v1_l1)]
            {
                self.sqr4().write_value(sqr4);
                self.sqr5().write_value(sqr5);
            }
        }
    }

    fn configure_dma(self, mode: ConversionMode) {
        clear_flags(self);
        self.cr1().modify(|w| {
            w.set_scan(true);
            w.set_discen(false);
        });
        self.cr2().modify(|w| {
            let dma = !matches!(mode, ConversionMode::NoDma);
            w.set_dma(dma);
            #[cfg(not(adc_v1_f1))]
            {
                // Keep issuing DMA requests after a DMA transfer ends (otherwise the DMA bit has
                // to be toggled before every transfer), and flag overruns per conversion.
                w.set_dds(dma);
                w.set_eocs(dma);
            }
            w.set_cont(matches!(mode, ConversionMode::Repeated(None)));
            match mode {
                ConversionMode::Repeated(Some((trigger, edge))) => {
                    #[cfg(adc_v1_f1)]
                    {
                        let _ = edge;
                        w.set_extsel(trigger);
                        w.set_exttrig(true);
                    }
                    #[cfg(not(adc_v1_f1))]
                    {
                        w.set_extsel(trigger);
                        w.set_exten(edge);
                    }
                }
                _ => {
                    #[cfg(adc_v1_f1)]
                    {
                        w.set_extsel(EXTSEL_SWSTART);
                        w.set_exttrig(true);
                    }
                    #[cfg(not(adc_v1_f1))]
                    w.set_exten(Exten::Disabled);
                }
            }
        });
    }

    fn start(self) {
        clear_flags(self);
        // With a hardware trigger selected, conversions start on the trigger; a software start
        // would add an extra, unsynchronized conversion.
        #[cfg(adc_v1_f1)]
        let software = self.cr2().read().extsel() == EXTSEL_SWSTART;
        #[cfg(not(adc_v1_f1))]
        let software = self.cr2().read().exten() == Exten::Disabled;
        if software {
            self.cr2().modify(|w| w.set_swstart(true));
        }
    }

    fn stop(self) {
        // There is no way to abort a conversion other than switching the converter off. Only do
        // it when conversions can still be pending: a single software-started sequence finishes
        // on its own.
        let cr2 = self.cr2().read();
        #[cfg(adc_v1_f1)]
        let triggered = cr2.extsel() != EXTSEL_SWSTART;
        #[cfg(not(adc_v1_f1))]
        let triggered = cr2.exten() != Exten::Disabled;
        if cr2.adon() && (cr2.cont() || cr2.dma() || triggered) {
            self.cr2().modify(|w| {
                w.set_adon(false);
                w.set_cont(false);
                w.set_dma(false);
            });
            #[cfg(adc_v1_l1)]
            while self.sr().read().adons() {}
        }
    }

    fn done(self) -> bool {
        self.sr().read().eoc()
    }

    fn data(self) -> *mut u16 {
        self.dr().as_ptr() as *mut u16
    }

    fn set_eoc_interrupt(self, enable: bool) {
        self.cr1().modify(|w| w.set_eocie(enable));
    }

    fn on_interrupt(self, state: &State) {
        let sr = self.sr().read();
        let cr1 = self.cr1().read();
        let mut wake = false;

        if cr1.eocie() && sr.eoc() {
            self.cr1().modify(|w| w.set_eocie(false));
            wake = true;
        }
        if cr1.jeocie() && sr.jeoc() {
            self.sr().modify(|w| w.set_jeoc(false));
            state.injected_done.store(true, Ordering::Release);
            wake = true;
        }
        if cr1.awdie() && sr.awd() {
            self.cr1().modify(|w| w.set_awdie(false));
            self.sr().modify(|w| w.set_awd(false));
            state.awd_triggered[0].store(true, Ordering::Release);
            wake = true;
        }
        if wake {
            state.waker.wake();
        }
    }

    #[cfg_attr(not(adc_v1_f4), allow(unused_variables))]
    fn enable_internal(self, common: Common, channel: InternalChannel, enable: bool) {
        match channel {
            InternalChannel::VrefInt | InternalChannel::Temperature => {
                #[cfg(adc_v1_f1)]
                self.cr2().modify(|w| w.set_tsvrefe(enable));
                #[cfg(adc_v1_f4)]
                {
                    let _ = self;
                    critical_section::with(|_| common.ccr().modify(|w| w.set_tsvrefe(enable)));
                }
                #[cfg(adc_v1_l1)]
                self.ccr().modify(|w| w.set_tsvrefe(enable));
            }
            #[cfg(adc_v1_f4)]
            InternalChannel::Vbat => critical_section::with(|_| common.ccr().modify(|w| w.set_vbaten(enable))),
            #[cfg(not(adc_v1_f4))]
            InternalChannel::Vbat => panic!("this ADC has no VBAT channel"),
            InternalChannel::VddCore => panic!("this ADC has no VDDCORE channel"),
            InternalChannel::Dac(_) => panic!("this ADC has no DAC channel"),
        }
        // Startup time of the internal reference and temperature sensor.
        if enable {
            block_for_us(10);
        }
    }

    fn configure_awd(self, index: usize, channels: WatchdogChannels, low: u32, high: u32) {
        assert!(index == 0, "this ADC has no watchdog {}", index + 1);
        self.cr1().modify(|w| {
            match channels {
                WatchdogChannels::All => w.set_awdsgl(false),
                WatchdogChannels::Single(ch) => {
                    w.set_awdsgl(true);
                    w.set_awdch(ch);
                }
                WatchdogChannels::Channels(_) => panic!("this watchdog monitors a single channel or all channels"),
            }
            w.set_awden(true);
            w.set_jawden(true);
        });
        self.ltr().write(|w| w.set_lt(low as u16));
        self.htr().write(|w| w.set_ht(high as u16));
    }

    fn disable_awd(self, _index: usize) {
        self.cr1().modify(|w| {
            w.set_awden(false);
            w.set_jawden(false);
        });
    }

    fn set_awd_interrupt(self, _index: usize, enable: bool) {
        if enable {
            self.sr().modify(|w| w.set_awd(false));
        }
        self.cr1().modify(|w| w.set_awdie(enable));
    }

    fn clear_awd_flag(self, _index: usize) -> bool {
        let set = self.sr().read().awd();
        if set {
            self.sr().modify(|w| w.set_awd(false));
        }
        set
    }

    fn set_continuous(self, enable: bool) {
        self.cr2().modify(|w| w.set_cont(enable));
    }
}

/// Clear the status flags (they are cleared by writing zero).
fn clear_flags(regs: Regs) {
    regs.sr().modify(|w| {
        w.set_eoc(false);
        w.set_strt(false);
        #[cfg(not(adc_v1_f1))]
        w.set_ovr(false);
    });
}

impl InjectedRegs for Regs {
    fn configure_injected_trigger(self, trigger: (u8, Exten), interrupt: bool) {
        self.cr1().modify(|w| {
            w.set_scan(true);
            w.set_jdiscen(false);
            w.set_jauto(false);
        });
        self.cr2().modify(|w| {
            #[cfg(adc_v1_f1)]
            {
                if trigger.1 == Exten::Disabled {
                    w.set_jextsel(EXTSEL_SWSTART);
                } else {
                    w.set_jextsel(trigger.0);
                }
                w.set_jexttrig(true);
            }
            #[cfg(not(adc_v1_f1))]
            {
                w.set_jextsel(trigger.0);
                w.set_jexten(trigger.1);
            }
        });
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
        self.cr1().modify(|w| w.set_jeocie(interrupt));
    }

    fn start_injected(self) {
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
        // A hardware-triggered injected sequence is armed by its trigger selection; a software
        // start is only valid for software-triggered sequences.
        #[cfg(adc_v1_f1)]
        let software = self.cr2().read().jextsel() == EXTSEL_SWSTART;
        #[cfg(not(adc_v1_f1))]
        let software = self.cr2().read().jexten() == Exten::Disabled;
        if software {
            self.cr2().modify(|w| w.set_jswstart(true));
        }
    }

    fn stop_injected(self) {
        // Injected conversions cannot be aborted; disarm the hardware trigger.
        self.cr2().modify(|w| {
            #[cfg(adc_v1_f1)]
            w.set_jexttrig(false);
            #[cfg(not(adc_v1_f1))]
            w.set_jexten(Exten::Disabled);
        });
        self.cr1().modify(|w| w.set_jeocie(false));
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
    }

    fn read_injected(self, data: &mut [u16]) {
        for (i, d) in data.iter_mut().enumerate() {
            *d = self.jdr(i).read().jdata();
        }
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
    }
}

impl SampleTimes for Regs {
    fn sample_time_for_half_cycles(half_cycles: u32) -> SampleTime {
        let i = SAMPLE_TIME_HALF_CYCLES
            .iter()
            .position(|c| *c >= half_cycles)
            .unwrap_or(SAMPLE_TIME_HALF_CYCLES.len() - 1);
        SampleTime::from_bits(i as u8)
    }
}
