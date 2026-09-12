//! The `v2` ADC (basic): one regular sequence selected by a channel bitmask (or an 8-entry sequencer),
//! one or two sample times shared by all channels, no injected conversions.
//!
//! Chips: F0 (`adc_v2_f0`), L0 (`adc_v2_l0`), WB10/WB15 (`adc_v2_wb1`), G0/C0/U0/WL (`adc_v2_g0`), and the
//! `ADC4` of U5 (`adc_v2_u5`) and the WBA (`adc_v2_wba`). The common registers (`CCR`) live inside the ADC block.

use core::sync::atomic::Ordering;

#[cfg(adc_oversampler)]
use super::Oversampling;
#[cfg(adc_presc_full)]
use super::Prescaler;
#[cfg(adc_sync_clock)]
use super::SyncDiv;
use super::{
    AdcRegs, Clock, Config, ConversionMode, InternalChannel, Resolution, SampleTimes, State, WatchdogChannels,
};
#[cfg(not(adc_v2_u5))]
use crate::pac::adc::Adc as Regs;
#[cfg(adc_v2_u5)]
use crate::pac::adc::Adc4 as Regs;
#[cfg(adc_sync_clock)]
use crate::pac::adc::vals::Ckmode;
#[cfg(all(adc_oversampler, not(adc_v2_u5)))]
use crate::pac::adc::vals::Ovsr;
#[cfg(all(adc_presc_full, not(adc_v2_u5)))]
use crate::pac::adc::vals::Presc;
#[cfg(adc_v2_u5)]
use crate::pac::adc::vals::{
    Adc4Align as Align, Adc4Dmacfg as Dmacfg, Adc4Exten as Exten, Adc4Ovsr as Ovsr, Adc4Presc as Presc, Adc4Res as Res,
    Adc4SampleTime as SampleTime, Adc4Scandir as Scandir,
};
#[cfg(not(adc_v2_u5))]
use crate::pac::adc::vals::{Align, Dmacfg, Exten, Res, SampleTime, Scandir};
use crate::time::Hertz;
use crate::wait::block_for_us;

/// Maximum ADC clock frequency.
#[cfg(stm32f0)]
const MAX_CLOCK: Hertz = Hertz::mhz(14);
#[cfg(stm32l0)]
const MAX_CLOCK: Hertz = Hertz::mhz(16);
#[cfg(adc_v2_wb1)]
const MAX_CLOCK: Hertz = Hertz::mhz(16);
#[cfg(stm32c0)]
const MAX_CLOCK: Hertz = Hertz::mhz(25);
#[cfg(any(stm32g0, stm32u0, stm32wl))]
const MAX_CLOCK: Hertz = Hertz::mhz(35);
#[cfg(any(adc_v2_u5, adc_v2_wba))]
const MAX_CLOCK: Hertz = Hertz::mhz(55);

/// Frequency of the dedicated asynchronous clock of the F0 (HSI14) and L0 (HSI16) ADC.
#[cfg(stm32f0)]
const ASYNC_CLOCK: Hertz = Hertz::mhz(14);
#[cfg(stm32l0)]
const ASYNC_CLOCK: Hertz = Hertz::mhz(16);

/// Whether the sequencer-mode `CHSELR` (CHSELRMOD = 1) exists.
const HAS_SEQUENCER: bool = cfg!(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba));
/// Channels the sequencer can address (4-bit entries, 0b1111 terminates the sequence).
const SEQUENCER_MAX_CHANNEL: u8 = 14;
const SEQUENCER_LEN: usize = 8;
/// Number of distinct sample times that can be used in one sequence.
const SAMPLE_TIME_SLOTS: usize = if HAS_SEQUENCER { 2 } else { 1 };

/// Sample times as half ADC clock cycles, indexed by the `SMP` field value.
#[cfg(adc_v2_f0)]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 15, 27, 57, 83, 111, 143, 479];
#[cfg(any(adc_v2_l0, adc_v2_wb1, adc_v2_g0))]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 7, 15, 25, 39, 79, 159, 321];
#[cfg(any(adc_v2_u5, adc_v2_wba))]
const SAMPLE_TIME_HALF_CYCLES: [u32; 8] = [3, 7, 15, 25, 39, 79, 159, 1629];

