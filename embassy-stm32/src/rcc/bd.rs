use core::sync::atomic::{Ordering, compiler_fence};

use crate::pac::common::{RW, Reg};
// For the H7, the Retention features live in the pwr registers
#[cfg(all(backup_sram, not(stm32h7)))]
use crate::pac::pwr::vals::Retention;
#[cfg(all(stm32h7, backup_sram))]
use crate::pac::pwr::vals::Retention;
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

#[derive(Clone, Copy)]
pub struct LseConfig {
    pub frequency: Hertz,
    pub mode: LseMode,
    /// If peripherals other than RTC/TAMP or RCC functions need the lse this bit must be set
    #[cfg(any(rcc_l5, rcc_u5, rcc_u3, rcc_wle, rcc_wl5, rcc_wba))]
    pub peripherals_clocked: bool,
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
pub enum LseDrive {
    #[cfg(not(stm32h5))] // ES0565: LSE Low drive mode is not functional
    Low = 0,
    MediumLow = 0x01,
    #[default]
    MediumHigh = 0x02,
    High = 0x03,
}

// All families but these have the LSEDRV register
#[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f410, rcc_l1)))]
impl From<LseDrive> for crate::pac::rcc::vals::Lsedrv {
    fn from(value: LseDrive) -> Self {
        use crate::pac::rcc::vals::Lsedrv;

        match value {
            #[cfg(not(stm32h5))] // ES0565: LSE Low drive mode is not functional
            LseDrive::Low => Lsedrv::Low,
            LseDrive::MediumLow => Lsedrv::MediumLow,
            LseDrive::MediumHigh => Lsedrv::MediumHigh,
            LseDrive::High => Lsedrv::High,
        }
    }
}

#[cfg(not(any(rtc_v2_l0, rtc_v2_l1, stm32c0)))]
type Bdcr = crate::pac::rcc::regs::Bdcr;
#[cfg(any(rtc_v2_l0, rtc_v2_l1))]
type Bdcr = crate::pac::rcc::regs::Csr;
#[cfg(any(stm32c0))]
type Bdcr = crate::pac::rcc::regs::Csr1;

#[cfg(any(stm32c0))]
fn unlock() {}

#[cfg(not(any(stm32c0)))]
fn unlock() {
    #[cfg(any(stm32f0, stm32f1, stm32f2, stm32f3, stm32l0, stm32l1))]
    let cr = crate::pac::PWR.cr();
    #[cfg(not(any(
        stm32f0, stm32f1, stm32f2, stm32f3, stm32l0, stm32l1, stm32u5, stm32u3, stm32h5, stm32wba, stm32n6
    )))]
    let cr = crate::pac::PWR.cr1();
    #[cfg(any(stm32u5, stm32u3, stm32h5, stm32wba, stm32n6))]
    let cr = crate::pac::PWR.dbpcr();

    cr.modify(|w| w.set_dbp(true));
    while !cr.read().dbp() {}
}

fn bdcr() -> Reg<Bdcr, RW> {
    #[cfg(any(rtc_v2_l0, rtc_v2_l1))]
    return crate::pac::RCC.csr();
    #[cfg(not(any(rtc_v2_l0, rtc_v2_l1, stm32c0)))]
    return crate::pac::RCC.bdcr();
    #[cfg(any(stm32c0))]
    return crate::pac::RCC.csr1();
}

#[derive(Clone, Copy)]
pub struct LsConfig {
    pub rtc: RtcClockSource,
    pub lsi: bool,
    pub lse: Option<LseConfig>,
    #[cfg(backup_sram)]
    pub enable_backup_sram: bool,
}

impl LsConfig {
    /// Creates an [`LsConfig`] using the LSI when possible.
    pub const fn new() -> Self {
        // on L5, just the fact that LSI is enabled makes things crash.
        // TODO: investigate.

        #[cfg(not(stm32l5))]
        return Self::default_lsi();
        #[cfg(stm32l5)]
        return Self::off();
    }

