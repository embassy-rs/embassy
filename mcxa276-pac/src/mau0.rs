#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x10],
    sys_ctlr: SysCtlr,
    gexp_status_ie: GexpStatusIe,
    gexp_status: GexpStatus,
    _reserved3: [u8; 0x14],
    op_ctrl: OpCtrl,
    _reserved4: [u8; 0x04],
    res_status_ie: ResStatusIe,
    res_status: ResStatus,
    res0: Res0,
    res1: Res1,
    res2: Res2,
    res3: Res3,
}
impl RegisterBlock {
    #[doc = "0x10 - System Control"]
    #[inline(always)]
    pub const fn sys_ctlr(&self) -> &SysCtlr {
        &self.sys_ctlr
    }
    #[doc = "0x14 - General Exception Status Interrupt Enable"]
    #[inline(always)]
    pub const fn gexp_status_ie(&self) -> &GexpStatusIe {
        &self.gexp_status_ie
    }
    #[doc = "0x18 - General Exception Status"]
    #[inline(always)]
    pub const fn gexp_status(&self) -> &GexpStatus {
        &self.gexp_status
    }
    #[doc = "0x30 - Operation Control"]
    #[inline(always)]
    pub const fn op_ctrl(&self) -> &OpCtrl {
        &self.op_ctrl
    }
    #[doc = "0x38 - Result Status Interrupt Enable"]
    #[inline(always)]
    pub const fn res_status_ie(&self) -> &ResStatusIe {
        &self.res_status_ie
    }
    #[doc = "0x3c - Result Status"]
    #[inline(always)]
    pub const fn res_status(&self) -> &ResStatus {
        &self.res_status
    }
    #[doc = "0x40 - Result Register 0"]
    #[inline(always)]
    pub const fn res0(&self) -> &Res0 {
        &self.res0
    }
    #[doc = "0x44 - Result Register 1"]
    #[inline(always)]
    pub const fn res1(&self) -> &Res1 {
        &self.res1
    }
    #[doc = "0x48 - Result Register 2"]
    #[inline(always)]
    pub const fn res2(&self) -> &Res2 {
        &self.res2
    }
    #[doc = "0x4c - Result Register 3"]
    #[inline(always)]
    pub const fn res3(&self) -> &Res3 {
        &self.res3
    }
}
#[doc = "SYS_CTLR (rw) register accessor: System Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sys_ctlr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sys_ctlr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sys_ctlr`] module"]
#[doc(alias = "SYS_CTLR")]
pub type SysCtlr = crate::Reg<sys_ctlr::SysCtlrSpec>;
#[doc = "System Control"]
pub mod sys_ctlr;
#[doc = "GEXP_STATUS_IE (rw) register accessor: General Exception Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`gexp_status_ie::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gexp_status_ie::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gexp_status_ie`] module"]
#[doc(alias = "GEXP_STATUS_IE")]
pub type GexpStatusIe = crate::Reg<gexp_status_ie::GexpStatusIeSpec>;
#[doc = "General Exception Status Interrupt Enable"]
pub mod gexp_status_ie;
#[doc = "GEXP_STATUS (rw) register accessor: General Exception Status\n\nYou can [`read`](crate::Reg::read) this register and get [`gexp_status::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gexp_status::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gexp_status`] module"]
#[doc(alias = "GEXP_STATUS")]
pub type GexpStatus = crate::Reg<gexp_status::GexpStatusSpec>;
#[doc = "General Exception Status"]
pub mod gexp_status;
#[doc = "OP_CTRL (rw) register accessor: Operation Control\n\nYou can [`read`](crate::Reg::read) this register and get [`op_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`op_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@op_ctrl`] module"]
#[doc(alias = "OP_CTRL")]
pub type OpCtrl = crate::Reg<op_ctrl::OpCtrlSpec>;
#[doc = "Operation Control"]
pub mod op_ctrl;
#[doc = "RES_STATUS_IE (rw) register accessor: Result Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`res_status_ie::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res_status_ie::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res_status_ie`] module"]
#[doc(alias = "RES_STATUS_IE")]
pub type ResStatusIe = crate::Reg<res_status_ie::ResStatusIeSpec>;
#[doc = "Result Status Interrupt Enable"]
pub mod res_status_ie;
#[doc = "RES_STATUS (rw) register accessor: Result Status\n\nYou can [`read`](crate::Reg::read) this register and get [`res_status::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res_status::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res_status`] module"]
#[doc(alias = "RES_STATUS")]
pub type ResStatus = crate::Reg<res_status::ResStatusSpec>;
#[doc = "Result Status"]
pub mod res_status;
#[doc = "RES0 (rw) register accessor: Result Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`res0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res0`] module"]
#[doc(alias = "RES0")]
pub type Res0 = crate::Reg<res0::Res0Spec>;
#[doc = "Result Register 0"]
pub mod res0;
#[doc = "RES1 (rw) register accessor: Result Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`res1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res1`] module"]
#[doc(alias = "RES1")]
pub type Res1 = crate::Reg<res1::Res1Spec>;
#[doc = "Result Register 1"]
pub mod res1;
#[doc = "RES2 (rw) register accessor: Result Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`res2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res2`] module"]
#[doc(alias = "RES2")]
pub type Res2 = crate::Reg<res2::Res2Spec>;
#[doc = "Result Register 2"]
pub mod res2;
#[doc = "RES3 (rw) register accessor: Result Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`res3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@res3`] module"]
#[doc(alias = "RES3")]
pub type Res3 = crate::Reg<res3::Res3Spec>;
#[doc = "Result Register 3"]
pub mod res3;