fn to_res(res: Resolution) -> Res {
    match res {
        Resolution::Bits12 => Res::Bits12,
        Resolution::Bits10 => Res::Bits10,
        Resolution::Bits8 => Res::Bits8,
        Resolution::Bits6 => Res::Bits6,
        // On the U5 the 14-bit resolution belongs to ADC1/ADC2, not to this ADC4.
        #[cfg(adc_res14)]
        Resolution::Bits14 => panic!("this ADC has no 14-bit resolution"),
    }
}

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

/// The prescaler values of the `PRESC` field, in order.
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

#[cfg(adc_sync_clock)]
fn to_ckmode(div: SyncDiv) -> Ckmode {
    match div {
        #[cfg(adc_sync_div1)]
        SyncDiv::Div1 => Ckmode::SyncDiv1,
        SyncDiv::Div2 => Ckmode::SyncDiv2,
        SyncDiv::Div4 => Ckmode::SyncDiv4,
    }
}

/// The clock configuration actually applied.
#[derive(Copy, Clone)]
enum ClockSetup {
    /// Asynchronous kernel clock, with the given prescaler applied.
    #[cfg(adc_presc_full)]
    Async(Prescaler),
    /// Synchronous bus clock divided by the given divider.
    #[cfg(adc_sync_clock)]
    Sync(SyncDiv),
}

/// Frequency of the asynchronous clock feeding the ADC: the RCC kernel clock, or the fixed
/// oscillator on chips where it is not modelled by the RCC driver.
fn async_clock(kernel_clock: Hertz) -> Hertz {
    #[cfg(any(stm32f0, stm32l0))]
    {
        let _ = kernel_clock;
        ASYNC_CLOCK
    }
    #[cfg(not(any(stm32f0, stm32l0)))]
    kernel_clock
}

fn choose_clock(clock: Clock, kernel_clock: Hertz) -> ClockSetup {
    match clock {
        #[cfg(adc_presc_full)]
        Clock::Async(presc) => ClockSetup::Async(presc),
        #[cfg(adc_sync_clock)]
        Clock::Sync(div) => ClockSetup::Sync(div),
        Clock::Auto => {
            // F0/L0: the asynchronous clock is a dedicated oscillator that may not be running,
            // so derive the clock from the bus instead.
            #[cfg(any(stm32f0, stm32l0))]
            {
                for div in [SyncDiv::Div2, SyncDiv::Div4] {
                    if kernel_clock / div.divisor() <= MAX_CLOCK {
                        return ClockSetup::Sync(div);
                    }
                }
                panic!("PCLK too fast for the ADC, use a slower PCLK");
            }
            #[cfg(not(any(stm32f0, stm32l0)))]
            {
                for presc in PRESCALERS {
                    if kernel_clock / presc.divisor() <= MAX_CLOCK {
                        return ClockSetup::Async(presc);
                    }
                }
                panic!("ADC kernel clock too fast, change 'config.rcc.mux' to a slower clock");
            }
        }
    }
}

/// Apply one channel configuration change (a `CHSELR` write, or a `CHSELRMOD`/`SCANDIR` change).
///
/// On WB1x/G0/C0/U0/WL the hardware takes a while to apply it and flags completion with `CCRDY`;
/// the flag must be clear before the next change, and conversions must not be started before it
/// is set.
fn channel_config(regs: Regs, f: impl FnOnce()) {
    #[cfg(any(adc_v2_wb1, adc_v2_g0))]
    regs.isr().write(|w| w.set_ccrdy(true));
    f();
    #[cfg(any(adc_v2_wb1, adc_v2_g0))]
    while !regs.isr().read().ccrdy() {}
    #[cfg(not(any(adc_v2_wb1, adc_v2_g0)))]
    let _ = regs;
}

/// Disable the converter (keeping the regulator on), waiting until it is off.
trait Disable {
    fn disable(self);
}

impl Disable for Regs {
    fn disable(self) {
        while self.cr().read().addis() {}
        if self.cr().read().aden() {
            self.stop();
            self.cr().modify(|w| w.set_addis(true));
            while self.cr().read().aden() {}
        }
    }
}

impl super::BasicAdcRegs for Regs {
    type SampleTime = SampleTime;
}

impl AdcRegs for Regs {
    type Common = ();

    const AWD_COUNT: usize = if cfg!(any(adc_v2_g0, adc_v2_u5, adc_v2_wba)) {
        3
    } else {
        1
    };
    /// No injected conversions on this ADC.
    #[cfg(any(adc_v1, adc_v3))]
    const INJECTED_RANKS: usize = 0;
    const MAX_SEQUENCE_LEN: usize = 32;

