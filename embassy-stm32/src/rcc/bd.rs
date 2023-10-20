use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac::common::{Reg, RW};
pub use crate::pac::rcc::vals::Rtcsel as RtcClockSource;
use crate::time::Hertz;

#[cfg(any(stm32f0, stm32f1, stm32f3))]
pub const LSI_FREQ: Hertz = Hertz(40_000);
#[cfg(not(any(stm32f0, stm32f1, stm32f3)))]
pub const LSI_FREQ: Hertz = Hertz(32_000);

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum LseMode {
    Oscillator(LseDrive),
    Bypass,
}

pub struct LseConfig {
    pub frequency: Hertz,
    pub mode: LseMode,
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
pub enum LseDrive {
    Low = 0,
    MediumLow = 0x01,
    #[default]
    MediumHigh = 0x02,
    High = 0x03,
}

// All families but these have the LSEDRV register
#[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f400, rcc_f410, rcc_l1)))]
impl From<LseDrive> for crate::pac::rcc::vals::Lsedrv {
    fn from(value: LseDrive) -> Self {
        use crate::pac::rcc::vals::Lsedrv;

        match value {
            LseDrive::Low => Lsedrv::LOW,
            LseDrive::MediumLow => Lsedrv::MEDIUMLOW,
            LseDrive::MediumHigh => Lsedrv::MEDIUMHIGH,
            LseDrive::High => Lsedrv::HIGH,
        }
    }
}

#[cfg(not(any(rtc_v2l0, rtc_v2l1, stm32c0)))]
type Bdcr = crate::pac::rcc::regs::Bdcr;
#[cfg(any(rtc_v2l0, rtc_v2l1))]
type Bdcr = crate::pac::rcc::regs::Csr;
#[cfg(any(stm32c0))]
type Bdcr = crate::pac::rcc::regs::Csr1;

#[cfg(any(stm32c0))]
fn unlock() {}

#[cfg(not(any(stm32c0)))]
fn unlock() {
    #[cfg(any(stm32f0, stm32f1, stm32f2, stm32f3, stm32l0, stm32l1))]
    let cr = crate::pac::PWR.cr();
    #[cfg(not(any(stm32f0, stm32f1, stm32f2, stm32f3, stm32l0, stm32l1, stm32u5, stm32h5, stm32wba)))]
    let cr = crate::pac::PWR.cr1();
    #[cfg(any(stm32u5, stm32h5, stm32wba))]
    let cr = crate::pac::PWR.dbpcr();

    cr.modify(|w| w.set_dbp(true));
    while !cr.read().dbp() {}
}

fn bdcr() -> Reg<Bdcr, RW> {
    #[cfg(any(rtc_v2l0, rtc_v2l1))]
    return crate::pac::RCC.csr();
    #[cfg(not(any(rtc_v2l0, rtc_v2l1, stm32c0)))]
    return crate::pac::RCC.bdcr();
    #[cfg(any(stm32c0))]
    return crate::pac::RCC.csr1();
}

pub struct LsConfig {
    pub rtc: RtcClockSource,
    pub lsi: bool,
    pub lse: Option<LseConfig>,
}

impl LsConfig {
    pub const fn default_lse() -> Self {
        Self {
            rtc: RtcClockSource::LSE,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumHigh),
            }),
            lsi: false,
        }
    }

    pub const fn default_lsi() -> Self {
        Self {
            rtc: RtcClockSource::LSI,
            lsi: true,
            lse: None,
        }
    }

    pub const fn off() -> Self {
        Self {
            rtc: RtcClockSource::DISABLE,
            lsi: false,
            lse: None,
        }
    }
}

impl Default for LsConfig {
    fn default() -> Self {
        // on L5, just the fact that LSI is enabled makes things crash.
        // TODO: investigate.

        #[cfg(not(stm32l5))]
        return Self::default_lsi();
        #[cfg(stm32l5)]
        return Self::off();
    }
}

