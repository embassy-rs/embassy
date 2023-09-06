#[allow(dead_code)]
#[derive(Default)]
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

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
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
        #[cfg(any(rtc_v2f2, rtc_v2f3, rtc_v2l1))]
        let cr = crate::pac::PWR.cr();
        #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
        let cr = crate::pac::PWR.cr1();

        // TODO: Missing from PAC for l0 and f0?
        #[cfg(not(any(rtc_v2f0, rtc_v2l0, rtc_v3u5)))]
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
    #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb))]
    pub fn enable_lse(lse_drive: LseDrive) {
        Self::modify(|w| {
            #[cfg(any(rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l4))]
            w.set_lsedrv(lse_drive.into());
            w.set_lseon(true);
        });

        while !Self::read().lserdy() {}
    }

    #[allow(dead_code)]
    #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb))]
    pub fn enable_lsi() {
        let csr = crate::pac::RCC.csr();

        Self::modify(|_| {
            #[cfg(not(rtc_v2wb))]
            csr.modify(|w| w.set_lsion(true));

            #[cfg(rtc_v2wb)]
            csr.modify(|w| w.set_lsi1on(true));
        });

        #[cfg(not(rtc_v2wb))]
        while !csr.read().lsirdy() {}

        #[cfg(rtc_v2wb)]
        while !csr.read().lsi1rdy() {}
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb, rtc_v3,
        rtc_v3u5
    ))]
    #[allow(dead_code, unused_variables)]
    pub fn set_rtc_clock_source(clock_source: RtcClockSource) {
        let clock_source = clock_source as u8;
        #[cfg(any(
            not(any(rtc_v3, rtc_v3u5, rtc_v2wb)),
            all(any(rtc_v3, rtc_v3u5), not(any(rcc_wl5, rcc_wle)))
        ))]
        let clock_source = crate::pac::rcc::vals::Rtcsel::from_bits(clock_source);

        #[cfg(not(rtc_v2wb))]
        Self::modify(|w| {
            // Select RTC source
            w.set_rtcsel(clock_source);
        });
    }

    #[cfg(any(rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb))]
    #[allow(dead_code, unused_variables)]
    pub fn configure_rtc(clock_source: RtcClockSource, lse_drive: Option<LseDrive>) {
        match clock_source {
            RtcClockSource::LSI => Self::enable_lsi(),
            RtcClockSource::LSE => Self::enable_lse(lse_drive.unwrap_or_default()),
            _ => {}
        };

        Self::set_rtc_clock_source(clock_source);
    }

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb, rtc_v3,
        rtc_v3u5
    ))]
    #[allow(dead_code)]
    pub fn enable_rtc() {
        let reg = Self::read();

        #[cfg(any(rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
        assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

        if !reg.rtcen() {
            #[cfg(not(any(rtc_v2l0, rtc_v2l1, rtc_v2f2)))]
            Self::modify(|w| w.set_bdrst(true));

            Self::modify(|w| {
                // Reset
                #[cfg(not(any(rtc_v2l0, rtc_v2l1, rtc_v2f2)))]
                w.set_bdrst(false);

                w.set_rtcen(true);
                w.set_rtcsel(reg.rtcsel());

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
