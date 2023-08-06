use stm32_metapac::rtc::vals::{Init, Osel, Pol};

use super::{sealed, Instance, RtcClockSource, RtcConfig};
use crate::pac::rtc::Rtc;

impl<'d, T: Instance> super::Rtc<'d, T> {
    pub(super) fn enable(clock_source: RtcClockSource) {
        #[cfg(not(rtc_v2wb))]
        use stm32_metapac::rcc::vals::Rtcsel;

        #[cfg(any(rtc_v2f2, rtc_v2f3, rtc_v2l1))]
        let cr = crate::pac::PWR.cr();
        #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb))]
        let cr = crate::pac::PWR.cr1();

        // TODO: Missing from PAC for l0 and f0?
        #[cfg(not(any(rtc_v2f0, rtc_v2l0)))]
        {
            cr.modify(|w| w.set_dbp(true));
            while !cr.read().dbp() {}
        }

        #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
        let reg = crate::pac::RCC.bdcr().read();
        #[cfg(any(rtc_v2l0, rtc_v2l1))]
        let reg = crate::pac::RCC.csr().read();

        #[cfg(any(rtc_v2h7, rtc_v2l4, rtc_v2wb))]
        assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

        #[cfg(rtc_v2wb)]
        let rtcsel = reg.rtcsel();
        #[cfg(not(rtc_v2wb))]
        let rtcsel = reg.rtcsel().to_bits();

        if !reg.rtcen() || rtcsel != clock_source as u8 {
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

                // Select RTC source
                #[cfg(not(rtc_v2wb))]
                w.set_rtcsel(Rtcsel::from_bits(clock_source as u8));
                #[cfg(rtc_v2wb)]
                w.set_rtcsel(clock_source as u8);
                w.set_rtcen(true);

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

    pub(super) fn write<F, R>(&mut self, init_mode: bool, f: F) -> R
    where
        F: FnOnce(&crate::pac::rtc::Rtc) -> R,
    {
        let r = T::regs();
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

impl Instance for crate::peripherals::RTC {}
