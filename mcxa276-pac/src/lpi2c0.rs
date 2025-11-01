#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    _reserved2: [u8; 0x08],
    mcr: Mcr,
    msr: Msr,
    mier: Mier,
    mder: Mder,
    mcfgr0: Mcfgr0,
    mcfgr1: Mcfgr1,
    mcfgr2: Mcfgr2,
    mcfgr3: Mcfgr3,
    _reserved10: [u8; 0x10],
    mdmr: Mdmr,
    _reserved11: [u8; 0x04],
    mccr0: Mccr0,
    _reserved12: [u8; 0x04],
    mccr1: Mccr1,
    _reserved13: [u8; 0x04],
    mfcr: Mfcr,
    mfsr: Mfsr,
    mtdr: Mtdr,
    _reserved16: [u8; 0x0c],
    mrdr: Mrdr,
    _reserved17: [u8; 0x04],
    mrdror: Mrdror,
    _reserved18: [u8; 0x94],
    scr: Scr,
    ssr: Ssr,
    sier: Sier,
    sder: Sder,
    scfgr0: Scfgr0,
    scfgr1: Scfgr1,
    scfgr2: Scfgr2,
    _reserved25: [u8; 0x14],
    samr: Samr,
    _reserved26: [u8; 0x0c],
    sasr: Sasr,
    star: Star,
    _reserved28: [u8; 0x08],
    stdr: Stdr,
    _reserved29: [u8; 0x0c],
    srdr: Srdr,
    _reserved30: [u8; 0x04],
    srdror: Srdror,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x04 - Parameter"]
    #[inline(always)]
    pub const fn param(&self) -> &Param {
        &self.param
    }
    #[doc = "0x10 - Controller Control"]
    #[inline(always)]
    pub const fn mcr(&self) -> &Mcr {
        &self.mcr
    }
    #[doc = "0x14 - Controller Status"]
    #[inline(always)]
    pub const fn msr(&self) -> &Msr {
        &self.msr
    }
    #[doc = "0x18 - Controller Interrupt Enable"]
    #[inline(always)]
    pub const fn mier(&self) -> &Mier {
        &self.mier
    }
    #[doc = "0x1c - Controller DMA Enable"]
    #[inline(always)]
    pub const fn mder(&self) -> &Mder {
        &self.mder
    }
    #[doc = "0x20 - Controller Configuration 0"]
    #[inline(always)]
    pub const fn mcfgr0(&self) -> &Mcfgr0 {
        &self.mcfgr0
    }
    #[doc = "0x24 - Controller Configuration 1"]
    #[inline(always)]
    pub const fn mcfgr1(&self) -> &Mcfgr1 {
        &self.mcfgr1
    }
    #[doc = "0x28 - Controller Configuration 2"]
    #[inline(always)]
    pub const fn mcfgr2(&self) -> &Mcfgr2 {
        &self.mcfgr2
    }
    #[doc = "0x2c - Controller Configuration 3"]
    #[inline(always)]
    pub const fn mcfgr3(&self) -> &Mcfgr3 {
        &self.mcfgr3
    }
    #[doc = "0x40 - Controller Data Match"]
    #[inline(always)]
    pub const fn mdmr(&self) -> &Mdmr {
        &self.mdmr
    }
    #[doc = "0x48 - Controller Clock Configuration 0"]
    #[inline(always)]
    pub const fn mccr0(&self) -> &Mccr0 {
        &self.mccr0
    }
    #[doc = "0x50 - Controller Clock Configuration 1"]
    #[inline(always)]
    pub const fn mccr1(&self) -> &Mccr1 {
        &self.mccr1
    }
    #[doc = "0x58 - Controller FIFO Control"]
    #[inline(always)]
    pub const fn mfcr(&self) -> &Mfcr {
        &self.mfcr
    }
    #[doc = "0x5c - Controller FIFO Status"]
    #[inline(always)]
    pub const fn mfsr(&self) -> &Mfsr {
        &self.mfsr
    }
    #[doc = "0x60 - Controller Transmit Data"]
    #[inline(always)]
    pub const fn mtdr(&self) -> &Mtdr {
        &self.mtdr
    }
    #[doc = "0x70 - Controller Receive Data"]
    #[inline(always)]
    pub const fn mrdr(&self) -> &Mrdr {
        &self.mrdr
    }
    #[doc = "0x78 - Controller Receive Data Read Only"]
    #[inline(always)]
    pub const fn mrdror(&self) -> &Mrdror {
        &self.mrdror
    }
    #[doc = "0x110 - Target Control"]
    #[inline(always)]
    pub const fn scr(&self) -> &Scr {
        &self.scr
    }
    #[doc = "0x114 - Target Status"]
    #[inline(always)]
    pub const fn ssr(&self) -> &Ssr {
        &self.ssr
    }
    #[doc = "0x118 - Target Interrupt Enable"]
    #[inline(always)]
    pub const fn sier(&self) -> &Sier {
        &self.sier
    }
    #[doc = "0x11c - Target DMA Enable"]
    #[inline(always)]
    pub const fn sder(&self) -> &Sder {
        &self.sder
    }
    #[doc = "0x120 - Target Configuration 0"]
    #[inline(always)]
    pub const fn scfgr0(&self) -> &Scfgr0 {
        &self.scfgr0
    }
    #[doc = "0x124 - Target Configuration 1"]
    #[inline(always)]
    pub const fn scfgr1(&self) -> &Scfgr1 {
        &self.scfgr1
    }
    #[doc = "0x128 - Target Configuration 2"]
    #[inline(always)]
    pub const fn scfgr2(&self) -> &Scfgr2 {
        &self.scfgr2
    }
    #[doc = "0x140 - Target Address Match"]
    #[inline(always)]
    pub const fn samr(&self) -> &Samr {
        &self.samr
    }
    #[doc = "0x150 - Target Address Status"]
    #[inline(always)]
    pub const fn sasr(&self) -> &Sasr {
        &self.sasr
    }
    #[doc = "0x154 - Target Transmit ACK"]
    #[inline(always)]
    pub const fn star(&self) -> &Star {
        &self.star
    }
    #[doc = "0x160 - Target Transmit Data"]
    #[inline(always)]
    pub const fn stdr(&self) -> &Stdr {
        &self.stdr
    }
    #[doc = "0x170 - Target Receive Data"]
    #[inline(always)]
    pub const fn srdr(&self) -> &Srdr {
        &self.srdr
    }
    #[doc = "0x178 - Target Receive Data Read Only"]
    #[inline(always)]
    pub const fn srdror(&self) -> &Srdror {
        &self.srdror
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "PARAM (r) register accessor: Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@param`] module"]
#[doc(alias = "PARAM")]
pub type Param = crate::Reg<param::ParamSpec>;
#[doc = "Parameter"]
pub mod param;
#[doc = "MCR (rw) register accessor: Controller Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcr`] module"]
#[doc(alias = "MCR")]
pub type Mcr = crate::Reg<mcr::McrSpec>;
#[doc = "Controller Control"]
pub mod mcr;
#[doc = "MSR (rw) register accessor: Controller Status\n\nYou can [`read`](crate::Reg::read) this register and get [`msr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`msr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@msr`] module"]
#[doc(alias = "MSR")]
pub type Msr = crate::Reg<msr::MsrSpec>;
#[doc = "Controller Status"]
pub mod msr;
#[doc = "MIER (rw) register accessor: Controller Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`mier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mier`] module"]
#[doc(alias = "MIER")]
pub type Mier = crate::Reg<mier::MierSpec>;
#[doc = "Controller Interrupt Enable"]
pub mod mier;
#[doc = "MDER (rw) register accessor: Controller DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`mder::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mder::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mder`] module"]
#[doc(alias = "MDER")]
pub type Mder = crate::Reg<mder::MderSpec>;
#[doc = "Controller DMA Enable"]
pub mod mder;
#[doc = "MCFGR0 (rw) register accessor: Controller Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcfgr0`] module"]
#[doc(alias = "MCFGR0")]
pub type Mcfgr0 = crate::Reg<mcfgr0::Mcfgr0Spec>;
#[doc = "Controller Configuration 0"]
pub mod mcfgr0;
#[doc = "MCFGR1 (rw) register accessor: Controller Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcfgr1`] module"]
#[doc(alias = "MCFGR1")]
pub type Mcfgr1 = crate::Reg<mcfgr1::Mcfgr1Spec>;
#[doc = "Controller Configuration 1"]
pub mod mcfgr1;
#[doc = "MCFGR2 (rw) register accessor: Controller Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcfgr2`] module"]
#[doc(alias = "MCFGR2")]
pub type Mcfgr2 = crate::Reg<mcfgr2::Mcfgr2Spec>;
#[doc = "Controller Configuration 2"]
pub mod mcfgr2;
#[doc = "MCFGR3 (rw) register accessor: Controller Configuration 3\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcfgr3`] module"]
#[doc(alias = "MCFGR3")]
pub type Mcfgr3 = crate::Reg<mcfgr3::Mcfgr3Spec>;
#[doc = "Controller Configuration 3"]
pub mod mcfgr3;
#[doc = "MDMR (rw) register accessor: Controller Data Match\n\nYou can [`read`](crate::Reg::read) this register and get [`mdmr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdmr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mdmr`] module"]
#[doc(alias = "MDMR")]
pub type Mdmr = crate::Reg<mdmr::MdmrSpec>;
#[doc = "Controller Data Match"]
pub mod mdmr;
#[doc = "MCCR0 (rw) register accessor: Controller Clock Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mccr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mccr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mccr0`] module"]
#[doc(alias = "MCCR0")]
pub type Mccr0 = crate::Reg<mccr0::Mccr0Spec>;
#[doc = "Controller Clock Configuration 0"]
pub mod mccr0;
#[doc = "MCCR1 (rw) register accessor: Controller Clock Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mccr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mccr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mccr1`] module"]
#[doc(alias = "MCCR1")]
pub type Mccr1 = crate::Reg<mccr1::Mccr1Spec>;
#[doc = "Controller Clock Configuration 1"]
pub mod mccr1;
#[doc = "MFCR (rw) register accessor: Controller FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mfcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mfcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mfcr`] module"]
#[doc(alias = "MFCR")]
pub type Mfcr = crate::Reg<mfcr::MfcrSpec>;
#[doc = "Controller FIFO Control"]
pub mod mfcr;
#[doc = "MFSR (r) register accessor: Controller FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mfsr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mfsr`] module"]
#[doc(alias = "MFSR")]
pub type Mfsr = crate::Reg<mfsr::MfsrSpec>;
#[doc = "Controller FIFO Status"]
pub mod mfsr;
#[doc = "MTDR (w) register accessor: Controller Transmit Data\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mtdr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mtdr`] module"]
#[doc(alias = "MTDR")]
pub type Mtdr = crate::Reg<mtdr::MtdrSpec>;
#[doc = "Controller Transmit Data"]
pub mod mtdr;
#[doc = "MRDR (r) register accessor: Controller Receive Data\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrdr`] module"]
#[doc(alias = "MRDR")]
pub type Mrdr = crate::Reg<mrdr::MrdrSpec>;
#[doc = "Controller Receive Data"]
pub mod mrdr;
#[doc = "MRDROR (r) register accessor: Controller Receive Data Read Only\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdror::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrdror`] module"]
#[doc(alias = "MRDROR")]
pub type Mrdror = crate::Reg<mrdror::MrdrorSpec>;
#[doc = "Controller Receive Data Read Only"]
pub mod mrdror;
#[doc = "SCR (rw) register accessor: Target Control\n\nYou can [`read`](crate::Reg::read) this register and get [`scr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr`] module"]
#[doc(alias = "SCR")]
pub type Scr = crate::Reg<scr::ScrSpec>;
#[doc = "Target Control"]
pub mod scr;
#[doc = "SSR (rw) register accessor: Target Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ssr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ssr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ssr`] module"]
#[doc(alias = "SSR")]
pub type Ssr = crate::Reg<ssr::SsrSpec>;
#[doc = "Target Status"]
pub mod ssr;
#[doc = "SIER (rw) register accessor: Target Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sier`] module"]
#[doc(alias = "SIER")]
pub type Sier = crate::Reg<sier::SierSpec>;
#[doc = "Target Interrupt Enable"]
pub mod sier;
#[doc = "SDER (rw) register accessor: Target DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sder::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sder::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sder`] module"]
#[doc(alias = "SDER")]
pub type Sder = crate::Reg<sder::SderSpec>;
#[doc = "Target DMA Enable"]
pub mod sder;
#[doc = "SCFGR0 (rw) register accessor: Target Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scfgr0`] module"]
#[doc(alias = "SCFGR0")]
pub type Scfgr0 = crate::Reg<scfgr0::Scfgr0Spec>;
#[doc = "Target Configuration 0"]
pub mod scfgr0;
#[doc = "SCFGR1 (rw) register accessor: Target Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scfgr1`] module"]
#[doc(alias = "SCFGR1")]
pub type Scfgr1 = crate::Reg<scfgr1::Scfgr1Spec>;
#[doc = "Target Configuration 1"]
pub mod scfgr1;
#[doc = "SCFGR2 (rw) register accessor: Target Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scfgr2`] module"]
#[doc(alias = "SCFGR2")]
pub type Scfgr2 = crate::Reg<scfgr2::Scfgr2Spec>;
#[doc = "Target Configuration 2"]
pub mod scfgr2;
#[doc = "SAMR (rw) register accessor: Target Address Match\n\nYou can [`read`](crate::Reg::read) this register and get [`samr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`samr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@samr`] module"]
#[doc(alias = "SAMR")]
pub type Samr = crate::Reg<samr::SamrSpec>;
#[doc = "Target Address Match"]
pub mod samr;
#[doc = "SASR (r) register accessor: Target Address Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sasr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sasr`] module"]
#[doc(alias = "SASR")]
pub type Sasr = crate::Reg<sasr::SasrSpec>;
#[doc = "Target Address Status"]
pub mod sasr;
#[doc = "STAR (rw) register accessor: Target Transmit ACK\n\nYou can [`read`](crate::Reg::read) this register and get [`star::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`star::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@star`] module"]
#[doc(alias = "STAR")]
pub type Star = crate::Reg<star::StarSpec>;
#[doc = "Target Transmit ACK"]
pub mod star;
#[doc = "STDR (w) register accessor: Target Transmit Data\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stdr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stdr`] module"]
#[doc(alias = "STDR")]
pub type Stdr = crate::Reg<stdr::StdrSpec>;
#[doc = "Target Transmit Data"]
pub mod stdr;
#[doc = "SRDR (r) register accessor: Target Receive Data\n\nYou can [`read`](crate::Reg::read) this register and get [`srdr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srdr`] module"]
#[doc(alias = "SRDR")]
pub type Srdr = crate::Reg<srdr::SrdrSpec>;
#[doc = "Target Receive Data"]
pub mod srdr;
#[doc = "SRDROR (r) register accessor: Target Receive Data Read Only\n\nYou can [`read`](crate::Reg::read) this register and get [`srdror::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srdror`] module"]
#[doc(alias = "SRDROR")]
pub type Srdror = crate::Reg<srdror::SrdrorSpec>;
#[doc = "Target Receive Data Read Only"]
pub mod srdror;
