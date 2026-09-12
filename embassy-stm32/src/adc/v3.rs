//! The `v3` ADC (advanced): `ISR`/`CFGR`, a 16-entry regular sequence (`SQR1..4`), a 4-entry injected
//! sequence, per-channel sample times, differential inputs and three analog watchdogs.
//!
//! Chips: F30x (`adc_v3_f3`), L4/L5/WB55 (`adc_v3_l4`), G4/H5/H7RS (`adc_v3_g4`), H7 (`adc_v3_h7`), U5 `ADC1`/
//! `ADC2` (`adc_v3_u5`), U3 (`adc_v3_u3`), N6 (`adc_v3_n6`), C5 (`adc_v3_c5`). The common registers live in
//! the `ADCx_COMMON` block.

use core::sync::atomic::Ordering;

#[cfg(adc_oversampler)]
use super::Oversampling;
#[cfg(adc_presc_full)]
use super::Prescaler;
#[cfg(adc_sync_clock)]
use super::SyncDiv;
use super::injected::InjectedRegs;
use super::{
    AdcRegs, Clock, Config, ConversionMode, InternalChannel, Resolution, SampleTimes, State, WatchdogChannels,
};
use crate::pac::adc::Adc as Regs;
#[cfg(adc_v3_h7)]
use crate::pac::adc::vals::Boost;
#[cfg(any(adc_v3_f3, adc_v3_l4, adc_v3_g4))]
use crate::pac::adc::vals::Dmacfg;
#[cfg(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5))]
use crate::pac::adc::vals::Dmngt;
#[cfg(all(adc_oversampler, not(adc_oversampler_1024)))]
use crate::pac::adc::vals::Ovsr;
use crate::pac::adc::vals::{Exten, Res, SampleTime};
#[cfg(adc_oversampler)]
use crate::pac::adc::vals::{Rovsm, Trovs};
use crate::pac::adccommon::AdcCommon;
#[cfg(adc_sync_clock)]
use crate::pac::adccommon::vals::Ckmode;
#[cfg(adc_presc_full)]
use crate::pac::adccommon::vals::Presc;
use crate::time::Hertz;
use crate::wait::block_for_us;

/// Whether this is the 12-bit generation (`TR1..3`, `DMAEN`/`DMACFG`) or the 14/16-bit one
/// (`LTR`/`HTR`, `DMNGT`, `PCSEL`).
const BIG: bool = cfg!(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5));

/// Maximum ADC clock frequency.
#[cfg(stm32f3)]
const MAX_CLOCK: Hertz = Hertz::mhz(72);
#[cfg(any(stm32l4, stm32l4_plus, stm32l5))]
const MAX_CLOCK: Hertz = Hertz::mhz(80);
#[cfg(stm32wb)]
const MAX_CLOCK: Hertz = Hertz::mhz(64);
#[cfg(stm32g4)]
const MAX_CLOCK: Hertz = Hertz::mhz(60);
#[cfg(any(stm32h5, stm32h7rs))]
const MAX_CLOCK: Hertz = Hertz::mhz(75);
#[cfg(stm32h7)]
const MAX_CLOCK: Hertz = Hertz::mhz(50);
#[cfg(stm32u5)]
const MAX_CLOCK: Hertz = Hertz::mhz(55);
#[cfg(stm32u3)]
const MAX_CLOCK: Hertz = Hertz::mhz(48);
#[cfg(stm32n6)]
const MAX_CLOCK: Hertz = Hertz::mhz(70);
#[cfg(stm32c5)]
const MAX_CLOCK: Hertz = Hertz::mhz(70);

/// Sample times as half ADC clock cycles, indexed by the `SMP` field value.
#[cfg(adc_v3_f3)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 5, 9, 15, 39, 123, 363, 1203];
#[cfg(any(adc_v3_l4, adc_v3_g4))]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [5, 13, 25, 49, 95, 185, 495, 1281];
#[cfg(adc_v3_h7)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 5, 17, 33, 65, 129, 775, 1621];
#[cfg(adc_v3_u5)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [10, 12, 24, 40, 72, 136, 782, 1628];
#[cfg(adc_v3_u3)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 5, 13, 23, 47, 93, 493, 2999];
#[cfg(adc_v3_n6)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [5, 7, 15, 25, 49, 95, 495, 3003];
#[cfg(adc_v3_c5)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [6, 10, 16, 26, 50, 96, 278, 578];

