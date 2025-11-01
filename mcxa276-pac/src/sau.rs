#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0xd0],
    ctrl: Ctrl,
    type_: Type,
    rnr: Rnr,
    rbar: Rbar,
    rlar: Rlar,
    sfsr: Sfsr,
    sfar: Sfar,
}
impl RegisterBlock {
    #[doc = "0xd0 - Security Attribution Unit Control Register"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0xd4 - Security Attribution Unit Type Register"]
    #[inline(always)]
    pub const fn type_(&self) -> &Type {
        &self.type_
    }
    #[doc = "0xd8 - Security Attribution Unit Region Number Register"]
    #[inline(always)]
    pub const fn rnr(&self) -> &Rnr {
        &self.rnr
    }
    #[doc = "0xdc - Security Attribution Unit Region Base Address Register"]
    #[inline(always)]
    pub const fn rbar(&self) -> &Rbar {
        &self.rbar
    }
    #[doc = "0xe0 - Security Attribution Unit Region Limit Address Register"]
    #[inline(always)]
    pub const fn rlar(&self) -> &Rlar {
        &self.rlar
    }
    #[doc = "0xe4 - Secure Fault Status Register"]
    #[inline(always)]
    pub const fn sfsr(&self) -> &Sfsr {
        &self.sfsr
    }
    #[doc = "0xe8 - Secure Fault Address Register"]
    #[inline(always)]
    pub const fn sfar(&self) -> &Sfar {
        &self.sfar
    }
}
#[doc = "CTRL (rw) register accessor: Security Attribution Unit Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Security Attribution Unit Control Register"]
pub mod ctrl;
#[doc = "TYPE (rw) register accessor: Security Attribution Unit Type Register\n\nYou can [`read`](crate::Reg::read) this register and get [`type_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`type_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@type_`] module"]
#[doc(alias = "TYPE")]
pub type Type = crate::Reg<type_::TypeSpec>;
#[doc = "Security Attribution Unit Type Register"]
pub mod type_;
#[doc = "RNR (rw) register accessor: Security Attribution Unit Region Number Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rnr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rnr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rnr`] module"]
#[doc(alias = "RNR")]
pub type Rnr = crate::Reg<rnr::RnrSpec>;
#[doc = "Security Attribution Unit Region Number Register"]
pub mod rnr;
#[doc = "RBAR (rw) register accessor: Security Attribution Unit Region Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rbar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rbar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rbar`] module"]
#[doc(alias = "RBAR")]
pub type Rbar = crate::Reg<rbar::RbarSpec>;
#[doc = "Security Attribution Unit Region Base Address Register"]
pub mod rbar;
#[doc = "RLAR (rw) register accessor: Security Attribution Unit Region Limit Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rlar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rlar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rlar`] module"]
#[doc(alias = "RLAR")]
pub type Rlar = crate::Reg<rlar::RlarSpec>;
#[doc = "Security Attribution Unit Region Limit Address Register"]
pub mod rlar;
#[doc = "SFSR (rw) register accessor: Secure Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sfsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sfsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sfsr`] module"]
#[doc(alias = "SFSR")]
pub type Sfsr = crate::Reg<sfsr::SfsrSpec>;
#[doc = "Secure Fault Status Register"]
pub mod sfsr;
#[doc = "SFAR (rw) register accessor: Secure Fault Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sfar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sfar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sfar`] module"]
#[doc(alias = "SFAR")]
pub type Sfar = crate::Reg<sfar::SfarSpec>;
#[doc = "Secure Fault Address Register"]
pub mod sfar;
