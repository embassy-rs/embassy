//! The 12-bit `ADC3` of the STM32H72x/H73x (RM0468 chapter 29, `adc_v3_h7_adc3`).
//!
//! A `v3` ADC with the `TR1..3`/`DMAEN` layout of the L4/G4 generation, next to the 16-bit
//! `ADC1`/`ADC2` handled by [`super::v3`]. No voltage regulator ready flag, no `BOOST`, no channel
//! preselection, no linearity calibration, 19 channels. The common registers live in the
//! `ADC3_COMMON` block.

use core::sync::atomic::Ordering;

use super::injected::InjectedRegs;
use super::{
    AdcRegs, Clock, Config, ConversionMode, InternalChannel, Oversampling, Prescaler, Resolution, SampleTimes, State,
    SyncDiv, WatchdogChannels,
};
use crate::pac::adc::Adc3 as Regs;
use crate::pac::adc::vals::{Adc3Dmacfg, Adc3Ovsr, Adc3Res, Adc3SampleTime, Exten, Rovsm, Trovs};
use crate::pac::adccommon::AdcCommon;
use crate::pac::adccommon::vals::{Ckmode, Presc};
use crate::time::Hertz;
use crate::wait::block_for_us;

/// Maximum ADC clock frequency.
const MAX_CLOCK: Hertz = Hertz::mhz(50);

/// Sample times as half ADC clock cycles, indexed by the `SMP` field value.
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [5, 13, 25, 49, 95, 185, 495, 1281];

/// Number of channels (`DIFSEL`/`AWDxCR` bits).
const CHANNELS: usize = 19;

fn to_res(res: Resolution) -> Adc3Res {
    match res {
        Resolution::Bits16 => panic!("this ADC has no 16-bit resolution"),
        Resolution::Bits14 => panic!("this ADC has no 14-bit resolution"),
        Resolution::Bits12 => Adc3Res::Bits12,
        Resolution::Bits10 => Adc3Res::Bits10,
        Resolution::Bits8 => Adc3Res::Bits8,
        Resolution::Bits6 => Adc3Res::Bits6,
    }
}

fn from_res(res: Adc3Res) -> Resolution {
    match res {
        Adc3Res::Bits12 => Resolution::Bits12,
        Adc3Res::Bits10 => Resolution::Bits10,
        Adc3Res::Bits8 => Resolution::Bits8,
        Adc3Res::Bits6 => Resolution::Bits6,
    }
}

const PRESCALERS: [Prescaler; 12] = [
    Prescaler::Div1,
    Prescaler::Div2,
    Prescaler::Div4,
    Prescaler::Div6,
    Prescaler::Div8,
    Prescaler::Div10,
    Prescaler::Div12,
    Prescaler::Div16,
    Prescaler::Div32,
    Prescaler::Div64,
    Prescaler::Div128,
    Prescaler::Div256,
];

fn to_presc(presc: Prescaler) -> Presc {
    match presc {
        Prescaler::Div1 => Presc::Div1,
        Prescaler::Div2 => Presc::Div2,
        Prescaler::Div4 => Presc::Div4,
        Prescaler::Div6 => Presc::Div6,
        Prescaler::Div8 => Presc::Div8,
        Prescaler::Div10 => Presc::Div10,
        Prescaler::Div12 => Presc::Div12,
        Prescaler::Div16 => Presc::Div16,
        Prescaler::Div32 => Presc::Div32,
        Prescaler::Div64 => Presc::Div64,
        Prescaler::Div128 => Presc::Div128,
        Prescaler::Div256 => Presc::Div256,
    }
}

impl super::BasicAdcRegs for Regs {
    type SampleTime = Adc3SampleTime;
}

impl AdcRegs for Regs {
    type Common = AdcCommon;

    const AWD_COUNT: usize = 3;
    const MAX_SEQUENCE_LEN: usize = 16;
    const INJECTED_RANKS: usize = 4;