/// Number of channels (`DIFSEL`/`PCSEL`/`AWDxCR` bits).
#[cfg(adc_v3_c5)]
const CHANNELS: usize = 14;
#[cfg(any(adc_v3_f3, adc_v3_l4))]
const CHANNELS: usize = 19;
#[cfg(not(any(adc_v3_c5, adc_v3_f3, adc_v3_l4)))]
const CHANNELS: usize = 20;

fn to_res(res: Resolution) -> Res {
    match res {
        #[cfg(adc_res16)]
        Resolution::Bits16 => Res::Bits16,
        #[cfg(adc_res14)]
        Resolution::Bits14 => Res::Bits14,
        Resolution::Bits12 => Res::Bits12,
        Resolution::Bits10 => Res::Bits10,
        Resolution::Bits8 => Res::Bits8,
        #[cfg(not(any(adc_v3_h7, adc_v3_u5)))]
        Resolution::Bits6 => Res::Bits6,
        // On the U5 the 6-bit resolution belongs to ADC4, not to ADC1/ADC2.
        #[cfg(adc_v3_u5)]
        Resolution::Bits6 => panic!("this ADC has no 6-bit resolution"),
    }
}

fn from_res(res: Res) -> Resolution {
    match res {
        #[cfg(adc_res16)]
        Res::Bits16 => Resolution::Bits16,
        #[cfg(adc_v3_h7)]
        Res::Bits14 | Res::Bits14v => Resolution::Bits14,
        #[cfg(adc_v3_h7)]
        Res::Bits12 | Res::Bits12v => Resolution::Bits12,
        #[cfg(all(adc_v3_u5, not(adc_v3_h7)))]
        Res::Bits14 => Resolution::Bits14,
        #[cfg(not(adc_v3_h7))]
        Res::Bits12 => Resolution::Bits12,
        Res::Bits10 => Resolution::Bits10,
        Res::Bits8 => Resolution::Bits8,
        #[cfg(not(any(adc_v3_h7, adc_v3_u5)))]
        Res::Bits6 => Resolution::Bits6,
        #[cfg(adc_v3_h7)]
        _ => Resolution::Bits16,
    }
}

#[cfg(adc_presc_full)]
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

#[cfg(adc_presc_full)]
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
    type SampleTime = SampleTime;
}

impl AdcRegs for Regs {
    type Common = AdcCommon;

    const AWD_COUNT: usize = 3;
    const MAX_SEQUENCE_LEN: usize = 16;
    const INJECTED_RANKS: usize = 4;