    fn init(self, _common: (), kernel_clock: Hertz, config: &Config) {
        // Clock. Written before anything else: some ADCs (L0) ignore the write once the voltage
        // regulator is on.
        match choose_clock(config.clock, kernel_clock) {
            #[cfg(adc_presc_full)]
            ClockSetup::Async(presc) => {
                self.ccr().modify(|w| w.set_presc(to_presc(presc)));
                #[cfg(adc_sync_clock)]
                self.cfgr2().modify(|w| w.set_ckmode(Ckmode::Asynchronous));
            }
            #[cfg(adc_sync_clock)]
            ClockSetup::Sync(div) => self.cfgr2().modify(|w| w.set_ckmode(to_ckmode(div))),
        }
        // Voltage regulator.
        #[cfg(not(adc_v2_f0))]
        {
            self.cr().modify(|w| w.set_advregen(true));
            #[cfg(any(adc_v2_u5, adc_v2_wba))]
            {
                self.isr().write(|w| w.set_ldordy(true));
                while !self.isr().read().ldordy() {}
            }
            #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
            block_for_us(20);
        }

        let clock = self.clock((), kernel_clock);
        assert!(
            clock <= MAX_CLOCK,
            "ADC clock {} exceeds the maximum of {}",
            clock,
            MAX_CLOCK
        );

        // Calibration. Auto-off must be disabled during the calibration.
        self.cfgr1().modify(|w| w.set_dmaen(false));
        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        let auto_off = self.cfgr1().read().autoff();
        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        self.cfgr1().modify(|w| w.set_autoff(false));
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        let auto_off = self.pwrr().read().autoff();
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        self.pwrr().modify(|w| w.set_autoff(false));

        self.cr().modify(|w| w.set_adcal(true));
        while self.cr().read().adcal() {}
        #[cfg(not(adc_v2_f0))]
        self.isr().write(|w| w.set_eocal(true));

        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        self.cfgr1().modify(|w| w.set_autoff(auto_off));
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        self.pwrr().modify(|w| w.set_autoff(auto_off));

        block_for_us(1);

        // Single conversion mode, software trigger, right-aligned data.
        self.cfgr1().modify(|w| {
            w.set_cont(false);
            w.set_discen(false);
            w.set_exten(Exten::Disabled);
            w.set_align(Align::Right);
            #[cfg(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            w.set_chselrmod(false);
        });

        // The resolution and CFGR2 must be written with the ADC disabled (writing CFGR2 with
        // ADEN set clears CKMODE on the L0).
        if let Some(res) = config.resolution {
            self.set_resolution(res);
        }
        #[cfg(adc_oversampler)]
        set_oversampling(self, config.oversampler());

        self.enable();
    }

    fn clock(self, _common: (), kernel_clock: Hertz) -> Hertz {
        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        match self.cfgr2().read().ckmode() {
            Ckmode::SyncDiv2 => return kernel_clock / 2u32,
            Ckmode::SyncDiv4 => return kernel_clock / 4u32,
            #[cfg(not(adc_v2_f0))]
            Ckmode::SyncDiv1 => return kernel_clock,
            _ => {}
        }
        #[cfg(any(adc_v2_l0, adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
        let presc = PRESCALERS[self.ccr().read().presc().to_bits() as usize].divisor();
        #[cfg(adc_v2_f0)]
        let presc = 1u32;
        async_clock(kernel_clock) / presc
    }

    fn power_down(self) {
        self.disable();
        #[cfg(not(adc_v2_f0))]
        self.cr().modify(|w| w.set_advregen(false));
    }

    fn enable(self) {
        // In auto-off mode the ADC powers itself up on demand and ADRDY would never be set.
        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        if self.cfgr1().read().autoff() {
            return;
        }
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        if self.pwrr().read().autoff() {
            return;
        }

        while self.cr().read().addis() {}
        if !self.cr().read().aden() {
            self.isr().write(|w| w.set_adrdy(true));
            self.cr().modify(|w| w.set_aden(true));
            while !self.isr().read().adrdy() {
                // ES0233 2.4.3 (F0): ADEN cannot be set right after a calibration. Keep setting
                // it until the ADC is ready.
                #[cfg(adc_v2_f0)]
                self.cr().modify(|w| w.set_aden(true));
            }
        }
    }

    fn set_resolution(self, res: Resolution) {
        // RES may only be changed with the ADC disabled; the next conversion re-enables it.
        self.disable();
        self.cfgr1().modify(|w| w.set_res(to_res(res)));
    }