    fn init(self, common: AdcCommon, kernel_clock: Hertz, config: &Config) {
        // Clock.
        match config.clock {
            Clock::Auto => {
                let presc = PRESCALERS
                    .iter()
                    .copied()
                    .find(|p| kernel_clock / p.divisor() <= MAX_CLOCK)
                    .expect("ADC kernel clock too fast, change 'config.rcc.mux' to a slower clock");
                critical_section::with(|_| {
                    common.ccr().modify(|w| {
                        w.set_ckmode(Ckmode::Asynchronous);
                        w.set_presc(to_presc(presc));
                    })
                });
            }
            Clock::Async(presc) => critical_section::with(|_| {
                common.ccr().modify(|w| {
                    w.set_ckmode(Ckmode::Asynchronous);
                    w.set_presc(to_presc(presc));
                })
            }),
            Clock::Sync(div) => critical_section::with(|_| {
                common.ccr().modify(|w| {
                    w.set_ckmode(match div {
                        SyncDiv::Div1 => Ckmode::SyncDiv1,
                        SyncDiv::Div2 => Ckmode::SyncDiv2,
                        SyncDiv::Div4 => Ckmode::SyncDiv4,
                    })
                })
            }),
        }
        let clock = self.clock(common, kernel_clock);
        assert!(
            clock <= MAX_CLOCK,
            "ADC clock {} exceeds the maximum of {}",
            clock,
            MAX_CLOCK
        );

        // Voltage regulator. There is no ready flag: wait for the start-up time from the datasheet
        // (RM0468 §29.4.6).
        self.cr().modify(|w| w.set_deeppwd(false));
        self.cr().modify(|w| w.set_advregen(true));
        block_for_us(20);

        // Calibration. ADEN cannot be set during four ADC clock cycles after ADCAL is cleared
        // (RM0468 §29.4.9).
        self.cr().modify(|w| w.set_adcaldif(false));
        self.cr().modify(|w| w.set_adcal(true));
        while self.cr().read().adcal() {}
        block_for_us(4_000_000u64.div_ceil(clock.0 as u64).max(1));

        self.enable();

        // Single conversion mode, software trigger.
        self.cfgr().modify(|w| {
            w.set_cont(false);
            w.set_discen(false);
            w.set_exten(Exten::Disabled);
        });

        if let Some(res) = config.resolution {
            self.set_resolution(res);
        }
        set_oversampling(self, config.oversampler());

        // ADC3 is not part of a dual-ADC pair: ADC3_COMMON has no DUAL, DELAY or DAMDF.
        assert!(
            config.dual_mode.is_none() && config.dual_data_format.is_none() && config.dual_delay.is_none(),
            "this ADC has no dual mode"
        );
    }

    fn clock(self, common: AdcCommon, kernel_clock: Hertz) -> Hertz {
        match common.ccr().read().ckmode() {
            Ckmode::SyncDiv1 => return kernel_clock,
            Ckmode::SyncDiv2 => return kernel_clock / 2u32,
            Ckmode::SyncDiv4 => return kernel_clock / 4u32,
            Ckmode::Asynchronous => {}
        }
        kernel_clock / PRESCALERS[common.ccr().read().presc().to_bits() as usize].divisor()
    }

    fn power_down(self) {
        if self.cr().read().aden() {
            self.cr().modify(|w| w.set_addis(true));
            while self.cr().read().aden() {}
        }
        self.cr().modify(|w| w.set_advregen(false));
        self.cr().modify(|w| w.set_deeppwd(true));
    }

    fn enable(self) {
        while self.cr().read().addis() {}
        if !self.cr().read().aden() {
            self.isr().write(|w| w.set_adrdy(true));
            self.cr().modify(|w| w.set_aden(true));
            while !self.isr().read().adrdy() {}
        }
    }

    fn set_resolution(self, res: Resolution) {
        self.cfgr().modify(|w| w.set_res(to_res(res)));
    }