    fn init(self, common: AdcCommon, kernel_clock: Hertz, config: &Config) {
        // Clock.
        //
        // F3: the RCC selects the ADC clock (`config.rcc.adc`), but its CKMODE setting is lost
        // in the peripheral reset above, so restore the mode matching the RCC's clock.
        #[cfg(adc_v3_f3)]
        {
            let hclk = unsafe { crate::rcc::get_freqs().hclk1.to_hertz().unwrap() };
            let ckmode = match config.clock {
                Clock::Auto if kernel_clock == hclk => Ckmode::SyncDiv1,
                Clock::Auto if kernel_clock == hclk / 2u32 => Ckmode::SyncDiv2,
                Clock::Auto if kernel_clock == hclk / 4u32 => Ckmode::SyncDiv4,
                Clock::Auto => Ckmode::Asynchronous,
                Clock::Sync(SyncDiv::Div1) => Ckmode::SyncDiv1,
                Clock::Sync(SyncDiv::Div2) => Ckmode::SyncDiv2,
                Clock::Sync(SyncDiv::Div4) => Ckmode::SyncDiv4,
            };
            critical_section::with(|_| common.ccr().modify(|w| w.set_ckmode(ckmode)));
        }
        // N6 and C5 have no prescaler: the RCC clock is used as is.
        #[cfg(not(adc_v3_f3))]
        match config.clock {
            Clock::Auto => {
                #[cfg(adc_presc_full)]
                {
                    let presc = PRESCALERS
                        .iter()
                        .copied()
                        .find(|p| kernel_clock / p.divisor() <= MAX_CLOCK)
                        .expect("ADC kernel clock too fast, change 'config.rcc.mux' to a slower clock");
                    critical_section::with(|_| {
                        common.ccr().modify(|w| {
                            #[cfg(adc_sync_clock)]
                            w.set_ckmode(Ckmode::Asynchronous);
                            w.set_presc(to_presc(presc));
                        })
                    });
                }
            }
            #[cfg(adc_presc_full)]
            Clock::Async(presc) => critical_section::with(|_| {
                common.ccr().modify(|w| {
                    #[cfg(adc_sync_clock)]
                    w.set_ckmode(Ckmode::Asynchronous);
                    w.set_presc(to_presc(presc));
                })
            }),
            #[cfg(adc_sync_clock)]
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

        #[cfg(adc_v3_h7)]
        self.cr().modify(|w| {
            w.set_boost(if clock < Hertz::khz(6_250) {
                Boost::Lt625
            } else if clock < Hertz::khz(12_500) {
                Boost::Lt125
            } else if clock < Hertz::mhz(25) {
                Boost::Lt25
            } else {
                Boost::Lt50
            })
        });

        // Voltage regulator.
        self.cr().modify(|w| w.set_deeppwd(false));
        #[cfg(not(adc_v3_n6))]
        self.cr().modify(|w| w.set_advregen(true));
        #[cfg(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_c5))]
        while !self.isr().read().ldordy() {}
        block_for_us(20);

        // Calibration.
        #[cfg(not(adc_v3_n6))]
        {
            #[cfg(any(adc_v3_f3, adc_v3_l4, adc_v3_g4, adc_v3_h7))]
            self.cr().modify(|w| w.set_adcaldif(false));
            #[cfg(any(adc_v3_h7, adc_v3_u5))]
            self.cr().modify(|w| w.set_adcallin(true));
            self.cr().modify(|w| w.set_adcal(true));
            while self.cr().read().adcal() {}
            block_for_us(1);
        }
        #[cfg(adc_v3_n6)]
        calibrate_n6(self);

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
        #[cfg(adc_oversampler)]
        set_oversampling(self, config.oversampler());

        #[cfg(any(adccommon_v3, adccommon_v4))]
        {
            let dual = config.dual_mode;
            #[cfg(adccommon_v4)]
            let damdf = config.dual_data_format;
            let delay = config.dual_delay;
            #[cfg(adccommon_v4)]
            let any = dual.is_some() || damdf.is_some() || delay.is_some();
            #[cfg(not(adccommon_v4))]
            let any = dual.is_some() || delay.is_some();
            if any {
                critical_section::with(|_| {
                    common.ccr().modify(|w| {
                        if let Some(dual) = dual {
                            w.set_dual(dual);
                        }
                        #[cfg(adccommon_v4)]
                        if let Some(damdf) = damdf {
                            w.set_damdf(damdf);
                        }
                        if let Some(delay) = delay {
                            w.set_delay(delay);
                        }
                    })
                });
            }
        }
    }

    fn clock(self, common: AdcCommon, kernel_clock: Hertz) -> Hertz {
        #[cfg(any(adc_v3_l4, adc_v3_g4, adc_v3_h7))]
        match common.ccr().read().ckmode() {
            Ckmode::SyncDiv1 => return kernel_clock,
            Ckmode::SyncDiv2 => return kernel_clock / 2u32,
            Ckmode::SyncDiv4 => return kernel_clock / 4u32,
            Ckmode::Asynchronous => {}
        }
        #[cfg(adc_v3_f3)]
        {
            // `kernel_clock` is the RCC's ADC clock; in synchronous mode divide HCLK instead.
            let hclk = unsafe { crate::rcc::get_freqs().hclk1.to_hertz().unwrap() };
            return match common.ccr().read().ckmode() {
                Ckmode::SyncDiv1 => hclk,
                Ckmode::SyncDiv2 => hclk / 2u32,
                Ckmode::SyncDiv4 => hclk / 4u32,
                Ckmode::Asynchronous => kernel_clock,
            };
        }
        #[cfg(any(adc_v3_l4, adc_v3_g4, adc_v3_h7, adc_v3_u5, adc_v3_u3))]
        {
            kernel_clock / PRESCALERS[common.ccr().read().presc().to_bits() as usize].divisor()
        }
        #[cfg(any(adc_v3_n6, adc_v3_c5))]
        {
            let _ = common;
            kernel_clock
        }
    }

    fn power_down(self) {
        if self.cr().read().aden() {
            self.cr().modify(|w| w.set_addis(true));
            while self.cr().read().aden() {}
        }
        #[cfg(not(adc_v3_n6))]
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

    fn configure_sequence(self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>, injected: bool) {
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
        #[cfg(not(any(adc_v3_u3, adc_v3_c5)))]
        let mut difsel = self.difsel().read();
        #[cfg(not(any(adc_v3_u3, adc_v3_c5)))]
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

            #[cfg(not(any(adc_v3_u3, adc_v3_c5)))]
            difsel.set_difsel(channel, differential);
            #[cfg(any(adc_v3_u3, adc_v3_c5))]
            assert!(!differential, "this ADC has no differential inputs");

            #[cfg(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5))]
            self.pcsel().modify(|w| w.set_pcsel(channel, true));

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
        #[cfg(not(any(adc_v3_u3, adc_v3_c5)))]
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
            #[cfg(not(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5)))]
            {
                w.set_dmaen(!matches!(mode, ConversionMode::NoDma));
                #[cfg(not(adc_v3_f3))]
                w.set_dmacfg(match mode {
                    ConversionMode::Repeated(_) => Dmacfg::Circular,
                    _ => Dmacfg::OneShot,
                });
                #[cfg(adc_v3_f3)]
                w.set_dmacfg(match mode {
                    ConversionMode::Repeated(_) => Dmacfg::Circular,
                    _ => Dmacfg::OneShot,
                });
            }
            #[cfg(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5))]
            w.set_dmngt(match mode {
                ConversionMode::NoDma => Dmngt::Dr,
                ConversionMode::Singular => Dmngt::DmaOneShot,
                ConversionMode::Repeated(_) => Dmngt::DmaCircular,
            });
            w.set_cont(matches!(mode, ConversionMode::Repeated(None)));
            w.set_ovrmod(matches!(mode, ConversionMode::Repeated(_)));
            match mode {
                ConversionMode::Repeated(Some((trigger, edge))) => {
                    w.set_extsel(trigger);
                    w.set_exten(edge);
                }
                _ => w.set_exten(Exten::Disabled),
            }
        });
    }

