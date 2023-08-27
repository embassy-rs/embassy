#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum RtcClockSource {
    /// 00: No clock
    NoClock = 0b00,
    /// 01: LSE oscillator clock used as RTC clock
    LSE = 0b01,
    /// 10: LSI oscillator clock used as RTC clock
    LSI = 0b10,
    /// 11: HSE oscillator clock divided by 32 used as RTC clock
    HSE = 0b11,
}

#[allow(dead_code)]
pub struct BackupDomain {}

impl BackupDomain {
    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ))]
    #[allow(dead_code)]
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

    #[cfg(any(rtc_v3, rtc_v3u5))]
    #[allow(dead_code)]
    fn unlock_registers() {
        // Unlock the backup domain
        #[cfg(not(any(rtc_v3u5, rcc_wl5, rcc_wle)))]
        {
            if !crate::pac::PWR.cr1().read().dbp() {
                crate::pac::PWR.cr1().modify(|w| w.set_dbp(true));
                while !crate::pac::PWR.cr1().read().dbp() {}
            }
        }
        #[cfg(any(rcc_wl5, rcc_wle))]
        {
            use crate::pac::pwr::vals::Dbp;

            if crate::pac::PWR.cr1().read().dbp() != Dbp::ENABLED {
                crate::pac::PWR.cr1().modify(|w| w.set_dbp(Dbp::ENABLED));
                while crate::pac::PWR.cr1().read().dbp() != Dbp::ENABLED {}
            }
        }
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ))]
    #[allow(dead_code)]
    pub fn set_rtc_clock_source(clock_source: RtcClockSource) {
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

    #[cfg(any(rtc_v3, rtc_v3u5))]
    #[allow(dead_code)]
    pub fn set_rtc_clock_source(clock_source: RtcClockSource) {
        let clock_source = clock_source as u8;
        #[cfg(not(any(rcc_wl5, rcc_wle)))]
        let clock_source = crate::pac::rcc::vals::Rtcsel::from_bits(clock_source);

        Self::unlock_registers();

        crate::pac::RCC.bdcr().modify(|w| {
            // Select RTC source
            w.set_rtcsel(clock_source);
        });
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ))]
    #[allow(dead_code)]
    pub fn enable_rtc() {
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

    #[cfg(any(rtc_v3, rtc_v3u5))]
    #[allow(dead_code)]
    pub fn enable_rtc() {
        let bdcr = crate::pac::RCC.bdcr();

        let reg = bdcr.read();
        assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

        if !reg.rtcen() {
            Self::unlock_registers();

            bdcr.modify(|w| w.set_bdrst(true));

            bdcr.modify(|w| {
                // Reset
                w.set_bdrst(false);

                w.set_rtcen(true);
                w.set_rtcsel(reg.rtcsel());

                // Restore bcdr
                w.set_lscosel(reg.lscosel());
                w.set_lscoen(reg.lscoen());

                w.set_lseon(reg.lseon());
                w.set_lsedrv(reg.lsedrv());
                w.set_lsebyp(reg.lsebyp());
            });
        }
    }
}