    fn resolution(self) -> Resolution {
        from_res(self.cfgr().read().res())
    }

    fn configure_sequence(self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Adc3SampleTime)>, injected: bool) {
        let len = sequence.len();
        assert!(len != 0, "sequence cannot be empty");
        if injected {
            assert!(len <= Self::INJECTED_RANKS, "injected sequence too long");
        } else {
            assert!(len <= Self::MAX_SEQUENCE_LEN, "sequence too long");
        }

        let mut smpr = [self.smpr(0).read(), self.smpr(1).read()];
        let mut sqr1 = crate::pac::adc::regs::Sqr1::default();
        let mut sqr2 = crate::pac::adc::regs::Sqr2::default();
        let mut sqr3 = crate::pac::adc::regs::Sqr3::default();
        let mut sqr4 = crate::pac::adc::regs::Sqr4::default();
        let mut jsqr = self.jsqr().read();
        let mut difsel = self.difsel().read();
        let old_difsel = difsel;

        if injected {
            jsqr.set_jl(len as u8 - 1);
        } else {
            sqr1.set_l(len as u8 - 1);
        }

        for (i, ((channel, differential), sample_time)) in sequence.enumerate() {
            let channel = channel as usize;
            assert!(channel < CHANNELS, "channel {} does not exist", channel);
            smpr[channel / 10].set_smp(channel % 10, sample_time);
            difsel.set_difsel(channel, differential);

            if injected {
                jsqr.set_jsq(i, channel as u8);
            } else {
                match i {
                    0..=3 => sqr1.set_sq(i, channel as u8),
                    4..=8 => sqr2.set_sq(i - 4, channel as u8),
                    9..=13 => sqr3.set_sq(i - 9, channel as u8),
                    _ => sqr4.set_sq(i - 14, channel as u8),
                }
            }
        }

        // DIFSEL may only be written with the ADC disabled; it is rarely changed, so only pay for
        // the disable/enable cycle when it is.
        if difsel.0 != old_difsel.0 {
            if self.cr().read().aden() {
                self.cr().modify(|w| w.set_addis(true));
                while self.cr().read().aden() {}
            }
            self.difsel().write_value(difsel);
        }

        self.smpr(0).write_value(smpr[0]);
        self.smpr(1).write_value(smpr[1]);
        if injected {
            self.jsqr().write_value(jsqr);
        } else {
            self.sqr1().write_value(sqr1);
            self.sqr2().write_value(sqr2);
            self.sqr3().write_value(sqr3);
            self.sqr4().write_value(sqr4);
        }
    }

    fn configure_dma(self, mode: ConversionMode) {
        self.isr().write(|w| {
            w.set_eoc(true);
            w.set_eos(true);
            w.set_ovr(true);
        });
        self.cfgr().modify(|w| {
            w.set_discen(false);
            #[cfg(dfsdm_adc)]
            let dfsdm = matches!(mode, ConversionMode::Dfsdm(_));
            #[cfg(not(dfsdm_adc))]
            let dfsdm = false;
            #[cfg(dfsdm_adc)]
            let dfsdm_cont = matches!(mode, ConversionMode::Dfsdm(None));
            #[cfg(not(dfsdm_adc))]
            let dfsdm_cont = false;
            w.set_dmaen(!matches!(mode, ConversionMode::NoDma) && !dfsdm);
            w.set_dmacfg(match mode {
                ConversionMode::Repeated(_) => Adc3Dmacfg::Circular,
                _ => Adc3Dmacfg::OneShot,
            });
            w.set_dfsdmcfg(dfsdm);
            w.set_cont(matches!(mode, ConversionMode::Repeated(None)) || dfsdm_cont);
            w.set_ovrmod(matches!(mode, ConversionMode::Repeated(_)));
            match mode {
                ConversionMode::Repeated(Some((trigger, edge))) => {
                    w.set_extsel(trigger);
                    w.set_exten(edge);
                }
                #[cfg(dfsdm_adc)]
                ConversionMode::Dfsdm(Some((trigger, edge))) => {
                    w.set_extsel(trigger);
                    w.set_exten(edge);
                }
                _ => w.set_exten(Exten::Disabled),
            }
        });
    }

    fn start(self) {
        // ADC_DR is the output of a three-stage FIFO (RM0468 §29.4.26). A conversion whose result
        // was never read (an aborted `read` future, say) would otherwise be returned as the result
        // of this one, so pop whatever is still queued.
        while self.isr().read().eoc() {
            let _ = self.dr().read();
        }
        self.isr().write(|w| {
            w.set_eoc(true);
            w.set_eos(true);
            w.set_eosmp(true);
            w.set_ovr(true);
        });
        self.cr().modify(|w| w.set_adstart(true));
    }

    fn stop(self) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|w| w.set_adstp(true));
            while self.cr().read().adstart() {}
        }
    }

    fn done(self) -> bool {
        self.isr().read().eos()
    }

    fn data(self) -> *mut u16 {
        self.dr().as_ptr() as *mut u16
    }

    fn set_eoc_interrupt(self, enable: bool) {
        self.ier().modify(|w| w.set_eosie(enable));
    }

    fn on_interrupt(self, state: &State) {
        let isr = self.isr().read();
        let ier = self.ier().read();
        let mut wake = false;

        if ier.eosie() && isr.eos() {
            self.ier().modify(|w| w.set_eosie(false));
            wake = true;
        }
        if ier.jeosie() && isr.jeos() {
            self.isr().write(|w| w.set_jeos(true));
            state.injected_done.store(true, Ordering::Release);
            wake = true;
        }
        for i in 0..Self::AWD_COUNT {
            if ier.awdie(i) && isr.awd(i) {
                self.ier().modify(|w| w.set_awdie(i, false));
                self.isr().write(|w| w.set_awd(i, true));
                state.awd_triggered[i].store(true, Ordering::Release);
                wake = true;
            }
        }
        if wake {
            state.waker.wake();
        }
    }

    fn enable_internal(self, common: AdcCommon, channel: InternalChannel, enable: bool) {
        critical_section::with(|_| {
            common.ccr().modify(|w| match channel {
                InternalChannel::VrefInt => w.set_vrefen(enable),
                InternalChannel::Temperature => w.set_tsen(enable),
                InternalChannel::Vbat => w.set_vbaten(enable),
                #[allow(unreachable_patterns)]
                _ => {}
            })
        });
        match channel {
            InternalChannel::VddCore => panic!("this ADC has no VDDCORE channel"),
            InternalChannel::Dac(_) => panic!("this ADC has no DAC channel"),
            _ => {}
        }
        // Startup time of the internal reference and temperature sensor.
        if enable {
            block_for_us(15);
        }
    }

    fn configure_awd(self, index: usize, channels: WatchdogChannels, low: u32, high: u32) {
        let (low, high) = awd_thresholds(self, index, low, high);
        match index {
            0 => {
                self.cfgr().modify(|w| {
                    match channels {
                        WatchdogChannels::All => w.set_awd1sgl(false),
                        WatchdogChannels::Single(ch) => {
                            w.set_awd1sgl(true);
                            w.set_awd1ch(ch);
                        }
                        WatchdogChannels::Channels(_) => {
                            panic!("watchdog 1 monitors either a single channel or all channels")
                        }
                    }
                    w.set_awd1en(true);
                    w.set_jawd1en(true);
                });
            }
            1 | 2 => {
                let mask = match channels {
                    WatchdogChannels::All => panic!("watchdogs 2 and 3 monitor a set of channels, use `Channels`"),
                    WatchdogChannels::Single(ch) => 1u32 << ch,
                    WatchdogChannels::Channels(mask) => mask,
                };
                if index == 1 {
                    self.awd2cr().write(|w| w.0 = mask);
                } else {
                    self.awd3cr().write(|w| w.0 = mask);
                }
            }
            _ => panic!("this ADC has no watchdog {}", index + 1),
        }
        self.tr(index).write(|w| {
            w.set_lt(low as u16);
            w.set_ht(high as u16);
        });
    }

    fn disable_awd(self, index: usize) {
        match index {
            0 => self.cfgr().modify(|w| {
                w.set_awd1en(false);
                w.set_jawd1en(false);
            }),
            1 => self.awd2cr().write(|w| w.0 = 0),
            2 => self.awd3cr().write(|w| w.0 = 0),
            _ => {}
        }
    }

    fn set_awd_interrupt(self, index: usize, enable: bool) {
        if enable {
            self.isr().write(|w| w.set_awd(index, true));
        }
        self.ier().modify(|w| w.set_awdie(index, enable));
    }

    fn clear_awd_flag(self, index: usize) -> bool {
        let set = self.isr().read().awd(index);
        if set {
            self.isr().write(|w| w.set_awd(index, true));
        }
        set
    }

    fn set_continuous(self, enable: bool) {
        self.cfgr().modify(|w| w.set_cont(enable));
    }
}

