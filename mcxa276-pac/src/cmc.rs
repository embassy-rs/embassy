#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    _reserved1: [u8; 0x0c],
    ckctrl: Ckctrl,
    ckstat: Ckstat,
    pmprot: Pmprot,
    gpmctrl: Gpmctrl,
    pmctrlmain: Pmctrlmain,
    _reserved6: [u8; 0x5c],
    srs: Srs,
    rpc: Rpc,
    ssrs: Ssrs,
    srie: Srie,
    srif: Srif,
    _reserved11: [u8; 0x0c],
    mr0: Mr0,
    _reserved12: [u8; 0x0c],
    fm0: Fm0,
    _reserved13: [u8; 0x2c],
    flashcr: Flashcr,
    _reserved14: [u8; 0x2c],
    corectl: Corectl,
    _reserved15: [u8; 0x0c],
    dbgctl: Dbgctl,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x10 - Clock Control"]
    #[inline(always)]
    pub const fn ckctrl(&self) -> &Ckctrl {
        &self.ckctrl
    }
    #[doc = "0x14 - Clock Status"]
    #[inline(always)]
    pub const fn ckstat(&self) -> &Ckstat {
        &self.ckstat
    }
    #[doc = "0x18 - Power Mode Protection"]
    #[inline(always)]
    pub const fn pmprot(&self) -> &Pmprot {
        &self.pmprot
    }
    #[doc = "0x1c - Global Power Mode Control"]
    #[inline(always)]
    pub const fn gpmctrl(&self) -> &Gpmctrl {
        &self.gpmctrl
    }
    #[doc = "0x20 - Power Mode Control"]
    #[inline(always)]
    pub const fn pmctrlmain(&self) -> &Pmctrlmain {
        &self.pmctrlmain
    }
    #[doc = "0x80 - System Reset Status"]
    #[inline(always)]
    pub const fn srs(&self) -> &Srs {
        &self.srs
    }
    #[doc = "0x84 - Reset Pin Control"]
    #[inline(always)]
    pub const fn rpc(&self) -> &Rpc {
        &self.rpc
    }
    #[doc = "0x88 - Sticky System Reset Status"]
    #[inline(always)]
    pub const fn ssrs(&self) -> &Ssrs {
        &self.ssrs
    }
    #[doc = "0x8c - System Reset Interrupt Enable"]
    #[inline(always)]
    pub const fn srie(&self) -> &Srie {
        &self.srie
    }
    #[doc = "0x90 - System Reset Interrupt Flag"]
    #[inline(always)]
    pub const fn srif(&self) -> &Srif {
        &self.srif
    }
    #[doc = "0xa0 - Mode"]
    #[inline(always)]
    pub const fn mr0(&self) -> &Mr0 {
        &self.mr0
    }
    #[doc = "0xb0 - Force Mode"]
    #[inline(always)]
    pub const fn fm0(&self) -> &Fm0 {
        &self.fm0
    }
    #[doc = "0xe0 - Flash Control"]
    #[inline(always)]
    pub const fn flashcr(&self) -> &Flashcr {
        &self.flashcr
    }
    #[doc = "0x110 - Core Control"]
    #[inline(always)]
    pub const fn corectl(&self) -> &Corectl {
        &self.corectl
    }
    #[doc = "0x120 - Debug Control"]
    #[inline(always)]
    pub const fn dbgctl(&self) -> &Dbgctl {
        &self.dbgctl
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "CKCTRL (rw) register accessor: Clock Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ckctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ckctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ckctrl`] module"]
#[doc(alias = "CKCTRL")]
pub type Ckctrl = crate::Reg<ckctrl::CkctrlSpec>;
#[doc = "Clock Control"]
pub mod ckctrl;
#[doc = "CKSTAT (rw) register accessor: Clock Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ckstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ckstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ckstat`] module"]
#[doc(alias = "CKSTAT")]
pub type Ckstat = crate::Reg<ckstat::CkstatSpec>;
#[doc = "Clock Status"]
pub mod ckstat;
#[doc = "PMPROT (rw) register accessor: Power Mode Protection\n\nYou can [`read`](crate::Reg::read) this register and get [`pmprot::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pmprot::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pmprot`] module"]
#[doc(alias = "PMPROT")]
pub type Pmprot = crate::Reg<pmprot::PmprotSpec>;
#[doc = "Power Mode Protection"]
pub mod pmprot;
#[doc = "GPMCTRL (rw) register accessor: Global Power Mode Control\n\nYou can [`read`](crate::Reg::read) this register and get [`gpmctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpmctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gpmctrl`] module"]
#[doc(alias = "GPMCTRL")]
pub type Gpmctrl = crate::Reg<gpmctrl::GpmctrlSpec>;
#[doc = "Global Power Mode Control"]
pub mod gpmctrl;
#[doc = "PMCTRLMAIN (rw) register accessor: Power Mode Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pmctrlmain::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pmctrlmain::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pmctrlmain`] module"]
#[doc(alias = "PMCTRLMAIN")]
pub type Pmctrlmain = crate::Reg<pmctrlmain::PmctrlmainSpec>;
#[doc = "Power Mode Control"]
pub mod pmctrlmain;
#[doc = "SRS (r) register accessor: System Reset Status\n\nYou can [`read`](crate::Reg::read) this register and get [`srs::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srs`] module"]
#[doc(alias = "SRS")]
pub type Srs = crate::Reg<srs::SrsSpec>;
#[doc = "System Reset Status"]
pub mod srs;
#[doc = "RPC (rw) register accessor: Reset Pin Control\n\nYou can [`read`](crate::Reg::read) this register and get [`rpc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rpc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rpc`] module"]
#[doc(alias = "RPC")]
pub type Rpc = crate::Reg<rpc::RpcSpec>;
#[doc = "Reset Pin Control"]
pub mod rpc;
#[doc = "SSRS (rw) register accessor: Sticky System Reset Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ssrs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ssrs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ssrs`] module"]
#[doc(alias = "SSRS")]
pub type Ssrs = crate::Reg<ssrs::SsrsSpec>;
#[doc = "Sticky System Reset Status"]
pub mod ssrs;
#[doc = "SRIE (rw) register accessor: System Reset Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`srie::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`srie::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srie`] module"]
#[doc(alias = "SRIE")]
pub type Srie = crate::Reg<srie::SrieSpec>;
#[doc = "System Reset Interrupt Enable"]
pub mod srie;
#[doc = "SRIF (rw) register accessor: System Reset Interrupt Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`srif::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`srif::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srif`] module"]
#[doc(alias = "SRIF")]
pub type Srif = crate::Reg<srif::SrifSpec>;
#[doc = "System Reset Interrupt Flag"]
pub mod srif;
#[doc = "MR0 (rw) register accessor: Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mr0`] module"]
#[doc(alias = "MR0")]
pub type Mr0 = crate::Reg<mr0::Mr0Spec>;
#[doc = "Mode"]
pub mod mr0;
#[doc = "FM0 (rw) register accessor: Force Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`fm0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fm0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fm0`] module"]
#[doc(alias = "FM0")]
pub type Fm0 = crate::Reg<fm0::Fm0Spec>;
#[doc = "Force Mode"]
pub mod fm0;
#[doc = "FLASHCR (rw) register accessor: Flash Control\n\nYou can [`read`](crate::Reg::read) this register and get [`flashcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flashcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flashcr`] module"]
#[doc(alias = "FLASHCR")]
pub type Flashcr = crate::Reg<flashcr::FlashcrSpec>;
#[doc = "Flash Control"]
pub mod flashcr;
#[doc = "CORECTL (rw) register accessor: Core Control\n\nYou can [`read`](crate::Reg::read) this register and get [`corectl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corectl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@corectl`] module"]
#[doc(alias = "CORECTL")]
pub type Corectl = crate::Reg<corectl::CorectlSpec>;
#[doc = "Core Control"]
pub mod corectl;
#[doc = "DBGCTL (rw) register accessor: Debug Control\n\nYou can [`read`](crate::Reg::read) this register and get [`dbgctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dbgctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dbgctl`] module"]
#[doc(alias = "DBGCTL")]
pub type Dbgctl = crate::Reg<dbgctl::DbgctlSpec>;
#[doc = "Debug Control"]
pub mod dbgctl;