    fn resolution(self) -> Resolution {
        match self.cfgr1().read().res() {
            Res::Bits12 => Resolution::Bits12,
            Res::Bits10 => Resolution::Bits10,
            Res::Bits8 => Resolution::Bits8,
            Res::Bits6 => Resolution::Bits6,
        }
    }

    fn configure_sequence(self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>, _injected: bool) {
        let len = sequence.len();
        assert!(len != 0, "sequence cannot be empty");

        // Sample time slots: one on F0/L0, two on the others (SMP[1] selected per channel).
        let mut sample_times: [Option<SampleTime>; 2] = [None, None];
        let mut sample_slot = [0u8; 32];
        let mut channels = [0u8; 32];
        assert!(len <= channels.len(), "sequence too long");

        let mut ascending = true;
        let mut descending = true;
        let mut sequencer_ok = HAS_SEQUENCER && len > 1 && len <= SEQUENCER_LEN;
        let mut mask = 0u32;

        for (i, ((channel, _differential), sample_time)) in sequence.enumerate() {
            channels[i] = channel;
            mask |= 1 << channel;
            if i > 0 {
                ascending &= channel > channels[i - 1];
                descending &= channel < channels[i - 1];
            }
            if channel > SEQUENCER_MAX_CHANNEL {
                sequencer_ok = false;
            }
            let slot = match sample_times.iter().position(|s| *s == Some(sample_time)) {
                Some(slot) => slot,
                None => {
                    let slot = sample_times.iter().position(|s| s.is_none()).unwrap_or(usize::MAX);
                    assert!(
                        slot < SAMPLE_TIME_SLOTS,
                        "this ADC supports only {} distinct sample time(s) per sequence",
                        SAMPLE_TIME_SLOTS
                    );
                    sample_times[slot] = Some(sample_time);
                    slot
                }
            };
            sample_slot[i] = slot as u8;
        }

        self.smpr().modify(|w| {
            for (slot, st) in sample_times.iter().enumerate() {
                if let Some(st) = st {
                    w.set_smp(slot, *st);
                }
            }
            #[cfg(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            for i in 0..len {
                w.set_smpsel(channels[i] as usize, sample_slot[i] != 0);
            }
        });
        #[cfg(not(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba)))]
        let _ = sample_slot;

