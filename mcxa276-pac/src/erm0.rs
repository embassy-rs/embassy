#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    cr0: Cr0,
    _reserved1: [u8; 0x0c],
    sr0: Sr0,
    _reserved2: [u8; 0xec],
    ear0: Ear0,
    syn0: Syn0,
    corr_err_cnt0: CorrErrCnt0,
    _reserved5: [u8; 0x0c],
    corr_err_cnt1: CorrErrCnt1,
}
impl RegisterBlock {
    #[doc = "0x00 - ERM Configuration Register 0"]
    #[inline(always)]
    pub const fn cr0(&self) -> &Cr0 {
        &self.cr0
    }
    #[doc = "0x10 - ERM Status Register 0"]
    #[inline(always)]
    pub const fn sr0(&self) -> &Sr0 {
        &self.sr0
    }
    #[doc = "0x100 - ERM Memory 0 Error Address Register"]
    #[inline(always)]
    pub const fn ear0(&self) -> &Ear0 {
        &self.ear0
    }
    #[doc = "0x104 - ERM Memory 0 Syndrome Register"]
    #[inline(always)]
    pub const fn syn0(&self) -> &Syn0 {
        &self.syn0
    }
    #[doc = "0x108 - ERM Memory 0 Correctable Error Count Register"]
    #[inline(always)]
    pub const fn corr_err_cnt0(&self) -> &CorrErrCnt0 {
        &self.corr_err_cnt0
    }
    #[doc = "0x118 - ERM Memory 1 Correctable Error Count Register"]
    #[inline(always)]
    pub const fn corr_err_cnt1(&self) -> &CorrErrCnt1 {
        &self.corr_err_cnt1
    }
}
#[doc = "CR0 (rw) register accessor: ERM Configuration Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`cr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cr0`] module"]
#[doc(alias = "CR0")]
pub type Cr0 = crate::Reg<cr0::Cr0Spec>;
#[doc = "ERM Configuration Register 0"]
pub mod cr0;
#[doc = "SR0 (rw) register accessor: ERM Status Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sr0`] module"]
#[doc(alias = "SR0")]
pub type Sr0 = crate::Reg<sr0::Sr0Spec>;
#[doc = "ERM Status Register 0"]
pub mod sr0;
#[doc = "EAR0 (r) register accessor: ERM Memory 0 Error Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ear0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ear0`] module"]
#[doc(alias = "EAR0")]
pub type Ear0 = crate::Reg<ear0::Ear0Spec>;
#[doc = "ERM Memory 0 Error Address Register"]
pub mod ear0;
#[doc = "SYN0 (r) register accessor: ERM Memory 0 Syndrome Register\n\nYou can [`read`](crate::Reg::read) this register and get [`syn0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@syn0`] module"]
#[doc(alias = "SYN0")]
pub type Syn0 = crate::Reg<syn0::Syn0Spec>;
#[doc = "ERM Memory 0 Syndrome Register"]
pub mod syn0;
#[doc = "CORR_ERR_CNT0 (rw) register accessor: ERM Memory 0 Correctable Error Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`corr_err_cnt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corr_err_cnt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@corr_err_cnt0`] module"]
#[doc(alias = "CORR_ERR_CNT0")]
pub type CorrErrCnt0 = crate::Reg<corr_err_cnt0::CorrErrCnt0Spec>;
#[doc = "ERM Memory 0 Correctable Error Count Register"]
pub mod corr_err_cnt0;
#[doc = "CORR_ERR_CNT1 (rw) register accessor: ERM Memory 1 Correctable Error Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`corr_err_cnt1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corr_err_cnt1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@corr_err_cnt1`] module"]
#[doc(alias = "CORR_ERR_CNT1")]
pub type CorrErrCnt1 = crate::Reg<corr_err_cnt1::CorrErrCnt1Spec>;
#[doc = "ERM Memory 1 Correctable Error Count Register"]
pub mod corr_err_cnt1;
