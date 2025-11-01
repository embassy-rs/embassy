#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    ccr0: Ccr0,
    ccr1: Ccr1,
    ccr2: Ccr2,
    _reserved5: [u8; 0x04],
    dcr: Dcr,
    ier: Ier,
    csr: Csr,
    rrcr0: Rrcr0,
    rrcr1: Rrcr1,
    rrcsr: Rrcsr,
    rrsr: Rrsr,
    _reserved12: [u8; 0x04],
    rrcr2: Rrcr2,
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
    #[doc = "0x08 - Comparator Control Register 0"]
    #[inline(always)]
    pub const fn ccr0(&self) -> &Ccr0 {
        &self.ccr0
    }
    #[doc = "0x0c - Comparator Control Register 1"]
    #[inline(always)]
    pub const fn ccr1(&self) -> &Ccr1 {
        &self.ccr1
    }
    #[doc = "0x10 - Comparator Control Register 2"]
    #[inline(always)]
    pub const fn ccr2(&self) -> &Ccr2 {
        &self.ccr2
    }
    #[doc = "0x18 - DAC Control"]
    #[inline(always)]
    pub const fn dcr(&self) -> &Dcr {
        &self.dcr
    }
    #[doc = "0x1c - Interrupt Enable"]
    #[inline(always)]
    pub const fn ier(&self) -> &Ier {
        &self.ier
    }
    #[doc = "0x20 - Comparator Status"]
    #[inline(always)]
    pub const fn csr(&self) -> &Csr {
        &self.csr
    }
    #[doc = "0x24 - Round Robin Control Register 0"]
    #[inline(always)]
    pub const fn rrcr0(&self) -> &Rrcr0 {
        &self.rrcr0
    }
    #[doc = "0x28 - Round Robin Control Register 1"]
    #[inline(always)]
    pub const fn rrcr1(&self) -> &Rrcr1 {
        &self.rrcr1
    }
    #[doc = "0x2c - Round Robin Control and Status"]
    #[inline(always)]
    pub const fn rrcsr(&self) -> &Rrcsr {
        &self.rrcsr
    }
    #[doc = "0x30 - Round Robin Status"]
    #[inline(always)]
    pub const fn rrsr(&self) -> &Rrsr {
        &self.rrsr
    }
    #[doc = "0x38 - Round Robin Control Register 2"]
    #[inline(always)]
    pub const fn rrcr2(&self) -> &Rrcr2 {
        &self.rrcr2
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
#[doc = "CCR0 (rw) register accessor: Comparator Control Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr0`] module"]
#[doc(alias = "CCR0")]
pub type Ccr0 = crate::Reg<ccr0::Ccr0Spec>;
#[doc = "Comparator Control Register 0"]
pub mod ccr0;
#[doc = "CCR1 (rw) register accessor: Comparator Control Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr1`] module"]
#[doc(alias = "CCR1")]
pub type Ccr1 = crate::Reg<ccr1::Ccr1Spec>;
#[doc = "Comparator Control Register 1"]
pub mod ccr1;
#[doc = "CCR2 (rw) register accessor: Comparator Control Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr2`] module"]
#[doc(alias = "CCR2")]
pub type Ccr2 = crate::Reg<ccr2::Ccr2Spec>;
#[doc = "Comparator Control Register 2"]
pub mod ccr2;
#[doc = "DCR (rw) register accessor: DAC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`dcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dcr`] module"]
#[doc(alias = "DCR")]
pub type Dcr = crate::Reg<dcr::DcrSpec>;
#[doc = "DAC Control"]
pub mod dcr;
#[doc = "IER (rw) register accessor: Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ier`] module"]
#[doc(alias = "IER")]
pub type Ier = crate::Reg<ier::IerSpec>;
#[doc = "Interrupt Enable"]
pub mod ier;
#[doc = "CSR (rw) register accessor: Comparator Status\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@csr`] module"]
#[doc(alias = "CSR")]
pub type Csr = crate::Reg<csr::CsrSpec>;
#[doc = "Comparator Status"]
pub mod csr;
#[doc = "RRCR0 (rw) register accessor: Round Robin Control Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rrcr0`] module"]
#[doc(alias = "RRCR0")]
pub type Rrcr0 = crate::Reg<rrcr0::Rrcr0Spec>;
#[doc = "Round Robin Control Register 0"]
pub mod rrcr0;
#[doc = "RRCR1 (rw) register accessor: Round Robin Control Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rrcr1`] module"]
#[doc(alias = "RRCR1")]
pub type Rrcr1 = crate::Reg<rrcr1::Rrcr1Spec>;
#[doc = "Round Robin Control Register 1"]
pub mod rrcr1;
#[doc = "RRCSR (rw) register accessor: Round Robin Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rrcsr`] module"]
#[doc(alias = "RRCSR")]
pub type Rrcsr = crate::Reg<rrcsr::RrcsrSpec>;
#[doc = "Round Robin Control and Status"]
pub mod rrcsr;
#[doc = "RRSR (rw) register accessor: Round Robin Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rrsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rrsr`] module"]
#[doc(alias = "RRSR")]
pub type Rrsr = crate::Reg<rrsr::RrsrSpec>;
#[doc = "Round Robin Status"]
pub mod rrsr;
#[doc = "RRCR2 (rw) register accessor: Round Robin Control Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rrcr2`] module"]
#[doc(alias = "RRCR2")]
pub type Rrcr2 = crate::Reg<rrcr2::Rrcr2Spec>;
#[doc = "Round Robin Control Register 2"]
pub mod rrcr2;
