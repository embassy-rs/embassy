#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    _reserved2: [u8; 0x38],
    pdor: Pdor,
    psor: Psor,
    pcor: Pcor,
    ptor: Ptor,
    pdir: Pdir,
    pddr: Pddr,
    pidr: Pidr,
    _reserved9: [u8; 0x04],
    pdr: [Pdr; 32],
    icr: [Icr; 32],
    giclr: Giclr,
    gichr: Gichr,
    _reserved13: [u8; 0x18],
    isfr0: Isfr0,
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
    #[doc = "0x40 - Port Data Output"]
    #[inline(always)]
    pub const fn pdor(&self) -> &Pdor {
        &self.pdor
    }
    #[doc = "0x44 - Port Set Output"]
    #[inline(always)]
    pub const fn psor(&self) -> &Psor {
        &self.psor
    }
    #[doc = "0x48 - Port Clear Output"]
    #[inline(always)]
    pub const fn pcor(&self) -> &Pcor {
        &self.pcor
    }
    #[doc = "0x4c - Port Toggle Output"]
    #[inline(always)]
    pub const fn ptor(&self) -> &Ptor {
        &self.ptor
    }
    #[doc = "0x50 - Port Data Input"]
    #[inline(always)]
    pub const fn pdir(&self) -> &Pdir {
        &self.pdir
    }
    #[doc = "0x54 - Port Data Direction"]
    #[inline(always)]
    pub const fn pddr(&self) -> &Pddr {
        &self.pddr
    }
    #[doc = "0x58 - Port Input Disable"]
    #[inline(always)]
    pub const fn pidr(&self) -> &Pidr {
        &self.pidr
    }
    #[doc = "0x60..0x80 - Pin Data"]
    #[inline(always)]
    pub const fn pdr(&self, n: usize) -> &Pdr {
        &self.pdr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x60..0x80 - Pin Data"]
    #[inline(always)]
    pub fn pdr_iter(&self) -> impl Iterator<Item = &Pdr> {
        self.pdr.iter()
    }
    #[doc = "0x80..0x100 - Interrupt Control index"]
    #[inline(always)]
    pub const fn icr(&self, n: usize) -> &Icr {
        &self.icr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x80..0x100 - Interrupt Control index"]
    #[inline(always)]
    pub fn icr_iter(&self) -> impl Iterator<Item = &Icr> {
        self.icr.iter()
    }
    #[doc = "0x100 - Global Interrupt Control Low"]
    #[inline(always)]
    pub const fn giclr(&self) -> &Giclr {
        &self.giclr
    }
    #[doc = "0x104 - Global Interrupt Control High"]
    #[inline(always)]
    pub const fn gichr(&self) -> &Gichr {
        &self.gichr
    }
    #[doc = "0x120 - Interrupt Status Flag"]
    #[inline(always)]
    pub const fn isfr0(&self) -> &Isfr0 {
        &self.isfr0
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
#[doc = "PDOR (rw) register accessor: Port Data Output\n\nYou can [`read`](crate::Reg::read) this register and get [`pdor::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdor::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pdor`] module"]
#[doc(alias = "PDOR")]
pub type Pdor = crate::Reg<pdor::PdorSpec>;
#[doc = "Port Data Output"]
pub mod pdor;
#[doc = "PSOR (rw) register accessor: Port Set Output\n\nYou can [`read`](crate::Reg::read) this register and get [`psor::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`psor::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@psor`] module"]
#[doc(alias = "PSOR")]
pub type Psor = crate::Reg<psor::PsorSpec>;
#[doc = "Port Set Output"]
pub mod psor;
#[doc = "PCOR (rw) register accessor: Port Clear Output\n\nYou can [`read`](crate::Reg::read) this register and get [`pcor::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcor::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcor`] module"]
#[doc(alias = "PCOR")]
pub type Pcor = crate::Reg<pcor::PcorSpec>;
#[doc = "Port Clear Output"]
pub mod pcor;
#[doc = "PTOR (rw) register accessor: Port Toggle Output\n\nYou can [`read`](crate::Reg::read) this register and get [`ptor::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ptor::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ptor`] module"]
#[doc(alias = "PTOR")]
pub type Ptor = crate::Reg<ptor::PtorSpec>;
#[doc = "Port Toggle Output"]
pub mod ptor;
#[doc = "PDIR (r) register accessor: Port Data Input\n\nYou can [`read`](crate::Reg::read) this register and get [`pdir::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pdir`] module"]
#[doc(alias = "PDIR")]
pub type Pdir = crate::Reg<pdir::PdirSpec>;
#[doc = "Port Data Input"]
pub mod pdir;
#[doc = "PDDR (rw) register accessor: Port Data Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`pddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pddr`] module"]
#[doc(alias = "PDDR")]
pub type Pddr = crate::Reg<pddr::PddrSpec>;
#[doc = "Port Data Direction"]
pub mod pddr;
#[doc = "PIDR (rw) register accessor: Port Input Disable\n\nYou can [`read`](crate::Reg::read) this register and get [`pidr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pidr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pidr`] module"]
#[doc(alias = "PIDR")]
pub type Pidr = crate::Reg<pidr::PidrSpec>;
#[doc = "Port Input Disable"]
pub mod pidr;
#[doc = "PDR (rw) register accessor: Pin Data\n\nYou can [`read`](crate::Reg::read) this register and get [`pdr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pdr`] module"]
#[doc(alias = "PDR")]
pub type Pdr = crate::Reg<pdr::PdrSpec>;
#[doc = "Pin Data"]
pub mod pdr;
#[doc = "ICR (rw) register accessor: Interrupt Control index\n\nYou can [`read`](crate::Reg::read) this register and get [`icr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`icr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@icr`] module"]
#[doc(alias = "ICR")]
pub type Icr = crate::Reg<icr::IcrSpec>;
#[doc = "Interrupt Control index"]
pub mod icr;
#[doc = "GICLR (rw) register accessor: Global Interrupt Control Low\n\nYou can [`read`](crate::Reg::read) this register and get [`giclr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`giclr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@giclr`] module"]
#[doc(alias = "GICLR")]
pub type Giclr = crate::Reg<giclr::GiclrSpec>;
#[doc = "Global Interrupt Control Low"]
pub mod giclr;
#[doc = "GICHR (rw) register accessor: Global Interrupt Control High\n\nYou can [`read`](crate::Reg::read) this register and get [`gichr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gichr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gichr`] module"]
#[doc(alias = "GICHR")]
pub type Gichr = crate::Reg<gichr::GichrSpec>;
#[doc = "Global Interrupt Control High"]
pub mod gichr;
#[doc = "ISFR0 (rw) register accessor: Interrupt Status Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`isfr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`isfr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@isfr0`] module"]
#[doc(alias = "ISFR0")]
pub type Isfr0 = crate::Reg<isfr0::Isfr0Spec>;
#[doc = "Interrupt Status Flag"]
pub mod isfr0;
