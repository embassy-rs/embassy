use defmt::Format;
use stm32_metapac::rtc::vals::{Init, Osel, Pol};

use super::{sealed, RtcClockSource, RtcConfig};
use crate::pac::rtc::Rtc;
use crate::peripherals::RTC;
use crate::rtc::sealed::Instance;

#[cfg(all(feature = "time", any(stm32wb, stm32f4)))]
pub struct RtcInstant {
    ssr: u16,
    st: u8,
}

#[cfg(all(feature = "time", any(stm32wb, stm32f4)))]
impl RtcInstant {
    pub fn now() -> Self {
        // TODO: read value twice
        use crate::rtc::bcd2_to_byte;

        let tr = RTC::regs().tr().read();
        let tr2 = RTC::regs().tr().read();
        let ssr = RTC::regs().ssr().read().ss();
        let ssr2 = RTC::regs().ssr().read().ss();

        let st = bcd2_to_byte((tr.st(), tr.su()));
        let st2 = bcd2_to_byte((tr2.st(), tr2.su()));

        assert!(st == st2);
        assert!(ssr == ssr2);

        let _ = RTC::regs().dr().read();

        trace!("ssr: {}", ssr);
        trace!("st: {}", st);

        Self { ssr, st }
    }
}

#[cfg(all(feature = "time", any(stm32wb, stm32f4)))]
impl core::ops::Sub for RtcInstant {
    type Output = embassy_time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        use embassy_time::{Duration, TICK_HZ};

        trace!("self st: {}", self.st);
        trace!("other st: {}", rhs.st);

        trace!("self ssr: {}", self.ssr);
        trace!("other ssr: {}", rhs.ssr);

        let st = if self.st < rhs.st { self.st + 60 } else { self.st };

        trace!("self st: {}", st);

        let self_ticks = st as u32 * 256 + (255 - self.ssr as u32);
        let other_ticks = rhs.st as u32 * 256 + (255 - rhs.ssr as u32);
        let rtc_ticks = self_ticks - other_ticks;

        trace!("self ticks: {}", self_ticks);
        trace!("other ticks: {}", other_ticks);
        trace!("rtc ticks: {}", rtc_ticks);

        // TODO: read prescaler

        Duration::from_ticks(
            ((((st as u32 * 256 + (255u32 - self.ssr as u32)) - (rhs.st as u32 * 256 + (255u32 - rhs.ssr as u32)))
                * TICK_HZ as u32) as u32
                / 256u32) as u64,
        )
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Format)]
pub(crate) enum WakeupPrescaler {
    Div2,
    Div4,
    Div8,
    Div16,
}

#[cfg(any(stm32wb, stm32f4))]
impl From<WakeupPrescaler> for crate::pac::rtc::vals::Wucksel {
    fn from(val: WakeupPrescaler) -> Self {
        use crate::pac::rtc::vals::Wucksel;

        match val {
            WakeupPrescaler::Div2 => Wucksel::DIV2,
            WakeupPrescaler::Div4 => Wucksel::DIV4,
            WakeupPrescaler::Div8 => Wucksel::DIV8,
            WakeupPrescaler::Div16 => Wucksel::DIV16,
        }
    }
}

#[cfg(any(stm32wb, stm32f4))]
impl From<crate::pac::rtc::vals::Wucksel> for WakeupPrescaler {
    fn from(val: crate::pac::rtc::vals::Wucksel) -> Self {
        use crate::pac::rtc::vals::Wucksel;

        match val {
            Wucksel::DIV2 => WakeupPrescaler::Div2,
            Wucksel::DIV4 => WakeupPrescaler::Div4,
            Wucksel::DIV8 => WakeupPrescaler::Div8,
            Wucksel::DIV16 => WakeupPrescaler::Div16,
            _ => unreachable!(),
        }
    }
}

impl From<WakeupPrescaler> for u32 {
    fn from(val: WakeupPrescaler) -> Self {
        match val {
            WakeupPrescaler::Div2 => 2,
            WakeupPrescaler::Div4 => 4,
            WakeupPrescaler::Div8 => 8,
            WakeupPrescaler::Div16 => 16,
        }
    }
}