    fn start(self) {
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
            #[cfg(any(stm32h5, stm32h7rs))]
            InternalChannel::VddCore => self.or().modify(|w| w.set_op0(enable)),
            #[cfg(any(adc_v3_u3, adc_v3_n6))]
            InternalChannel::VddCore => self.or().modify(|w| w.set_vddcoreen(enable)),
            #[cfg(not(any(stm32h5, stm32h7rs, adc_v3_u3, adc_v3_n6)))]
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
        #[cfg(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5))]
        {
            self.ltr(index).write(|w| w.set_ltr(low));
            self.htr(index).write(|w| w.set_htr(high));
        }
        #[cfg(not(any(adc_v3_h7, adc_v3_u5, adc_v3_u3, adc_v3_n6, adc_v3_c5)))]
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

    #[cfg(any(adc_v3_u5, adc_v3_u3, stm32h5, stm32h7rs))]
    fn set_low_frequency_trigger(self, enable: bool) {
        self.cfgr2().modify(|w| w.set_lftrig(enable));
    }
}

/// Scale watchdog thresholds given in data-register units to the hardware comparison.
fn awd_thresholds(regs: Regs, index: usize, low: u32, high: u32) -> (u32, u32) {
    if BIG {
        // Wide threshold registers compare the full converted data.
        let _ = (regs, index);
        (low, high)
    } else {
        // 12-bit thresholds compared against left-aligned data for watchdog 1, and 8-bit
        // thresholds against the 8 most significant bits for watchdogs 2 and 3.
        let bits = regs.resolution().bits() as u32;
        let shift = 12 - bits;
        if index == 0 {
            (low << shift, high << shift)
        } else {
            (low >> (bits - 8), high >> (bits - 8))
        }
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
            *d = self.jdr(i).read().jdata() as u16;
        }
        self.isr().write(|w| w.set_jeos(true));
    }
}