/// Scale watchdog thresholds given in data-register units to the hardware comparison: 12-bit
/// thresholds compared against left-aligned data for watchdog 1, and 8-bit thresholds against the
/// 8 most significant bits for watchdogs 2 and 3 (RM0468 Tables 248 and 249).
fn awd_thresholds(regs: Regs, index: usize, low: u32, high: u32) -> (u32, u32) {
    let shift = 12 - regs.resolution().bits() as u32;
    if index == 0 {
        (low << shift, high << shift)
    } else {
        ((low << shift) >> 4, (high << shift) >> 4)
    }
}

impl InjectedRegs for Regs {
    fn configure_injected_trigger(self, trigger: (u8, Exten), interrupt: bool) {
        self.cfgr().modify(|w| w.set_jdiscen(false));
        self.jsqr().modify(|w| {
            w.set_jextsel(trigger.0);
            w.set_jexten(trigger.1);
        });
        self.isr().write(|w| w.set_jeos(true));
        self.ier().modify(|w| w.set_jeosie(interrupt));
    }

    fn start_injected(self) {
        self.cr().modify(|w| w.set_jadstart(true));
    }

    fn stop_injected(self) {
        if self.cr().read().jadstart() && !self.cr().read().addis() {
            self.cr().modify(|w| w.set_jadstp(true));
            while self.cr().read().jadstart() {}
        }
    }

