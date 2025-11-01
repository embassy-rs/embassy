#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    pe1: Pe1,
    pe2: Pe2,
    _reserved4: [u8; 0x08],
    me: Me,
    de: De,
    pf: Pf,
    _reserved7: [u8; 0x0c],
    filt: Filt,
    _reserved8: [u8; 0x04],
    pdc1: Pdc1,
    pdc2: Pdc2,
    _reserved10: [u8; 0x08],
    fdc: Fdc,
    _reserved11: [u8; 0x04],
    pmc: Pmc,
    _reserved12: [u8; 0x04],
    fmc: Fmc,
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
    #[doc = "0x08 - Pin Enable 1"]
    #[inline(always)]
    pub const fn pe1(&self) -> &Pe1 {
        &self.pe1
    }
    #[doc = "0x0c - Pin Enable 2"]
    #[inline(always)]
    pub const fn pe2(&self) -> &Pe2 {
        &self.pe2
    }
    #[doc = "0x18 - Module Interrupt Enable"]
    #[inline(always)]
    pub const fn me(&self) -> &Me {
        &self.me
    }
    #[doc = "0x1c - Module DMA/Trigger Enable"]
    #[inline(always)]
    pub const fn de(&self) -> &De {
        &self.de
    }
    #[doc = "0x20 - Pin Flag"]
    #[inline(always)]
    pub const fn pf(&self) -> &Pf {
        &self.pf
    }
    #[doc = "0x30 - Pin Filter"]
    #[inline(always)]
    pub const fn filt(&self) -> &Filt {
        &self.filt
    }
    #[doc = "0x38 - Pin DMA/Trigger Configuration 1"]
    #[inline(always)]
    pub const fn pdc1(&self) -> &Pdc1 {
        &self.pdc1
    }
    #[doc = "0x3c - Pin DMA/Trigger Configuration 2"]
    #[inline(always)]
    pub const fn pdc2(&self) -> &Pdc2 {
        &self.pdc2
    }
    #[doc = "0x48 - Pin Filter DMA/Trigger Configuration"]
    #[inline(always)]
    pub const fn fdc(&self) -> &Fdc {
        &self.fdc
    }
    #[doc = "0x50 - Pin Mode Configuration"]
    #[inline(always)]
    pub const fn pmc(&self) -> &Pmc {
        &self.pmc
    }
    #[doc = "0x58 - Pin Filter Mode Configuration"]
    #[inline(always)]
    pub const fn fmc(&self) -> &Fmc {
        &self.fmc
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
#[doc = "PE1 (rw) register accessor: Pin Enable 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pe1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pe1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pe1`] module"]
#[doc(alias = "PE1")]
pub type Pe1 = crate::Reg<pe1::Pe1Spec>;
#[doc = "Pin Enable 1"]
pub mod pe1;
#[doc = "PE2 (rw) register accessor: Pin Enable 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pe2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pe2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pe2`] module"]
#[doc(alias = "PE2")]
pub type Pe2 = crate::Reg<pe2::Pe2Spec>;
#[doc = "Pin Enable 2"]
pub mod pe2;
#[doc = "ME (rw) register accessor: Module Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`me::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`me::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@me`] module"]
#[doc(alias = "ME")]
pub type Me = crate::Reg<me::MeSpec>;
#[doc = "Module Interrupt Enable"]
pub mod me;
#[doc = "DE (rw) register accessor: Module DMA/Trigger Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`de::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`de::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@de`] module"]
#[doc(alias = "DE")]
pub type De = crate::Reg<de::DeSpec>;
#[doc = "Module DMA/Trigger Enable"]
pub mod de;
#[doc = "PF (rw) register accessor: Pin Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`pf::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pf::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pf`] module"]
#[doc(alias = "PF")]
pub type Pf = crate::Reg<pf::PfSpec>;
#[doc = "Pin Flag"]
pub mod pf;
#[doc = "FILT (rw) register accessor: Pin Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`filt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`filt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@filt`] module"]
#[doc(alias = "FILT")]
pub type Filt = crate::Reg<filt::FiltSpec>;
#[doc = "Pin Filter"]
pub mod filt;
#[doc = "PDC1 (rw) register accessor: Pin DMA/Trigger Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pdc1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdc1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pdc1`] module"]
#[doc(alias = "PDC1")]
pub type Pdc1 = crate::Reg<pdc1::Pdc1Spec>;
#[doc = "Pin DMA/Trigger Configuration 1"]
pub mod pdc1;
#[doc = "PDC2 (rw) register accessor: Pin DMA/Trigger Configuration 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pdc2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pdc2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pdc2`] module"]
#[doc(alias = "PDC2")]
pub type Pdc2 = crate::Reg<pdc2::Pdc2Spec>;
#[doc = "Pin DMA/Trigger Configuration 2"]
pub mod pdc2;
#[doc = "FDC (rw) register accessor: Pin Filter DMA/Trigger Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`fdc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fdc`] module"]
#[doc(alias = "FDC")]
pub type Fdc = crate::Reg<fdc::FdcSpec>;
#[doc = "Pin Filter DMA/Trigger Configuration"]
pub mod fdc;
#[doc = "PMC (rw) register accessor: Pin Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`pmc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pmc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pmc`] module"]
#[doc(alias = "PMC")]
pub type Pmc = crate::Reg<pmc::PmcSpec>;
#[doc = "Pin Mode Configuration"]
pub mod pmc;
#[doc = "FMC (rw) register accessor: Pin Filter Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`fmc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fmc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fmc`] module"]
#[doc(alias = "FMC")]
pub type Fmc = crate::Reg<fmc::FmcSpec>;
#[doc = "Pin Filter Mode Configuration"]
pub mod fmc;