#[cfg(adc_oversampler)]
fn set_oversampling(regs: Regs, oversampling: Option<Oversampling>) {
    regs.cfgr2().modify(|w| match oversampling {
        None => w.set_rovse(false),
        Some(o) => {
            #[cfg(not(adc_oversampler_1024))]
            {
                assert!(o.shift <= 8, "oversampling shift must be at most 8");
                w.set_ovsr(Ovsr::from_bits(o.ratio.log2() - 1));
            }
            #[cfg(adc_oversampler_1024)]
            {
                assert!(o.shift <= 11, "oversampling shift must be at most 11");
                w.set_ovsr((1u16 << o.ratio.log2()) - 1);
            }
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

/// STM32N6 reference manual 32.4.8: software procedure to calibrate the ADC.
#[cfg(adc_v3_n6)]
fn calibrate_n6(regs: Regs) {
    const ADC_MIDPOINT: u64 = 0x7ff;
    // Steps 4 to 8
    let sample_and_average = || -> u64 {
        let mut data = [0u64; 8];
        for reading in &mut data {
            // 4. Set the ADSTART bit in the ADC_CR register.
            regs.cr().modify(|w| w.set_adstart(true));
            // 5. Wait until the ADSTART bit is cleared or the EOC flag is set.
            while regs.cr().read().adstart() && !regs.isr().read().eoc() {}
            // 6. Read the ADC_DR register, then copy the converted data to the memory.
            *reading = regs.dr().read().rdata() as u64;
            // 7. Repeat from step 4 several times (for example eight times).
        }
        // 8. Average the data stored in memory by dividing the accumulated data by the
        // number of the conversions
        data.iter().sum::<u64>() / data.len() as u64
    };
    // 1. Ensure DEEPPWD = 0, ADEN = 1 and wait until the ADRDY bit is set.
    regs.cr().modify(|reg| reg.set_deeppwd(false));
    block_for_us(1);
    regs.enable();
    // 2. Set ADCAL and ensure CALADDOS = 0.
    regs.cr().modify(|w| w.set_adcal(true));
    regs.calfact().modify(|w| w.set_caladdos(false));
    // 3. Select the calibration input mode by clearing ADCALDIF (single-ended input).
    regs.cr().modify(|w| w.set_adcaldif(false));
    // Steps 4 to 8
    let mut average = sample_and_average();
    // 9. If the averaged data is zero, set CALADDOS. Repeat all steps from step 4.
    if average == 0 {
        regs.calfact().modify(|w| w.set_caladdos(true));
        average = sample_and_average();
    }
    // 10. Store the averaged data to CALFACT_S[8:0].
    regs.calfact().modify(|w| w.set_calfact_s(average as u16));
    // 11. Select the calibration input mode by setting ADCALDIF (differential input).
    regs.cr().modify(|w| w.set_adcaldif(true));
    // 12. Keep the same CALADDOS setting as the one obtained during the single-end
    // calibration.
    // 13. Repeat steps 4 to 8.
    average = sample_and_average();
    // 14. Subtract 0x7FF from the averaged data. If the result is positive, store it in the
    // CALFACT_D[8:0] bitfield. If it is negative, set CALADDOS, then repeat steps from 4 to
    // 8.
    if average < ADC_MIDPOINT {
        regs.calfact().modify(|w| w.set_caladdos(true));
        average = sample_and_average();
    }
    // 15. Subtract again 0x7FF from the new averaged data. The resulting value is positive.
    // Store it in CALFACT_D[8:0].
    let result = average.saturating_sub(ADC_MIDPOINT) as u16;
    regs.calfact().modify(|w| w.set_calfact_d(result));
    // 16. CALADDOS is now set, so clear ADCALDIF, and repeat steps 4 to 8..
    regs.cr().modify(|w| w.set_adcaldif(false));
    average = sample_and_average();
    // 17. Store the averaged data in CALFACT_S[8:0].
    regs.calfact().modify(|w| w.set_calfact_s(average as u16));
    // 18. Clear ADCAL bit.
    regs.cr().modify(|w| w.set_adcal(false));
    block_for_us(1);
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