    fn read_injected(self, data: &mut [u16]) {
        for (i, d) in data.iter_mut().enumerate() {
            *d = self.jdr(i).read().jdata();
        }
        self.isr().write(|w| w.set_jeos(true));
    }
}

fn set_oversampling(regs: Regs, oversampling: Option<Oversampling>) {
    regs.cfgr2().modify(|w| match oversampling {
        None => w.set_rovse(false),
        Some(o) => {
            assert!(o.ratio.log2() <= 8, "this ADC oversamples by at most 256");
            assert!(o.shift <= 8, "oversampling shift must be at most 8");
            w.set_ovsr(Adc3Ovsr::from_bits(o.ratio.log2() - 1));
            w.set_ovss(o.shift);
            w.set_trovs(if o.triggered {
                Trovs::Triggered
            } else {
                Trovs::Automatic
            });
            w.set_rovsm(if o.resumed { Rovsm::Resumed } else { Rovsm::Continued });
            w.set_rovse(true);
        }
    });
}

impl SampleTimes for Regs {
    fn sample_time_for_half_cycles(half_cycles: u32) -> Adc3SampleTime {
        let i = SAMPLE_TIME_HALF_CYCLES
            .iter()
            .position(|c| *c >= half_cycles)
            .unwrap_or(SAMPLE_TIME_HALF_CYCLES.len() - 1);
        Adc3SampleTime::from_bits(i as u8)
    }
}
