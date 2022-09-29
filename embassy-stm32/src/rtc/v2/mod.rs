use stm32_metapac::rtc::vals::{Init, Osel, Pol};

use super::{Instance, RtcConfig};
use crate::pac::rtc::Rtc;

#[cfg_attr(rtc_v2f0, path = "v2f0.rs")]
#[cfg_attr(rtc_v2f2, path = "v2f2.rs")]
#[cfg_attr(rtc_v2f3, path = "v2f3.rs")]
#[cfg_attr(rtc_v2f4, path = "v2f4.rs")]
#[cfg_attr(rtc_v2f7, path = "v2f7.rs")]
#[cfg_attr(rtc_v2h7, path = "v2h7.rs")]
#[cfg_attr(rtc_v2l0, path = "v2l0.rs")]
#[cfg_attr(rtc_v2l1, path = "v2l1.rs")]
#[cfg_attr(rtc_v2l4, path = "v2l4.rs")]
#[cfg_attr(rtc_v2wb, path = "v2wb.rs")]
mod family;

pub use family::*;

impl<'d, T: Instance> super::Rtc<'d, T> {
    /// Applies the RTC config
    /// It this changes the RTC clock source the time will be reset
    pub(super) fn apply_config(&mut self, rtc_config: RtcConfig) {
        // Unlock the backup domain
        unsafe {
            unlock_backup_domain(rtc_config.clock_config as u8);
        }

        self.write(true, |rtc| unsafe {
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

        self.rtc_config = rtc_config;
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
            unsafe {
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
            }
        })
    }

    pub(super) fn write<F, R>(&mut self, init_mode: bool, f: F) -> R
    where
        F: FnOnce(&crate::pac::rtc::Rtc) -> R,
    {
        let r = T::regs();
        // Disable write protection.
        // This is safe, as we're only writin the correct and expected values.
        unsafe {
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
        }

        let result = f(&r);

        unsafe {
            if init_mode {
                r.isr().modify(|w| w.set_init(Init::FREERUNNINGMODE)); // Exits init mode
            }

            // Re-enable write protection.
            // This is safe, as the field accepts the full range of 8-bit values.
            r.wpr().write(|w| w.set_key(0xff));
        }
        result
    }
}

/// Read content of the backup register.
///
/// The registers retain their values during wakes from standby mode or system resets. They also
/// retain their value when Vdd is switched off as long as V_BAT is powered.
pub fn read_backup_register(rtc: &Rtc, register: usize) -> Option<u32> {
    if register < BACKUP_REGISTER_COUNT {
        Some(unsafe { rtc.bkpr(register).read().bkp() })
    } else {
        None
    }
}

/// Set content of the backup register.
///
/// The registers retain their values during wakes from standby mode or system resets. They also
/// retain their value when Vdd is switched off as long as V_BAT is powered.
pub fn write_backup_register(rtc: &Rtc, register: usize, value: u32) {
    if register < BACKUP_REGISTER_COUNT {
        unsafe { rtc.bkpr(register).write(|w| w.set_bkp(value)) }
    }
}