#[allow(dead_code)]
impl WakeupPrescaler {
    pub fn compute_min(val: u32) -> Self {
        *[
            WakeupPrescaler::Div2,
            WakeupPrescaler::Div4,
            WakeupPrescaler::Div8,
            WakeupPrescaler::Div16,
        ]
        .iter()
        .skip_while(|psc| <WakeupPrescaler as Into<u32>>::into(**psc) <= val)
        .next()
        .unwrap_or(&WakeupPrescaler::Div16)
    }
}

impl super::Rtc {
    fn unlock_registers() {
        #[cfg(any(rtc_v2f2, rtc_v2f3, rtc_v2l1))]
        let cr = crate::pac::PWR.cr();
        #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb))]
        let cr = crate::pac::PWR.cr1();

        // TODO: Missing from PAC for l0 and f0?
        #[cfg(not(any(rtc_v2f0, rtc_v2l0)))]
        {
            if !cr.read().dbp() {
                cr.modify(|w| w.set_dbp(true));
                while !cr.read().dbp() {}
            }
        }
    }

    #[allow(dead_code)]
    #[cfg(all(feature = "time", any(stm32wb, stm32f4)))]
    /// start the wakeup alarm and return the actual duration of the alarm
    /// the actual duration will be the closest value possible that is less
    /// than the requested duration.
    ///
    /// note: this api is exposed for testing purposes until low power is implemented.
    /// it is not intended to be public
    pub(crate) fn start_wakeup_alarm(&self, requested_duration: embassy_time::Duration) -> RtcInstant {
        use embassy_time::{Duration, TICK_HZ};

        use crate::rcc::get_freqs;

        let rtc_hz = unsafe { get_freqs() }.rtc.unwrap().0 as u64;

        let rtc_ticks = requested_duration.as_ticks() * rtc_hz / TICK_HZ;
        let prescaler = WakeupPrescaler::compute_min((rtc_ticks / u16::MAX as u64) as u32);

        // adjust the rtc ticks to the prescaler
        let rtc_ticks = rtc_ticks / (<WakeupPrescaler as Into<u32>>::into(prescaler) as u64);
        let rtc_ticks = if rtc_ticks >= u16::MAX as u64 {
            u16::MAX - 1
        } else {
            rtc_ticks as u16
        };

        let duration = Duration::from_ticks(
            rtc_ticks as u64 * TICK_HZ * (<WakeupPrescaler as Into<u32>>::into(prescaler) as u64) / rtc_hz,
        );

        trace!("set wakeup timer for {} ms", duration.as_millis());
        trace!("set wakeup timer for {} ticks with pre {}", rtc_ticks, prescaler);

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wutie(true));

            trace!("clear wute");
            regs.cr().modify(|w| w.set_wute(false));
            regs.isr().modify(|w| w.set_wutf(false));

            trace!("wait for wutwf...");
            while !regs.isr().read().wutwf() {}
            trace!("wait for wutwf...done");

            regs.cr().modify(|w| {
                w.set_wucksel(prescaler.into());

                w.set_wutie(true);
            });

