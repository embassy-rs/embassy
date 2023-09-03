use stm32_metapac::rtc::vals::{Init, Osel, Pol};

use super::sealed;
use crate::pac::rtc::Rtc;
use crate::peripherals::RTC;
use crate::rtc::sealed::Instance;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum WakeupPrescaler {
    Div2 = 2,
    Div4 = 4,
    Div8 = 8,
    Div16 = 16,
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
        .skip_while(|psc| **psc as u32 <= val)
        .next()
        .unwrap_or(&WakeupPrescaler::Div16)
    }
}

impl super::Rtc {
    #[cfg(feature = "low-power")]
    /// start the wakeup alarm and wtih a duration that is as close to but less than
    /// the requested duration, and record the instant the wakeup alarm was started
    pub(crate) fn start_wakeup_alarm(&self, requested_duration: embassy_time::Duration) {
        use embassy_time::{Duration, TICK_HZ};

        use crate::rcc::get_freqs;

        let rtc_hz = unsafe { get_freqs() }.rtc.unwrap().0 as u64;

        let rtc_ticks = requested_duration.as_ticks() * rtc_hz / TICK_HZ;
        let prescaler = WakeupPrescaler::compute_min((rtc_ticks / u16::MAX as u64) as u32);

        // adjust the rtc ticks to the prescaler and subtract one rtc tick
        let rtc_ticks = rtc_ticks / prescaler as u64;
        let rtc_ticks = if rtc_ticks >= u16::MAX as u64 {
            u16::MAX - 1
        } else {
            rtc_ticks as u16
        }
        .saturating_sub(1);

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wute(false));
            regs.isr().modify(|w| w.set_wutf(false));
            while !regs.isr().read().wutwf() {}

            regs.cr().modify(|w| w.set_wucksel(prescaler.into()));
            regs.wutr().write(|w| w.set_wut(rtc_ticks));
            regs.cr().modify(|w| w.set_wute(true));
            regs.cr().modify(|w| w.set_wutie(true));
        });

        trace!(
            "rtc: start wakeup alarm for {} ms (psc: {}, ticks: {}) at {}",
            Duration::from_ticks(rtc_ticks as u64 * TICK_HZ * prescaler as u64 / rtc_hz).as_millis(),
            prescaler as u32,
            rtc_ticks,
            self.instant(),
        );

        critical_section::with(|cs| assert!(self.stop_time.borrow(cs).replace(Some(self.instant())).is_none()))
    }

    #[cfg(feature = "low-power")]
    pub(crate) fn enable_wakeup_line(&self) {
        use crate::pac::EXTI;

        EXTI.rtsr(0).modify(|w| w.set_line(22, true));
        EXTI.imr(0).modify(|w| w.set_line(22, true));
    }

    #[cfg(feature = "low-power")]
    /// stop the wakeup alarm and return the time elapsed since `start_wakeup_alarm`
    /// was called, otherwise none
    pub(crate) fn stop_wakeup_alarm(&self) -> Option<embassy_time::Duration> {
        use crate::interrupt::typelevel::Interrupt;

        trace!("rtc: stop wakeup alarm at {}", self.instant());

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wutie(false));
            regs.cr().modify(|w| w.set_wute(false));
            regs.isr().modify(|w| w.set_wutf(false));

            crate::pac::EXTI.pr(0).modify(|w| w.set_line(22, true));
            crate::interrupt::typelevel::RTC_WKUP::unpend();
        });

        critical_section::with(|cs| {
            if let Some(stop_time) = self.stop_time.borrow(cs).take() {
                Some(self.instant() - stop_time)
            } else {
                None
            }
        })
    }

    /// Applies the RTC config
    /// It this changes the RTC clock source the time will be reset
    pub(super) fn configure(&mut self, async_psc: u8, sync_psc: u16) {
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
                w.set_prediv_s(sync_psc);
                w.set_prediv_a(async_psc);
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
