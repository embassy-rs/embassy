#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    csr: Csr,
    psr: Psr,
    cmr: Cmr,
    cnr: Cnr,
}
impl RegisterBlock {
    #[doc = "0x00 - Control Status"]
    #[inline(always)]
    pub const fn csr(&self) -> &Csr {
        &self.csr
    }
    #[doc = "0x04 - Prescaler and Glitch Filter"]
    #[inline(always)]
    pub const fn psr(&self) -> &Psr {
        &self.psr
    }
    #[doc = "0x08 - Compare"]
    #[inline(always)]
    pub const fn cmr(&self) -> &Cmr {
        &self.cmr
    }
    #[doc = "0x0c - Counter"]
    #[inline(always)]
    pub const fn cnr(&self) -> &Cnr {
        &self.cnr
    }
}
#[doc = "CSR (rw) register accessor: Control Status\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@csr`] module"]
#[doc(alias = "CSR")]
pub type Csr = crate::Reg<csr::CsrSpec>;
#[doc = "Control Status"]
pub mod csr;
#[doc = "PSR (rw) register accessor: Prescaler and Glitch Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`psr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`psr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@psr`] module"]
#[doc(alias = "PSR")]
pub type Psr = crate::Reg<psr::PsrSpec>;
#[doc = "Prescaler and Glitch Filter"]
pub mod psr;
#[doc = "CMR (rw) register accessor: Compare\n\nYou can [`read`](crate::Reg::read) this register and get [`cmr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmr`] module"]
#[doc(alias = "CMR")]
pub type Cmr = crate::Reg<cmr::CmrSpec>;
#[doc = "Compare"]
pub mod cmr;
#[doc = "CNR (rw) register accessor: Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`cnr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cnr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cnr`] module"]
#[doc(alias = "CNR")]
pub type Cnr = crate::Reg<cnr::CnrSpec>;
#[doc = "Counter"]
pub mod cnr;