            regs.cr().modify(|w| w.set_wute(true));
        });

        if !RTC::regs().cr().read().wute() {
            trace!("wakeup timer not enabled");
        } else {
            trace!("wakeup timer enabled");
        }

        if !RTC::regs().cr().read().wutie() {
            trace!("wakeup timer interrupt not enabled");
        } else {
            trace!("wakeup timer interrupt enabled");
        }

        RtcInstant::now()
    }

    #[allow(dead_code)]
    #[cfg(all(feature = "time", any(stm32wb, stm32f4)))]
    /// stop the wakeup alarm and return the time remaining
    ///
    /// note: this api is exposed for testing purposes until low power is implemented.
    /// it is not intended to be public
    pub(crate) fn stop_wakeup_alarm(&self) -> RtcInstant {
        trace!("disable wakeup timer...");

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wute(false));
            regs.isr().modify(|w| w.set_wutf(false));
        });

        RtcInstant::now()
    }

    #[allow(dead_code)]
    pub(crate) fn set_clock_source(clock_source: RtcClockSource) {
        #[cfg(not(rtc_v2wb))]
        use stm32_metapac::rcc::vals::Rtcsel;

        #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
        let cr = crate::pac::RCC.bdcr();
        #[cfg(any(rtc_v2l0, rtc_v2l1))]
        let cr = crate::pac::RCC.csr();

        Self::unlock_registers();

        cr.modify(|w| {
            // Select RTC source
            #[cfg(not(rtc_v2wb))]
            w.set_rtcsel(Rtcsel::from_bits(clock_source as u8));
            #[cfg(rtc_v2wb)]
            w.set_rtcsel(clock_source as u8);
        });
    }

    pub(super) fn enable() {
        #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
        let reg = crate::pac::RCC.bdcr().read();
        #[cfg(any(rtc_v2l0, rtc_v2l1))]
        let reg = crate::pac::RCC.csr().read();

        #[cfg(any(rtc_v2h7, rtc_v2l4, rtc_v2wb))]
        assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

        if !reg.rtcen() {
            Self::unlock_registers();

            #[cfg(not(any(rtc_v2l0, rtc_v2l1, rtc_v2f2)))]
            crate::pac::RCC.bdcr().modify(|w| w.set_bdrst(true));
            #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
            let cr = crate::pac::RCC.bdcr();
            #[cfg(any(rtc_v2l0, rtc_v2l1))]
            let cr = crate::pac::RCC.csr();

            cr.modify(|w| {
                // Reset
                #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
                w.set_bdrst(false);

                w.set_rtcen(true);
                w.set_rtcsel(reg.rtcsel());

                // Restore bcdr
                #[cfg(any(rtc_v2l4, rtc_v2wb))]
                w.set_lscosel(reg.lscosel());
                #[cfg(any(rtc_v2l4, rtc_v2wb))]
                w.set_lscoen(reg.lscoen());

                w.set_lseon(reg.lseon());

                #[cfg(any(rtc_v2f0, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb))]
                w.set_lsedrv(reg.lsedrv());
                w.set_lsebyp(reg.lsebyp());
            });
        }
    }

    /// Applies the RTC config
    /// It this changes the RTC clock source the time will be reset
    pub(super) fn configure(&mut self, rtc_config: RtcConfig) {
        self.write(true, |rtc| {
            rtc.cr().modify(|w| {
                #[cfg(rtc_v2f2)]
                w.set_fmt(false);
                #[cfg(not(rtc_v2f2))]
                w.set_fmt(stm32_metapac::rtc::vals::Fmt::TWENTY_FOUR_HOUR);
                w.set_osel(Osel::DISABLED);
                w.set_pol(Pol::HIGH);
            });

            rtc.prer().modify(|w| {
                w.set_prediv_s(rtc_config.sync_prescaler);
                w.set_prediv_a(rtc_config.async_prescaler);
            });
        });
    }

    /// Calibrate the clock drift.
    ///
    /// `clock_drift` can be adjusted from -487.1 ppm to 488.5 ppm and is clamped to this range.
    ///
    /// ### Note
    ///
    /// To perform a calibration when `async_prescaler` is less then 3, `sync_prescaler`
    /// has to be reduced accordingly (see RM0351 Rev 9, sec 38.3.12).
    #[cfg(not(rtc_v2f2))]
    pub fn calibrate(&mut self, mut clock_drift: f32, period: super::RtcCalibrationCyclePeriod) {
        const RTC_CALR_MIN_PPM: f32 = -487.1;
        const RTC_CALR_MAX_PPM: f32 = 488.5;
        const RTC_CALR_RESOLUTION_PPM: f32 = 0.9537;

        if clock_drift < RTC_CALR_MIN_PPM {
            clock_drift = RTC_CALR_MIN_PPM;
        } else if clock_drift > RTC_CALR_MAX_PPM {
            clock_drift = RTC_CALR_MAX_PPM;
        }

        clock_drift = clock_drift / RTC_CALR_RESOLUTION_PPM;

        self.write(false, |rtc| {
            rtc.calr().write(|w| {
                match period {
                    super::RtcCalibrationCyclePeriod::Seconds8 => {
                        w.set_calw8(stm32_metapac::rtc::vals::Calw8::EIGHT_SECOND);
                    }
                    super::RtcCalibrationCyclePeriod::Seconds16 => {
                        w.set_calw16(stm32_metapac::rtc::vals::Calw16::SIXTEEN_SECOND);
                    }
                    super::RtcCalibrationCyclePeriod::Seconds32 => {
                        // Set neither `calw8` nor `calw16` to use 32 seconds
                    }
                }

                // Extra pulses during calibration cycle period: CALP * 512 - CALM
                //
                // CALP sets whether pulses are added or omitted.
                //
                // CALM contains how many pulses (out of 512) are masked in a
                // given calibration cycle period.
                if clock_drift > 0.0 {
                    // Maximum (about 512.2) rounds to 512.
                    clock_drift += 0.5;

                    // When the offset is positive (0 to 512), the opposite of
                    // the offset (512 - offset) is masked, i.e. for the
                    // maximum offset (512), 0 pulses are masked.
                    w.set_calp(stm32_metapac::rtc::vals::Calp::INCREASEFREQ);
                    w.set_calm(512 - clock_drift as u16);
                } else {
                    // Minimum (about -510.7) rounds to -511.
                    clock_drift -= 0.5;

                    // When the offset is negative or zero (-511 to 0),
                    // the absolute offset is masked, i.e. for the minimum
                    // offset (-511), 511 pulses are masked.
                    w.set_calp(stm32_metapac::rtc::vals::Calp::NOCHANGE);
                    w.set_calm((clock_drift * -1.0) as u16);
                }
            });
        })
    }

    pub(super) fn write<F, R>(&self, init_mode: bool, f: F) -> R
    where
        F: FnOnce(&crate::pac::rtc::Rtc) -> R,
    {
        let r = RTC::regs();
        // Disable write protection.
        // This is safe, as we're only writin the correct and expected values.
        r.wpr().write(|w| w.set_key(0xca));
        r.wpr().write(|w| w.set_key(0x53));

        // true if initf bit indicates RTC peripheral is in init mode
        if init_mode && !r.isr().read().initf() {
            // to update calendar date/time, time format, and prescaler configuration, RTC must be in init mode
            r.isr().modify(|w| w.set_init(Init::INITMODE));
            // wait till init state entered
            // ~2 RTCCLK cycles
            while !r.isr().read().initf() {}
        }

        let result = f(&r);

        if init_mode {
            r.isr().modify(|w| w.set_init(Init::FREERUNNINGMODE)); // Exits init mode
        }

        // Re-enable write protection.
        // This is safe, as the field accepts the full range of 8-bit values.
        r.wpr().write(|w| w.set_key(0xff));
        result
    }
}

impl sealed::Instance for crate::peripherals::RTC {
    const BACKUP_REGISTER_COUNT: usize = 20;

    fn enable_peripheral_clk() {
        #[cfg(any(rtc_v2l4, rtc_v2wb))]
        {
            // enable peripheral clock for communication
            crate::pac::RCC.apb1enr1().modify(|w| w.set_rtcapben(true));

            // read to allow the pwr clock to enable
            crate::pac::PWR.cr1().read();
        }
        #[cfg(any(rtc_v2f2))]
        {
            crate::pac::RCC.apb1enr().modify(|w| w.set_pwren(true));
            crate::pac::PWR.cr().read();
        }
    }

    fn read_backup_register(rtc: &Rtc, register: usize) -> Option<u32> {
        if register < Self::BACKUP_REGISTER_COUNT {
            Some(rtc.bkpr(register).read().bkp())
        } else {
            None
        }
    }

    fn write_backup_register(rtc: &Rtc, register: usize, value: u32) {
        if register < Self::BACKUP_REGISTER_COUNT {
            rtc.bkpr(register).write(|w| w.set_bkp(value));
        }
    }
}
