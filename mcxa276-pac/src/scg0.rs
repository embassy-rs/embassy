#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    trim_lock: TrimLock,
    _reserved3: [u8; 0x04],
    csr: Csr,
    rccr: Rccr,
    _reserved5: [u8; 0xe8],
    sosccsr: Sosccsr,
    _reserved6: [u8; 0x04],
    sosccfg: Sosccfg,
    _reserved7: [u8; 0xf4],
    sirccsr: Sirccsr,
    _reserved8: [u8; 0x08],
    sirctcfg: Sirctcfg,
    sirctrim: Sirctrim,
    _reserved10: [u8; 0x04],
    sircstat: Sircstat,
    _reserved11: [u8; 0xe4],
    firccsr: Firccsr,
    _reserved12: [u8; 0x04],
    firccfg: Firccfg,
    _reserved13: [u8; 0x04],
    firctrim: Firctrim,
    _reserved14: [u8; 0xec],
    rosccsr: Rosccsr,
    _reserved15: [u8; 0x01fc],
    spllcsr: Spllcsr,
    spllctrl: Spllctrl,
    spllstat: Spllstat,
    spllndiv: Spllndiv,
    spllmdiv: Spllmdiv,
    spllpdiv: Spllpdiv,
    splllock_cnfg: SplllockCnfg,
    _reserved22: [u8; 0x04],
    spllsscgstat: Spllsscgstat,
    spllsscg0: Spllsscg0,
    spllsscg1: Spllsscg1,
    _reserved25: [u8; 0x01d4],
    ldocsr: Ldocsr,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID Register"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x04 - Parameter Register"]
    #[inline(always)]
    pub const fn param(&self) -> &Param {
        &self.param
    }
    #[doc = "0x08 - Trim Lock register"]
    #[inline(always)]
    pub const fn trim_lock(&self) -> &TrimLock {
        &self.trim_lock
    }
    #[doc = "0x10 - Clock Status Register"]
    #[inline(always)]
    pub const fn csr(&self) -> &Csr {
        &self.csr
    }
    #[doc = "0x14 - Run Clock Control Register"]
    #[inline(always)]
    pub const fn rccr(&self) -> &Rccr {
        &self.rccr
    }
    #[doc = "0x100 - SOSC Control Status Register"]
    #[inline(always)]
    pub const fn sosccsr(&self) -> &Sosccsr {
        &self.sosccsr
    }
    #[doc = "0x108 - SOSC Configuration Register"]
    #[inline(always)]
    pub const fn sosccfg(&self) -> &Sosccfg {
        &self.sosccfg
    }
    #[doc = "0x200 - SIRC Control Status Register"]
    #[inline(always)]
    pub const fn sirccsr(&self) -> &Sirccsr {
        &self.sirccsr
    }
    #[doc = "0x20c - SIRC Trim Configuration Register"]
    #[inline(always)]
    pub const fn sirctcfg(&self) -> &Sirctcfg {
        &self.sirctcfg
    }
    #[doc = "0x210 - SIRC Trim Register"]
    #[inline(always)]
    pub const fn sirctrim(&self) -> &Sirctrim {
        &self.sirctrim
    }
    #[doc = "0x218 - SIRC Auto-trimming Status Register"]
    #[inline(always)]
    pub const fn sircstat(&self) -> &Sircstat {
        &self.sircstat
    }
    #[doc = "0x300 - FIRC Control Status Register"]
    #[inline(always)]
    pub const fn firccsr(&self) -> &Firccsr {
        &self.firccsr
    }
    #[doc = "0x308 - FIRC Configuration Register"]
    #[inline(always)]
    pub const fn firccfg(&self) -> &Firccfg {
        &self.firccfg
    }
    #[doc = "0x310 - FIRC Trim Register"]
    #[inline(always)]
    pub const fn firctrim(&self) -> &Firctrim {
        &self.firctrim
    }
    #[doc = "0x400 - ROSC Control Status Register"]
    #[inline(always)]
    pub const fn rosccsr(&self) -> &Rosccsr {
        &self.rosccsr
    }
    #[doc = "0x600 - SPLL Control Status Register"]
    #[inline(always)]
    pub const fn spllcsr(&self) -> &Spllcsr {
        &self.spllcsr
    }
    #[doc = "0x604 - SPLL Control Register"]
    #[inline(always)]
    pub const fn spllctrl(&self) -> &Spllctrl {
        &self.spllctrl
    }
    #[doc = "0x608 - SPLL Status Register"]
    #[inline(always)]
    pub const fn spllstat(&self) -> &Spllstat {
        &self.spllstat
    }
    #[doc = "0x60c - SPLL N Divider Register"]
    #[inline(always)]
    pub const fn spllndiv(&self) -> &Spllndiv {
        &self.spllndiv
    }
    #[doc = "0x610 - SPLL M Divider Register"]
    #[inline(always)]
    pub const fn spllmdiv(&self) -> &Spllmdiv {
        &self.spllmdiv
    }
    #[doc = "0x614 - SPLL P Divider Register"]
    #[inline(always)]
    pub const fn spllpdiv(&self) -> &Spllpdiv {
        &self.spllpdiv
    }
    #[doc = "0x618 - SPLL LOCK Configuration Register"]
    #[inline(always)]
    pub const fn splllock_cnfg(&self) -> &SplllockCnfg {
        &self.splllock_cnfg
    }
    #[doc = "0x620 - SPLL SSCG Status Register"]
    #[inline(always)]
    pub const fn spllsscgstat(&self) -> &Spllsscgstat {
        &self.spllsscgstat
    }
    #[doc = "0x624 - SPLL Spread Spectrum Control 0 Register"]
    #[inline(always)]
    pub const fn spllsscg0(&self) -> &Spllsscg0 {
        &self.spllsscg0
    }
    #[doc = "0x628 - SPLL Spread Spectrum Control 1 Register"]
    #[inline(always)]
    pub const fn spllsscg1(&self) -> &Spllsscg1 {
        &self.spllsscg1
    }
    #[doc = "0x800 - LDO Control and Status Register"]
    #[inline(always)]
    pub const fn ldocsr(&self) -> &Ldocsr {
        &self.ldocsr
    }
}
#[doc = "VERID (r) register accessor: Version ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID Register"]
pub mod verid;
#[doc = "PARAM (r) register accessor: Parameter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@param`] module"]
#[doc(alias = "PARAM")]
pub type Param = crate::Reg<param::ParamSpec>;
#[doc = "Parameter Register"]
pub mod param;
#[doc = "TRIM_LOCK (rw) register accessor: Trim Lock register\n\nYou can [`read`](crate::Reg::read) this register and get [`trim_lock::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trim_lock::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trim_lock`] module"]
#[doc(alias = "TRIM_LOCK")]
pub type TrimLock = crate::Reg<trim_lock::TrimLockSpec>;
#[doc = "Trim Lock register"]
pub mod trim_lock;
#[doc = "CSR (r) register accessor: Clock Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@csr`] module"]
#[doc(alias = "CSR")]
pub type Csr = crate::Reg<csr::CsrSpec>;
#[doc = "Clock Status Register"]
pub mod csr;
#[doc = "RCCR (rw) register accessor: Run Clock Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rccr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rccr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rccr`] module"]
#[doc(alias = "RCCR")]
pub type Rccr = crate::Reg<rccr::RccrSpec>;
#[doc = "Run Clock Control Register"]
pub mod rccr;
#[doc = "SOSCCSR (rw) register accessor: SOSC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sosccsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sosccsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sosccsr`] module"]
#[doc(alias = "SOSCCSR")]
pub type Sosccsr = crate::Reg<sosccsr::SosccsrSpec>;
#[doc = "SOSC Control Status Register"]
pub mod sosccsr;
#[doc = "SOSCCFG (rw) register accessor: SOSC Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sosccfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sosccfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sosccfg`] module"]
#[doc(alias = "SOSCCFG")]
pub type Sosccfg = crate::Reg<sosccfg::SosccfgSpec>;
#[doc = "SOSC Configuration Register"]
pub mod sosccfg;
#[doc = "SIRCCSR (rw) register accessor: SIRC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirccsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirccsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sirccsr`] module"]
#[doc(alias = "SIRCCSR")]
pub type Sirccsr = crate::Reg<sirccsr::SirccsrSpec>;
#[doc = "SIRC Control Status Register"]
pub mod sirccsr;
#[doc = "SIRCTCFG (rw) register accessor: SIRC Trim Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirctcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirctcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sirctcfg`] module"]
#[doc(alias = "SIRCTCFG")]
pub type Sirctcfg = crate::Reg<sirctcfg::SirctcfgSpec>;
#[doc = "SIRC Trim Configuration Register"]
pub mod sirctcfg;
#[doc = "SIRCTRIM (rw) register accessor: SIRC Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirctrim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirctrim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sirctrim`] module"]
#[doc(alias = "SIRCTRIM")]
pub type Sirctrim = crate::Reg<sirctrim::SirctrimSpec>;
#[doc = "SIRC Trim Register"]
pub mod sirctrim;
#[doc = "SIRCSTAT (rw) register accessor: SIRC Auto-trimming Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sircstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sircstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sircstat`] module"]
#[doc(alias = "SIRCSTAT")]
pub type Sircstat = crate::Reg<sircstat::SircstatSpec>;
#[doc = "SIRC Auto-trimming Status Register"]
pub mod sircstat;
#[doc = "FIRCCSR (rw) register accessor: FIRC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firccsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firccsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@firccsr`] module"]
#[doc(alias = "FIRCCSR")]
pub type Firccsr = crate::Reg<firccsr::FirccsrSpec>;
#[doc = "FIRC Control Status Register"]
pub mod firccsr;
#[doc = "FIRCCFG (rw) register accessor: FIRC Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firccfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firccfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@firccfg`] module"]
#[doc(alias = "FIRCCFG")]
pub type Firccfg = crate::Reg<firccfg::FirccfgSpec>;
#[doc = "FIRC Configuration Register"]
pub mod firccfg;
#[doc = "FIRCTRIM (rw) register accessor: FIRC Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firctrim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firctrim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@firctrim`] module"]
#[doc(alias = "FIRCTRIM")]
pub type Firctrim = crate::Reg<firctrim::FirctrimSpec>;
#[doc = "FIRC Trim Register"]
pub mod firctrim;
#[doc = "ROSCCSR (rw) register accessor: ROSC Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rosccsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rosccsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rosccsr`] module"]
#[doc(alias = "ROSCCSR")]
pub type Rosccsr = crate::Reg<rosccsr::RosccsrSpec>;
#[doc = "ROSC Control Status Register"]
pub mod rosccsr;
#[doc = "SPLLCSR (rw) register accessor: SPLL Control Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllcsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllcsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllcsr`] module"]
#[doc(alias = "SPLLCSR")]
pub type Spllcsr = crate::Reg<spllcsr::SpllcsrSpec>;
#[doc = "SPLL Control Status Register"]
pub mod spllcsr;
#[doc = "SPLLCTRL (rw) register accessor: SPLL Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllctrl`] module"]
#[doc(alias = "SPLLCTRL")]
pub type Spllctrl = crate::Reg<spllctrl::SpllctrlSpec>;
#[doc = "SPLL Control Register"]
pub mod spllctrl;
#[doc = "SPLLSTAT (r) register accessor: SPLL Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllstat::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllstat`] module"]
#[doc(alias = "SPLLSTAT")]
pub type Spllstat = crate::Reg<spllstat::SpllstatSpec>;
#[doc = "SPLL Status Register"]
pub mod spllstat;
#[doc = "SPLLNDIV (rw) register accessor: SPLL N Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllndiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllndiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllndiv`] module"]
#[doc(alias = "SPLLNDIV")]
pub type Spllndiv = crate::Reg<spllndiv::SpllndivSpec>;
#[doc = "SPLL N Divider Register"]
pub mod spllndiv;
#[doc = "SPLLMDIV (rw) register accessor: SPLL M Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllmdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllmdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllmdiv`] module"]
#[doc(alias = "SPLLMDIV")]
pub type Spllmdiv = crate::Reg<spllmdiv::SpllmdivSpec>;
#[doc = "SPLL M Divider Register"]
pub mod spllmdiv;
#[doc = "SPLLPDIV (rw) register accessor: SPLL P Divider Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllpdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllpdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllpdiv`] module"]
#[doc(alias = "SPLLPDIV")]
pub type Spllpdiv = crate::Reg<spllpdiv::SpllpdivSpec>;
#[doc = "SPLL P Divider Register"]
pub mod spllpdiv;
#[doc = "SPLLLOCK_CNFG (rw) register accessor: SPLL LOCK Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`splllock_cnfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`splllock_cnfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@splllock_cnfg`] module"]
#[doc(alias = "SPLLLOCK_CNFG")]
pub type SplllockCnfg = crate::Reg<splllock_cnfg::SplllockCnfgSpec>;
#[doc = "SPLL LOCK Configuration Register"]
pub mod splllock_cnfg;
#[doc = "SPLLSSCGSTAT (r) register accessor: SPLL SSCG Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllsscgstat::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllsscgstat`] module"]
#[doc(alias = "SPLLSSCGSTAT")]
pub type Spllsscgstat = crate::Reg<spllsscgstat::SpllsscgstatSpec>;
#[doc = "SPLL SSCG Status Register"]
pub mod spllsscgstat;
#[doc = "SPLLSSCG0 (rw) register accessor: SPLL Spread Spectrum Control 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllsscg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllsscg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllsscg0`] module"]
#[doc(alias = "SPLLSSCG0")]
pub type Spllsscg0 = crate::Reg<spllsscg0::Spllsscg0Spec>;
#[doc = "SPLL Spread Spectrum Control 0 Register"]
pub mod spllsscg0;
#[doc = "SPLLSSCG1 (rw) register accessor: SPLL Spread Spectrum Control 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllsscg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllsscg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@spllsscg1`] module"]
#[doc(alias = "SPLLSSCG1")]
pub type Spllsscg1 = crate::Reg<spllsscg1::Spllsscg1Spec>;
#[doc = "SPLL Spread Spectrum Control 1 Register"]
pub mod spllsscg1;
#[doc = "LDOCSR (rw) register accessor: LDO Control and Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ldocsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ldocsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ldocsr`] module"]
#[doc(alias = "LDOCSR")]
pub type Ldocsr = crate::Reg<ldocsr::LdocsrSpec>;
#[doc = "LDO Control and Status Register"]
pub mod ldocsr;