        if sequencer_ok {
            // Fully configurable sequencer: any order, repeats allowed.
            #[cfg(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            {
                if !self.cfgr1().read().chselrmod() {
                    channel_config(self, || self.cfgr1().modify(|w| w.set_chselrmod(true)));
                }
                channel_config(self, || {
                    self.chselr_sq().write(|w| {
                        for (i, ch) in channels.iter().enumerate().take(SEQUENCER_LEN) {
                            w.set_sq(i, if i < len { *ch } else { 0b1111 });
                        }
                    })
                });
            }
        } else {
            assert!(
                ascending || descending,
                "this ADC scans channels in ascending or descending order only (or up to 8 channels below 15 in any order)"
            );
            #[cfg(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            if self.cfgr1().read().chselrmod() {
                channel_config(self, || self.cfgr1().modify(|w| w.set_chselrmod(false)));
            }
            channel_config(self, || self.chselr().write(|w| w.0 = mask));
            let scandir = if ascending { Scandir::Upward } else { Scandir::Backward };
            if self.cfgr1().read().scandir() != scandir {
                channel_config(self, || self.cfgr1().modify(|w| w.set_scandir(scandir)));
            }
        }
    }

    fn configure_dma(self, mode: ConversionMode) {
        self.isr().write(|w| {
            w.set_eoc(true);
            w.set_eos(true);
            w.set_ovr(true);
        });
        self.cfgr1().modify(|w| {
            w.set_discen(false);
            w.set_dmaen(!matches!(mode, ConversionMode::NoDma));
            w.set_dmacfg(match mode {
                ConversionMode::Repeated(_) => Dmacfg::Circular,
                _ => Dmacfg::OneShot,
            });
            w.set_cont(matches!(mode, ConversionMode::Repeated(None)));
            // Keep the data register when a sample is missed in one-shot mode; overwrite it
            // when streaming, so the newest sample wins.
            w.set_ovrmod(matches!(mode, ConversionMode::Repeated(_)));
            match mode {
                ConversionMode::Repeated(Some((trigger, edge))) => {
                    w.set_extsel(trigger);
                    // On the U5 `super::Exten` is the ADC1/ADC2 enum; the values are the same.
                    #[cfg(adc_v2_u5)]
                    w.set_exten(Exten::from_bits(edge.to_bits()));
                    #[cfg(not(adc_v2_u5))]
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
        if self.cr().read().adstart() {
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

    fn enable_internal(self, _common: (), channel: InternalChannel, enable: bool) {
        critical_section::with(|_| {
            self.ccr().modify(|w| match channel {
                InternalChannel::VrefInt => w.set_vrefen(enable),
                InternalChannel::Temperature => w.set_tsen(enable),
                #[cfg(not(adc_v2_l0))]
                InternalChannel::Vbat => w.set_vbaten(enable),
                #[cfg(any(adc_v2_u5, adc_v2_wba))]
                InternalChannel::VddCore => w.set_vddcoren(enable),
                #[cfg(any(adc_v2_u5, adc_v2_wba))]
                InternalChannel::Dac(_) => {}
                #[allow(unreachable_patterns)]
                _ => panic!("internal channel not available on this ADC"),
            })
        });
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        if let InternalChannel::Dac(channel) = channel {
            self.or().modify(|w| w.set_chn21sel(channel == 1));
        }
        // Startup time of the internal reference and temperature sensor.
        if enable {
            block_for_us(15);
        }
    }

    fn configure_awd(self, index: usize, channels: WatchdogChannels, low: u32, high: u32) {
        // The comparison is done on 12-bit data: left-aligned raw data at lower resolutions, or
        // the 12 most significant bits of the 16-bit result when oversampling.
        #[cfg(any(adc_v2_l0, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
        let oversampling = self.cfgr2().read().ovse();
        #[cfg(not(any(adc_v2_l0, adc_v2_g0, adc_v2_u5, adc_v2_wba)))]
        let oversampling = false;
        let (low, high) = if oversampling {
            ((low >> 4) as u16, (high >> 4) as u16)
        } else {
            let shift = 12 - self.resolution().bits();
            ((low << shift) as u16, (high << shift) as u16)
        };
        match index {
            0 => {
                self.tr(0).write(|w| {
                    w.set_lt(low);
                    w.set_ht(high);
                });
                self.cfgr1().modify(|w| {
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
                });
            }
            #[cfg(any(adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            1 | 2 => {
                self.tr(index).write(|w| {
                    w.set_lt(low);
                    w.set_ht(high);
                });
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
    }

    fn disable_awd(self, index: usize) {
        match index {
            0 => self.cfgr1().modify(|w| w.set_awd1en(false)),
            #[cfg(any(adc_v2_g0, adc_v2_u5, adc_v2_wba))]
            1 => self.awd2cr().write(|w| w.0 = 0),
            #[cfg(any(adc_v2_g0, adc_v2_u5, adc_v2_wba))]
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
        self.cfgr1().modify(|w| w.set_cont(enable));
    }

    #[cfg(any(adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba))]
    fn set_low_frequency_trigger(self, enable: bool) {
        // CFGR2 must be written with the ADC disabled; the next conversion re-enables it.
        self.disable();
        self.cfgr2().modify(|w| w.set_lftrig(enable));
    }
}

#[cfg(adc_oversampler)]
fn set_oversampling(regs: Regs, oversampling: Option<Oversampling>) {
    regs.cfgr2().modify(|w| match oversampling {
        None => w.set_ovse(false),
        Some(o) => {
            // On the U5 the 10-bit ratios belong to ADC1/ADC2, not to this ADC4.
            #[cfg(adc_oversampler_1024)]
            assert!(o.ratio.log2() <= 8, "this ADC oversamples by at most 256");
            assert!(o.shift <= 8, "oversampling shift must be at most 8");
            w.set_ovsr(Ovsr::from_bits(o.ratio.log2() - 1));
            w.set_ovss(o.shift);
            w.set_tovs(o.triggered);
            w.set_ovse(true);
        }
    });
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

// ---------------------------------------------------------------------------------------------
// Family-specific extras on the driver.

impl<'d, T: super::Instance<Regs = Regs>, M: crate::mode::Mode> super::Adc<'d, T, M> {
    /// Enable or disable auto-off mode: the ADC powers itself off after each conversion and
    /// wakes up on the next start, at the cost of extra latency.
    pub fn set_auto_off(&mut self, enable: bool) {
        #[cfg(not(any(adc_v2_u5, adc_v2_wba)))]
        T::regs().cfgr1().modify(|w| w.set_autoff(enable));
        #[cfg(any(adc_v2_u5, adc_v2_wba))]
        T::regs().pwrr().modify(|w| w.set_autoff(enable));
    }
}
