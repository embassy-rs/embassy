#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    _reserved2: [u8; 0x08],
    cr: Cr,
    sr: Sr,
    ier: Ier,
    der: Der,
    cfgr0: Cfgr0,
    cfgr1: Cfgr1,
    _reserved8: [u8; 0x08],
    dmr0: Dmr0,
    dmr1: Dmr1,
    _reserved10: [u8; 0x08],
    ccr: Ccr,
    ccr1: Ccr1,
    _reserved12: [u8; 0x10],
    fcr: Fcr,
    fsr: Fsr,
    tcr: Tcr,
    tdr: Tdr,
    _reserved16: [u8; 0x08],
    rsr: Rsr,
    rdr: Rdr,
    rdror: Rdror,
    _reserved19: [u8; 0x0380],
    tcbr: Tcbr,
    tdbr: [Tdbr; 128],
    rdbr: [Rdbr; 128],
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
    #[doc = "0x10 - Control"]
    #[inline(always)]
    pub const fn cr(&self) -> &Cr {
        &self.cr
    }
    #[doc = "0x14 - Status"]
    #[inline(always)]
    pub const fn sr(&self) -> &Sr {
        &self.sr
    }
    #[doc = "0x18 - Interrupt Enable"]
    #[inline(always)]
    pub const fn ier(&self) -> &Ier {
        &self.ier
    }
    #[doc = "0x1c - DMA Enable"]
    #[inline(always)]
    pub const fn der(&self) -> &Der {
        &self.der
    }
    #[doc = "0x20 - Configuration 0"]
    #[inline(always)]
    pub const fn cfgr0(&self) -> &Cfgr0 {
        &self.cfgr0
    }
    #[doc = "0x24 - Configuration 1"]
    #[inline(always)]
    pub const fn cfgr1(&self) -> &Cfgr1 {
        &self.cfgr1
    }
    #[doc = "0x30 - Data Match 0"]
    #[inline(always)]
    pub const fn dmr0(&self) -> &Dmr0 {
        &self.dmr0
    }
    #[doc = "0x34 - Data Match 1"]
    #[inline(always)]
    pub const fn dmr1(&self) -> &Dmr1 {
        &self.dmr1
    }
    #[doc = "0x40 - Clock Configuration"]
    #[inline(always)]
    pub const fn ccr(&self) -> &Ccr {
        &self.ccr
    }
    #[doc = "0x44 - Clock Configuration 1"]
    #[inline(always)]
    pub const fn ccr1(&self) -> &Ccr1 {
        &self.ccr1
    }
    #[doc = "0x58 - FIFO Control"]
    #[inline(always)]
    pub const fn fcr(&self) -> &Fcr {
        &self.fcr
    }
    #[doc = "0x5c - FIFO Status"]
    #[inline(always)]
    pub const fn fsr(&self) -> &Fsr {
        &self.fsr
    }
    #[doc = "0x60 - Transmit Command"]
    #[inline(always)]
    pub const fn tcr(&self) -> &Tcr {
        &self.tcr
    }
    #[doc = "0x64 - Transmit Data"]
    #[inline(always)]
    pub const fn tdr(&self) -> &Tdr {
        &self.tdr
    }
    #[doc = "0x70 - Receive Status"]
    #[inline(always)]
    pub const fn rsr(&self) -> &Rsr {
        &self.rsr
    }
    #[doc = "0x74 - Receive Data"]
    #[inline(always)]
    pub const fn rdr(&self) -> &Rdr {
        &self.rdr
    }
    #[doc = "0x78 - Receive Data Read Only"]
    #[inline(always)]
    pub const fn rdror(&self) -> &Rdror {
        &self.rdror
    }
    #[doc = "0x3fc - Transmit Command Burst"]
    #[inline(always)]
    pub const fn tcbr(&self) -> &Tcbr {
        &self.tcbr
    }
    #[doc = "0x400..0x600 - Transmit Data Burst"]
    #[inline(always)]
    pub const fn tdbr(&self, n: usize) -> &Tdbr {
        &self.tdbr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x400..0x600 - Transmit Data Burst"]
    #[inline(always)]
    pub fn tdbr_iter(&self) -> impl Iterator<Item = &Tdbr> {
        self.tdbr.iter()
    }
    #[doc = "0x600..0x800 - Receive Data Burst"]
    #[inline(always)]
    pub const fn rdbr(&self, n: usize) -> &Rdbr {
        &self.rdbr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x600..0x800 - Receive Data Burst"]
    #[inline(always)]
    pub fn rdbr_iter(&self) -> impl Iterator<Item = &Rdbr> {
        self.rdbr.iter()
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
#[doc = "CR (rw) register accessor: Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cr`] module"]
#[doc(alias = "CR")]
pub type Cr = crate::Reg<cr::CrSpec>;
#[doc = "Control"]
pub mod cr;
#[doc = "SR (rw) register accessor: Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sr`] module"]
#[doc(alias = "SR")]
pub type Sr = crate::Reg<sr::SrSpec>;
#[doc = "Status"]
pub mod sr;
#[doc = "IER (rw) register accessor: Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ier`] module"]
#[doc(alias = "IER")]
pub type Ier = crate::Reg<ier::IerSpec>;
#[doc = "Interrupt Enable"]
pub mod ier;
#[doc = "DER (rw) register accessor: DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`der::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`der::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@der`] module"]
#[doc(alias = "DER")]
pub type Der = crate::Reg<der::DerSpec>;
#[doc = "DMA Enable"]
pub mod der;
#[doc = "CFGR0 (rw) register accessor: Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`cfgr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfgr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfgr0`] module"]
#[doc(alias = "CFGR0")]
pub type Cfgr0 = crate::Reg<cfgr0::Cfgr0Spec>;
#[doc = "Configuration 0"]
pub mod cfgr0;
#[doc = "CFGR1 (rw) register accessor: Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`cfgr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfgr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfgr1`] module"]
#[doc(alias = "CFGR1")]
pub type Cfgr1 = crate::Reg<cfgr1::Cfgr1Spec>;
#[doc = "Configuration 1"]
pub mod cfgr1;
#[doc = "DMR0 (rw) register accessor: Data Match 0\n\nYou can [`read`](crate::Reg::read) this register and get [`dmr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dmr0`] module"]
#[doc(alias = "DMR0")]
pub type Dmr0 = crate::Reg<dmr0::Dmr0Spec>;
#[doc = "Data Match 0"]
pub mod dmr0;
#[doc = "DMR1 (rw) register accessor: Data Match 1\n\nYou can [`read`](crate::Reg::read) this register and get [`dmr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dmr1`] module"]
#[doc(alias = "DMR1")]
pub type Dmr1 = crate::Reg<dmr1::Dmr1Spec>;
#[doc = "Data Match 1"]
pub mod dmr1;
#[doc = "CCR (rw) register accessor: Clock Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr`] module"]
#[doc(alias = "CCR")]
pub type Ccr = crate::Reg<ccr::CcrSpec>;
#[doc = "Clock Configuration"]
pub mod ccr;
#[doc = "CCR1 (rw) register accessor: Clock Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr1`] module"]
#[doc(alias = "CCR1")]
pub type Ccr1 = crate::Reg<ccr1::Ccr1Spec>;
#[doc = "Clock Configuration 1"]
pub mod ccr1;
#[doc = "FCR (rw) register accessor: FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`fcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fcr`] module"]
#[doc(alias = "FCR")]
pub type Fcr = crate::Reg<fcr::FcrSpec>;
#[doc = "FIFO Control"]
pub mod fcr;
#[doc = "FSR (r) register accessor: FIFO Status\n\nYou can [`read`](crate::Reg::read) this register and get [`fsr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fsr`] module"]
#[doc(alias = "FSR")]
pub type Fsr = crate::Reg<fsr::FsrSpec>;
#[doc = "FIFO Status"]
pub mod fsr;
#[doc = "TCR (rw) register accessor: Transmit Command\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcr`] module"]
#[doc(alias = "TCR")]
pub type Tcr = crate::Reg<tcr::TcrSpec>;
#[doc = "Transmit Command"]
pub mod tcr;
#[doc = "TDR (w) register accessor: Transmit Data\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tdr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tdr`] module"]
#[doc(alias = "TDR")]
pub type Tdr = crate::Reg<tdr::TdrSpec>;
#[doc = "Transmit Data"]
pub mod tdr;
#[doc = "RSR (r) register accessor: Receive Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rsr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rsr`] module"]
#[doc(alias = "RSR")]
pub type Rsr = crate::Reg<rsr::RsrSpec>;
#[doc = "Receive Status"]
pub mod rsr;
#[doc = "RDR (r) register accessor: Receive Data\n\nYou can [`read`](crate::Reg::read) this register and get [`rdr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rdr`] module"]
#[doc(alias = "RDR")]
pub type Rdr = crate::Reg<rdr::RdrSpec>;
#[doc = "Receive Data"]
pub mod rdr;
#[doc = "RDROR (r) register accessor: Receive Data Read Only\n\nYou can [`read`](crate::Reg::read) this register and get [`rdror::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rdror`] module"]
#[doc(alias = "RDROR")]
pub type Rdror = crate::Reg<rdror::RdrorSpec>;
#[doc = "Receive Data Read Only"]
pub mod rdror;
#[doc = "TCBR (w) register accessor: Transmit Command Burst\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcbr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcbr`] module"]
#[doc(alias = "TCBR")]
pub type Tcbr = crate::Reg<tcbr::TcbrSpec>;
#[doc = "Transmit Command Burst"]
pub mod tcbr;
#[doc = "TDBR (w) register accessor: Transmit Data Burst\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tdbr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tdbr`] module"]
#[doc(alias = "TDBR")]
pub type Tdbr = crate::Reg<tdbr::TdbrSpec>;
#[doc = "Transmit Data Burst"]
pub mod tdbr;
#[doc = "RDBR (r) register accessor: Receive Data Burst\n\nYou can [`read`](crate::Reg::read) this register and get [`rdbr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rdbr`] module"]
#[doc(alias = "RDBR")]
pub type Rdbr = crate::Reg<rdbr::RdbrSpec>;
#[doc = "Receive Data Burst"]
pub mod rdbr;
