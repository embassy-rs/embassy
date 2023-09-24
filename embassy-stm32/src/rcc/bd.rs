#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
pub enum LseDrive {
    #[cfg(any(rtc_v2f7, rtc_v2l4))]
    Low = 0,
    MediumLow = 0x01,
    #[default]
    MediumHigh = 0x02,
    #[cfg(any(rtc_v2f7, rtc_v2l4))]
    High = 0x03,
}

#[cfg(any(rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l4))]
impl From<LseDrive> for crate::pac::rcc::vals::Lsedrv {
    fn from(value: LseDrive) -> Self {
        use crate::pac::rcc::vals::Lsedrv;

        match value {
            #[cfg(any(rtc_v2f7, rtc_v2l4))]
            LseDrive::Low => Lsedrv::LOW,
            LseDrive::MediumLow => Lsedrv::MEDIUMLOW,
            LseDrive::MediumHigh => Lsedrv::MEDIUMHIGH,
            #[cfg(any(rtc_v2f7, rtc_v2l4))]
            LseDrive::High => Lsedrv::HIGH,
        }
    }
}

pub use crate::pac::rcc::vals::Rtcsel as RtcClockSource;

#[cfg(not(any(rtc_v2l0, rtc_v2l1, stm32c0)))]
#[allow(dead_code)]
type Bdcr = crate::pac::rcc::regs::Bdcr;

#[cfg(any(rtc_v2l0, rtc_v2l1))]
#[allow(dead_code)]
type Bdcr = crate::pac::rcc::regs::Csr;

#[allow(dead_code)]
pub struct BackupDomain {}

impl BackupDomain {
    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb, rtc_v3,
        rtc_v3u5
    ))]
    #[allow(dead_code, unused_variables)]
    fn modify<R>(f: impl FnOnce(&mut Bdcr) -> R) -> R {
        #[cfg(any(rtc_v2f2, rtc_v2f3, rtc_v2l1, rtc_v2l0))]
        let cr = crate::pac::PWR.cr();
        #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
        let cr = crate::pac::PWR.cr1();

        // TODO: Missing from PAC for l0 and f0?
        #[cfg(not(any(rtc_v2f0, rtc_v3u5)))]
        {
            cr.modify(|w| w.set_dbp(true));
            while !cr.read().dbp() {}
        }

        #[cfg(any(rtc_v2l0, rtc_v2l1))]
        let cr = crate::pac::RCC.csr();

        #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
        let cr = crate::pac::RCC.bdcr();

        cr.modify(|w| f(w))
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb, rtc_v3,
        rtc_v3u5
    ))]
    #[allow(dead_code)]
    fn read() -> Bdcr {
        #[cfg(any(rtc_v2l0, rtc_v2l1))]
        let r = crate::pac::RCC.csr().read();

        #[cfg(not(any(rtc_v2l0, rtc_v2l1)))]
        let r = crate::pac::RCC.bdcr().read();

        r
    }

    #[allow(dead_code, unused_variables)]
    fn enable_peripheral_clk() {
        #[cfg(any(rtc_v2l4, rtc_v2wb))]
        {
            crate::pac::RCC.apb1enr1().modify(|w| w.set_rtcapben(true));
            crate::pac::PWR.cr1().read();
        }
        #[cfg(any(rtc_v2f2))]
        {
            crate::pac::RCC.apb1enr().modify(|w| w.set_pwren(true));
            crate::pac::PWR.cr().read();
        }

        #[cfg(any(rtc_v2f0, rtc_v2l0))]
        crate::pac::RCC.apb1enr().modify(|w| w.set_pwren(true));

        #[cfg(any(rcc_wle, rcc_wl5, rcc_g4))]
        crate::pac::RCC.apb1enr1().modify(|w| w.set_rtcapben(true));

        #[cfg(rcc_g0)]
        crate::pac::RCC.apbenr1().modify(|w| w.set_rtcapben(true));

        #[cfg(any(rtc_v3, rtc_v3u5))]
        crate::pac::PWR.cr1().read();
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb, rtc_v3,
        rtc_v3u5
    ))]
    #[allow(dead_code, unused_variables)]
    pub fn configure_ls(clock_source: RtcClockSource, lsi: bool, lse: Option<LseDrive>) {
        if lsi || lse.is_some() {
            Self::enable_peripheral_clk();
        }

        if lsi {
            #[cfg(rtc_v3u5)]
            let csr = crate::pac::RCC.bdcr();

            #[cfg(not(rtc_v3u5))]
            let csr = crate::pac::RCC.csr();

            Self::modify(|_| {
                #[cfg(not(any(rcc_wb, rcc_wba)))]
                csr.modify(|w| w.set_lsion(true));

                #[cfg(any(rcc_wb, rcc_wba))]
                csr.modify(|w| w.set_lsi1on(true));
            });

            #[cfg(not(any(rcc_wb, rcc_wba)))]
            while !csr.read().lsirdy() {}

            #[cfg(any(rcc_wb, rcc_wba))]
            while !csr.read().lsi1rdy() {}
        }

        if let Some(lse_drive) = lse {
            Self::modify(|w| {
                #[cfg(any(rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l4))]
                w.set_lsedrv(lse_drive.into());
                w.set_lseon(true);
            });

            while !Self::read().lserdy() {}
        }

        match clock_source {
            RtcClockSource::LSI => assert!(lsi),
            RtcClockSource::LSE => assert!(&lse.is_some()),
            _ => {}
        };

        if clock_source == RtcClockSource::NOCLOCK {
            // disable it
            Self::modify(|w| {
                #[cfg(not(rcc_wba))]
                w.set_rtcen(false);
                w.set_rtcsel(clock_source);
            });
        } else {
            // check if it's already enabled and in the source we want.
            let reg = Self::read();
            let ok = reg.rtcsel() == clock_source;
            #[cfg(not(rcc_wba))]
            let ok = ok & reg.rtcen();

            // if not, configure it.
            if !ok {
                #[cfg(any(rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
                assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

                #[cfg(not(any(rcc_l0, rcc_l1)))]
                Self::modify(|w| w.set_bdrst(true));

                Self::modify(|w| {
                    // Reset
                    #[cfg(not(any(rcc_l0, rcc_l1)))]
                    w.set_bdrst(false);

                    #[cfg(not(rcc_wba))]
                    w.set_rtcen(true);
                    w.set_rtcsel(clock_source);

                    // Restore bcdr
                    #[cfg(any(rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
                    w.set_lscosel(reg.lscosel());
                    #[cfg(any(rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
                    w.set_lscoen(reg.lscoen());

                    w.set_lseon(reg.lseon());

                    #[cfg(any(rtc_v2f0, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
                    w.set_lsedrv(reg.lsedrv());
                    w.set_lsebyp(reg.lsebyp());
                });
            }
        }
    }
}
