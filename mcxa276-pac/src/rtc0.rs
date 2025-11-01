#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    tsr: Tsr,
    tpr: Tpr,
    tar: Tar,
    tcr: Tcr,
    cr: Cr,
    sr: Sr,
    lr: Lr,
    ier: Ier,
}
impl RegisterBlock {
    #[doc = "0x00 - RTC Time Seconds"]
    #[inline(always)]
    pub const fn tsr(&self) -> &Tsr {
        &self.tsr
    }
    #[doc = "0x04 - RTC Time Prescaler"]
    #[inline(always)]
    pub const fn tpr(&self) -> &Tpr {
        &self.tpr
    }
    #[doc = "0x08 - RTC Time Alarm"]
    #[inline(always)]
    pub const fn tar(&self) -> &Tar {
        &self.tar
    }
    #[doc = "0x0c - RTC Time Compensation"]
    #[inline(always)]
    pub const fn tcr(&self) -> &Tcr {
        &self.tcr
    }
    #[doc = "0x10 - RTC Control"]
    #[inline(always)]
    pub const fn cr(&self) -> &Cr {
        &self.cr
    }
    #[doc = "0x14 - RTC Status"]
    #[inline(always)]
    pub const fn sr(&self) -> &Sr {
        &self.sr
    }
    #[doc = "0x18 - RTC Lock"]
    #[inline(always)]
    pub const fn lr(&self) -> &Lr {
        &self.lr
    }
    #[doc = "0x1c - RTC Interrupt Enable"]
    #[inline(always)]
    pub const fn ier(&self) -> &Ier {
        &self.ier
    }
}
#[doc = "TSR (rw) register accessor: RTC Time Seconds\n\nYou can [`read`](crate::Reg::read) this register and get [`tsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tsr`] module"]
#[doc(alias = "TSR")]
pub type Tsr = crate::Reg<tsr::TsrSpec>;
#[doc = "RTC Time Seconds"]
pub mod tsr;
#[doc = "TPR (rw) register accessor: RTC Time Prescaler\n\nYou can [`read`](crate::Reg::read) this register and get [`tpr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tpr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tpr`] module"]
#[doc(alias = "TPR")]
pub type Tpr = crate::Reg<tpr::TprSpec>;
#[doc = "RTC Time Prescaler"]
pub mod tpr;
#[doc = "TAR (rw) register accessor: RTC Time Alarm\n\nYou can [`read`](crate::Reg::read) this register and get [`tar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tar`] module"]
#[doc(alias = "TAR")]
pub type Tar = crate::Reg<tar::TarSpec>;
#[doc = "RTC Time Alarm"]
pub mod tar;
#[doc = "TCR (rw) register accessor: RTC Time Compensation\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcr`] module"]
#[doc(alias = "TCR")]
pub type Tcr = crate::Reg<tcr::TcrSpec>;
#[doc = "RTC Time Compensation"]
pub mod tcr;
#[doc = "CR (rw) register accessor: RTC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cr`] module"]
#[doc(alias = "CR")]
pub type Cr = crate::Reg<cr::CrSpec>;
#[doc = "RTC Control"]
pub mod cr;
#[doc = "SR (rw) register accessor: RTC Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sr`] module"]
#[doc(alias = "SR")]
pub type Sr = crate::Reg<sr::SrSpec>;
#[doc = "RTC Status"]
pub mod sr;
#[doc = "LR (rw) register accessor: RTC Lock\n\nYou can [`read`](crate::Reg::read) this register and get [`lr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lr`] module"]
#[doc(alias = "LR")]
pub type Lr = crate::Reg<lr::LrSpec>;
#[doc = "RTC Lock"]
pub mod lr;
#[doc = "IER (rw) register accessor: RTC Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ier`] module"]
#[doc(alias = "IER")]
pub type Ier = crate::Reg<ier::IerSpec>;
#[doc = "RTC Interrupt Enable"]
pub mod ier;
