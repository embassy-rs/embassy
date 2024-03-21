use stm32_metapac::rtc::vals::{Calp, Calw16, Calw8, Fmt, Key, Osel, Pol, TampalrmType};

use super::{sealed, RtcCalibrationCyclePeriod};
use crate::pac::rtc::Rtc;
use crate::peripherals::RTC;
use crate::rtc::sealed::Instance;

impl super::Rtc {
    /// Applies the RTC config
    /// It this changes the RTC clock source the time will be reset
    pub(super) fn configure(&mut self, async_psc: u8, sync_psc: u16) {
        self.write(true, |rtc| {
            rtc.cr().modify(|w| {
                w.set_bypshad(true);
                w.set_fmt(Fmt::TWENTYFOURHOUR);
                w.set_osel(Osel::DISABLED);
                w.set_pol(Pol::HIGH);
            });

            rtc.prer().modify(|w| {
                w.set_prediv_s(sync_psc);
                w.set_prediv_a(async_psc);
            });

            // TODO: configuration for output pins
            rtc.cr().modify(|w| {
                w.set_out2en(false);
                w.set_tampalrm_type(TampalrmType::PUSHPULL);
                w.set_tampalrm_pu(false);
            });
        });
    }

    const RTC_CALR_MIN_PPM: f32 = -487.1;
    const RTC_CALR_MAX_PPM: f32 = 488.5;
    const RTC_CALR_RESOLUTION_PPM: f32 = 0.9537;

    /// Calibrate the clock drift.
    ///
    /// `clock_drift` can be adjusted from -487.1 ppm to 488.5 ppm and is clamped to this range.
    ///
    /// ### Note
    ///
    /// To perform a calibration when `async_prescaler` is less then 3, `sync_prescaler`
    /// has to be reduced accordingly (see RM0351 Rev 9, sec 38.3.12).
    pub fn calibrate(&mut self, mut clock_drift: f32, period: RtcCalibrationCyclePeriod) {
        if clock_drift < Self::RTC_CALR_MIN_PPM {
            clock_drift = Self::RTC_CALR_MIN_PPM;
        } else if clock_drift > Self::RTC_CALR_MAX_PPM {
            clock_drift = Self::RTC_CALR_MAX_PPM;
        }

        clock_drift = clock_drift / Self::RTC_CALR_RESOLUTION_PPM;

        self.write(false, |rtc| {
            rtc.calr().write(|w| {
                match period {
                    RtcCalibrationCyclePeriod::Seconds8 => {
                        w.set_calw8(Calw8::EIGHTSECONDS);
                    }
                    RtcCalibrationCyclePeriod::Seconds16 => {
                        w.set_calw16(Calw16::SIXTEENSECONDS);
                    }
                    RtcCalibrationCyclePeriod::Seconds32 => {
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
                    w.set_calp(Calp::INCREASEFREQ);
                    w.set_calm(512 - clock_drift as u16);
                } else {
                    // Minimum (about -510.7) rounds to -511.
                    clock_drift -= 0.5;

                    // When the offset is negative or zero (-511 to 0),
                    // the absolute offset is masked, i.e. for the minimum
                    // offset (-511), 511 pulses are masked.
                    w.set_calp(Calp::NOCHANGE);
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
        r.wpr().write(|w| w.set_key(Key::DEACTIVATE1));
        r.wpr().write(|w| w.set_key(Key::DEACTIVATE2));

        if init_mode && !r.icsr().read().initf() {
            r.icsr().modify(|w| w.set_init(true));
            // wait till init state entered
            // ~2 RTCCLK cycles
            while !r.icsr().read().initf() {}
        }

        let result = f(&r);

        if init_mode {
            r.icsr().modify(|w| w.set_init(false)); // Exits init mode
        }

        // Re-enable write protection.
        // This is safe, as the field accepts the full range of 8-bit values.
        r.wpr().write(|w| w.set_key(Key::ACTIVATE));

        result
    }
}

impl sealed::Instance for crate::peripherals::RTC {
    const BACKUP_REGISTER_COUNT: usize = 32;

    #[cfg(all(feature = "low-power", stm32g4))]
    const EXTI_WAKEUP_LINE: usize = 20;

    #[cfg(all(feature = "low-power", stm32g0))]
    const EXTI_WAKEUP_LINE: usize = 19;

    #[cfg(all(feature = "low-power", stm32g0))]
    type WakeupInterrupt = crate::interrupt::typelevel::RTC_TAMP;

    #[cfg(all(feature = "low-power", stm32g4))]
    type WakeupInterrupt = crate::interrupt::typelevel::RTC_WKUP;

    #[cfg(all(feature = "low-power", stm32l5))]
    const EXTI_WAKEUP_LINE: usize = 17;

    #[cfg(all(feature = "low-power", stm32l5))]
    type WakeupInterrupt = crate::interrupt::typelevel::RTC;

    fn read_backup_register(_rtc: &Rtc, register: usize) -> Option<u32> {
        #[allow(clippy::if_same_then_else)]
        if register < Self::BACKUP_REGISTER_COUNT {
            //Some(rtc.bkpr()[register].read().bits())
            None // RTC3 backup registers come from the TAMP peripe=heral, not RTC. Not() even in the L412 PAC
        } else {
            None
        }
    }

    fn write_backup_register(_rtc: &Rtc, register: usize, _value: u32) {
        if register < Self::BACKUP_REGISTER_COUNT {
            // RTC3 backup registers come from the TAMP peripe=heral, not RTC. Not() even in the L412 PAC
            //self.rtc.bkpr()[register].write(|w| w.bits(value))
        }
    }
}