impl LsConfig {
    pub(crate) fn init(&self) -> Option<Hertz> {
        let rtc_clk = match self.rtc {
            RtcClockSource::LSI => {
                assert!(self.lsi);
                Some(LSI_FREQ)
            }
            RtcClockSource::LSE => Some(self.lse.as_ref().unwrap().frequency),
            RtcClockSource::DISABLE => None,
            _ => todo!(),
        };

        let (lse_en, lse_byp, lse_drv) = match &self.lse {
            Some(c) => match c.mode {
                LseMode::Oscillator(lse_drv) => (true, false, Some(lse_drv)),
                LseMode::Bypass => (true, true, None),
            },
            None => (false, false, None),
        };
        _ = lse_drv; // not all chips have it.

        // Disable backup domain write protection
        unlock();

        if self.lsi {
            #[cfg(any(stm32u5, stm32h5, stm32wba))]
            let csr = crate::pac::RCC.bdcr();
            #[cfg(not(any(stm32u5, stm32h5, stm32wba, stm32c0)))]
            let csr = crate::pac::RCC.csr();
            #[cfg(any(stm32c0))]
            let csr = crate::pac::RCC.csr2();

            #[cfg(not(any(rcc_wb, rcc_wba)))]
            csr.modify(|w| w.set_lsion(true));

            #[cfg(any(rcc_wb, rcc_wba))]
            csr.modify(|w| w.set_lsi1on(true));

            #[cfg(not(any(rcc_wb, rcc_wba)))]
            while !csr.read().lsirdy() {}

            #[cfg(any(rcc_wb, rcc_wba))]
            while !csr.read().lsi1rdy() {}
        }

        // backup domain configuration (LSEON, RTCEN, RTCSEL) is kept across resets.
        // once set, changing it requires a backup domain reset.
        // first check if the configuration matches what we want.

        // check if it's already enabled and in the source we want.
        let reg = bdcr().read();
        let mut ok = true;
        ok &= reg.rtcsel() == self.rtc;
        #[cfg(not(rcc_wba))]
        {
            ok &= reg.rtcen() == (self.rtc != RtcClockSource::DISABLE);
        }
        ok &= reg.lseon() == lse_en;
        ok &= reg.lsebyp() == lse_byp;
        #[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f400, rcc_f410, rcc_l1)))]
        if let Some(lse_drv) = lse_drv {
            ok &= reg.lsedrv() == lse_drv.into();
        }

        // if configuration is OK, we're done.
        if ok {
            trace!("BDCR ok: {:08x}", bdcr().read().0);
            return rtc_clk;
        }

        // If not OK, reset backup domain and configure it.
        #[cfg(not(any(rcc_l0, rcc_l0_v2, rcc_l1, stm32h5, stm32c0)))]
        {
            bdcr().modify(|w| w.set_bdrst(true));
            bdcr().modify(|w| w.set_bdrst(false));
        }
        #[cfg(any(stm32h5))]
        {
            bdcr().modify(|w| w.set_vswrst(true));
            bdcr().modify(|w| w.set_vswrst(false));
        }
        #[cfg(any(stm32c0))]
        {
            bdcr().modify(|w| w.set_rtcrst(true));
            bdcr().modify(|w| w.set_rtcrst(false));
        }

        if lse_en {
            bdcr().modify(|w| {
                #[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f400, rcc_f410, rcc_l1)))]
                if let Some(lse_drv) = lse_drv {
                    w.set_lsedrv(lse_drv.into());
                }
                w.set_lsebyp(lse_byp);
                w.set_lseon(true);
            });

            while !bdcr().read().lserdy() {}
        }

        if self.rtc != RtcClockSource::DISABLE {
            bdcr().modify(|w| {
                #[cfg(any(rtc_v2h7, rtc_v2l4, rtc_v2wb, rtc_v3, rtc_v3u5))]
                assert!(!w.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

                #[cfg(not(rcc_wba))]
                w.set_rtcen(true);
                w.set_rtcsel(self.rtc);
            });
        }

        trace!("BDCR configured: {:08x}", bdcr().read().0);

        compiler_fence(Ordering::SeqCst);

        rtc_clk
    }
}