    pub const fn default_lse() -> Self {
        Self {
            rtc: RtcClockSource::Lse,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumHigh),
                #[cfg(any(rcc_l5, rcc_u5, rcc_u3, rcc_wle, rcc_wl5, rcc_wba))]
                peripherals_clocked: false,
            }),
            lsi: false,
            #[cfg(backup_sram)]
            enable_backup_sram: false,
        }
    }

    pub const fn default_lsi() -> Self {
        Self {
            rtc: RtcClockSource::Lsi,
            lsi: true,
            lse: None,
            #[cfg(backup_sram)]
            enable_backup_sram: false,
        }
    }

    pub const fn off() -> Self {
        Self {
            rtc: RtcClockSource::Disable,
            lsi: false,
            lse: None,
            #[cfg(backup_sram)]
            enable_backup_sram: false,
        }
    }
}

impl Default for LsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl LsConfig {
    fn log_data() {
        let rtcsel = crate::pac::RCC.ccipr7().read().rtcsel();
        info!("1rtcsel = {:?}", rtcsel);
        let rtcen = crate::pac::RCC.apb4lenr().read().rtcen();
        info!("rtcen = {:?}", if rtcen {1} else {0});
        let lseon = crate::pac::RCC.cr().read().lseon();
        info!("lseon = {:?}", if lseon {1} else {0});
        let lsebyp = crate::pac::RCC.lsecfgr().read().lsebyp();
        info!("lsebyp = {:?}", if lsebyp {1} else {0});
        let lsedrv = crate::pac::RCC.lsecfgr().read().lsedrv();
        info!("lsedrv = {:?}", lsedrv);
    }
    pub(crate) fn init(&self) -> Option<Hertz> {
        Self::log_data();
        let rtc_clk = match self.rtc {
            RtcClockSource::Lsi => {
                assert!(self.lsi);
                Some(LSI_FREQ)
            }
            RtcClockSource::Lse => Some(self.lse.as_ref().unwrap().frequency),
            RtcClockSource::Disable => None,
            _ => todo!(),
        };

        let (lse_en, lse_byp, lse_drv) = match &self.lse {
            Some(c) => match c.mode {
                LseMode::Oscillator(lse_drv) => (true, false, Some(lse_drv)),
                LseMode::Bypass => (true, true, None),
            },
            None => (false, false, None),
        };
        Self::log_data();
        #[cfg(any(rcc_l5, rcc_u5, rcc_wle, rcc_wl5, rcc_wba))]
        let lse_sysen = if let Some(lse) = self.lse {
            Some(lse.peripherals_clocked)
        } else {
            None
        };
        #[cfg(rcc_u0)]
        let lse_sysen = Some(lse_en);

        _ = lse_drv; // not all chips have it.

        // Disable backup domain write protection
        unlock();

        Self::log_data();
        if self.lsi {
            #[cfg(any(stm32u5, stm32h5, stm32wba))]
            let csr = crate::pac::RCC.bdcr();
            #[cfg(stm32n6)]
            let csr = crate::pac::RCC.sr();
            #[cfg(not(any(stm32u5, stm32h5, stm32wba, stm32c0, stm32n6)))]
            let csr = crate::pac::RCC.csr();
            #[cfg(stm32c0)]
            let csr = crate::pac::RCC.csr2();

            #[cfg(not(any(rcc_wb, rcc_wba, rcc_n6)))]
            csr.modify(|w| w.set_lsion(true));

            #[cfg(rcc_n6)]
            crate::pac::RCC.cr().modify(|w| w.set_lsion(true));

            #[cfg(any(rcc_wb, rcc_wba))]
            csr.modify(|w| w.set_lsi1on(true));

            #[cfg(not(any(rcc_wb, rcc_wba)))]
            while !csr.read().lsirdy() {}

            #[cfg(any(rcc_wb, rcc_wba))]
            while !csr.read().lsi1rdy() {}
        }

        Self::log_data();
        // Enable backup regulator for peristent battery backed sram
        #[cfg(backup_sram)]
        {
            #[cfg(stm32h7)]
            unsafe {
                super::BKSRAM_RETAINED = crate::pac::PWR.cr2().read().bren() == Retention::Preserved
            };
            #[cfg(not(stm32h7))]
            unsafe {
                super::BKSRAM_RETAINED = crate::pac::PWR.bdcr().read().bren() == Retention::Preserved
            };

            // H7 has an additional backup SRAM enable bit that must be set in the RCC registers
            #[cfg(stm32h7)]
            crate::pac::RCC.ahb4enr().modify(|w| {
                w.set_bkpsramen(true);
            });

            #[cfg(stm32h7)]
            assert!(crate::pac::PWR.cr1().read().dbp() == true);

            #[cfg(stm32h7)]
            crate::pac::PWR.cr2().modify(|w| {
                w.set_bren(match self.enable_backup_sram {
                    true => Retention::Preserved,
                    false => Retention::Lost,
                });
            });
            #[cfg(not(stm32h7))]
            crate::pac::PWR.bdcr().modify(|w| {
                w.set_bren(match self.enable_backup_sram {
                    true => Retention::Preserved,
                    false => Retention::Lost,
                });
            });

            // Wait for backup regulator voltage to stabilize
            #[cfg(stm32h7)]
            while self.enable_backup_sram && !crate::pac::PWR.cr2().read().brrdy() {}
            #[cfg(not(stm32h7))]
            while self.enable_backup_sram && !crate::pac::PWR.bdsr().read().brrdy() {}
        }

        // backup domain configuration (LSEON, RTCEN, RTCSEL) is kept across resets.
        // once set, changing it requires a backup domain reset.
        // first check if the configuration matches what we want.
        // N6 has all the fields spread across multiple registers under RCC.

        // check if it's already enabled and in the source we want.
        #[cfg(not(rcc_n6))]
        let reg = bdcr().read();
        #[cfg(rcc_n6)]
        let reg = crate::pac::RCC.cr().read();
        #[cfg(rcc_n6)]
        let apb4lenr = crate::pac::RCC.apb4lenr().read();
        #[cfg(rcc_n6)]
        let ccipr7 = crate::pac::RCC.ccipr7().read();
        #[cfg(rcc_n6)]
        let lsecfgr = crate::pac::RCC.lsecfgr().read();

        Self::log_data();
        let mut ok = true;
        info!("OK 1 = {:?}", ok);
        #[cfg(not(rcc_n6))]
        {
            ok &= reg.rtcsel() == self.rtc;
        }
        #[cfg(rcc_n6)]
        {
            ok &= ccipr7.rtcsel() == self.rtc;
        }
        info!("OK 2 = {:?}, {}, {}", ok, ccipr7.rtcsel(), self.rtc);
        #[cfg(not(any(rcc_wba, rcc_n6)))]
        {
            ok &= reg.rtcen() == (self.rtc != RtcClockSource::Disable);
        }
        #[cfg(rcc_n6)]
        {
            ok &= apb4lenr.rtcen() == (self.rtc != RtcClockSource::Disable);
        }
        info!("OK 3 = {:?}, {}, {}", ok, apb4lenr.rtcen(), (self.rtc != RtcClockSource::Disable));
        ok &= reg.lseon() == lse_en;
        #[cfg(not(rcc_n6))]
        {
            ok &= reg.lsebyp() == lse_byp;
        }
        info!("OK 4 = {:?}, {}, {}", ok, reg.lseon(), lse_en);
        #[cfg(rcc_n6)]
        {
            ok &= lsecfgr.lsebyp() == lse_byp;
        }
        info!("OK 5 = {:?}, {}, {}", ok, lsecfgr.lsebyp(), lse_byp);
        #[cfg(any(rcc_l5, rcc_u5, rcc_wle, rcc_wl5, rcc_wba, rcc_u0))]
        if let Some(lse_sysen) = lse_sysen
            && !lse_sysen
        {
            ok &= !reg.lsesysen();
        }
        #[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f410, rcc_l1, rcc_n6)))]
        if let Some(lse_drv) = lse_drv {
            ok &= reg.lsedrv() == lse_drv.into();
        }
        #[cfg(rcc_n6)]
        if let Some(lse_drv) = lse_drv {
            ok &= lsecfgr.lsedrv() == lse_drv.into();
            use crate::pac::rcc::vals::Lsedrv;
            info!("OK 6 = {:?}, {}, {}", ok, lsecfgr.lsedrv().to_bits(), <LseDrive as Into<Lsedrv>>::into(lse_drv).to_bits());
        }

        // After a power-on reset LSESYSEN will be set to 0
        // even if VBAT was present and kept the RTC running
        #[cfg(any(rcc_l5, rcc_u5, rcc_wle, rcc_wl5, rcc_wba, rcc_u0))]
        if ok
            && let Some(lse_sysen) = lse_sysen
            && lse_sysen
        {
            bdcr().modify(|w| {
                w.set_lsesysen(true);
            });

            while !bdcr().read().lsesysrdy() {}
        }

        Self::log_data();
        // if configuration is OK, we're done.
        if ok {
            info!("BDCR ok: {:08x}", bdcr().read().0);
            return rtc_clk;
        }
        Self::log_data();

        // If not OK, reset backup domain and configure it.
        #[cfg(not(any(rcc_l0, rcc_l0_v2, rcc_l1, stm32h5, stm32h7rs, stm32c0, stm32n6)))]
        {
            bdcr().modify(|w| w.set_bdrst(true));
            bdcr().modify(|w| w.set_bdrst(false));
        }
        // H5 has a terrible, terrible errata: 'SRAM2 is erased when the backup domain is reset'
        // pending a more sane sane way to handle this, just don't reset BD for now.
        // This means the RTCSEL write below will have no effect, only if it has already been written
        // after last power-on. Since it's uncommon to dynamically change RTCSEL, this is better than
        // letting half our RAM go magically *poof*.
        // STM32H503CB/EB/KB/RB device errata - 2.2.8 SRAM2 unduly erased upon a backup domain reset
        // STM32H562xx/563xx/573xx device errata - 2.2.14 SRAM2 is erased when the backup domain is reset
        //#[cfg(any(stm32h5, stm32h7rs))]
        #[cfg(any(stm32h7rs, stm32n6))]
        {
            bdcr().modify(|w| w.set_vswrst(true));
            bdcr().modify(|w| w.set_vswrst(false));
        }
        info!("Domain reset");
        #[cfg(any(stm32c0, stm32l0))]
        {
            bdcr().modify(|w| w.set_rtcrst(true));
            bdcr().modify(|w| w.set_rtcrst(false));
        }
        Self::log_data();

        if lse_en {
            #[cfg(not(rcc_n6))]
            {
                bdcr().modify(|w| {
                    #[cfg(not(any(rcc_f1, rcc_f1cl, rcc_f100, rcc_f2, rcc_f4, rcc_f410, rcc_l1)))]
                    if let Some(lse_drv) = lse_drv {
                        w.set_lsedrv(lse_drv.into());
                    }
                    w.set_lsebyp(lse_byp);
                    w.set_lseon(true);
                });

                while !bdcr().read().lserdy() {}
            }
            #[cfg(rcc_n6)]
            {
                crate::pac::RCC.lsecfgr().modify(|w| {
                    if let Some(lse_drv) = lse_drv {
                        w.set_lsedrv(lse_drv.into());
                    }
                    w.set_lsebyp(lse_byp);
                });
                crate::pac::RCC.cr().modify(|w| w.set_lseon(true));

                while !crate::pac::RCC.sr().read().lserdy() {}
            }

            #[cfg(any(rcc_l5, rcc_u5, rcc_wle, rcc_wl5, rcc_wba, rcc_u0))]
            if let Some(lse_sysen) = lse_sysen {
                bdcr().modify(|w| {
                    w.set_lsesysen(lse_sysen);
                });

                if lse_sysen {
                    while !bdcr().read().lsesysrdy() {}
                }
            }
        }
        Self::log_data();

        if self.rtc != RtcClockSource::Disable {
            #[cfg(not(rcc_n6))]
            bdcr().modify(|w| {
                #[cfg(any(rtc_v2_h7, rtc_v2_l4, rtc_v2_wb, rtc_v3_base, rtc_v3_u5))]
                assert!(!w.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

                #[cfg(not(rcc_wba))]
                w.set_rtcen(true);
                w.set_rtcsel(self.rtc);
            });

            #[cfg(rcc_n6)]
            {
                crate::pac::RCC.ccipr7().modify(|w| w.set_rtcsel(self.rtc));
                crate::pac::RCC.apb4lenr().modify(|w| w.set_rtcen(true))
            }
        }
        Self::log_data();

        info!("BDCR configured: {:08x}", bdcr().read().0);

        compiler_fence(Ordering::SeqCst);

        rtc_clk
    }
}
