#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    pkc_status: PkcStatus,
    pkc_ctrl: PkcCtrl,
    pkc_cfg: PkcCfg,
    _reserved3: [u8; 0x04],
    pkc_mode1: PkcMode1,
    pkc_xyptr1: PkcXyptr1,
    pkc_zrptr1: PkcZrptr1,
    pkc_len1: PkcLen1,
    pkc_mode2: PkcMode2,
    pkc_xyptr2: PkcXyptr2,
    pkc_zrptr2: PkcZrptr2,
    pkc_len2: PkcLen2,
    _reserved11: [u8; 0x10],
    pkc_uptr: PkcUptr,
    pkc_uptrt: PkcUptrt,
    pkc_ulen: PkcUlen,
    _reserved14: [u8; 0x04],
    pkc_mcdata: PkcMcdata,
    _reserved15: [u8; 0x0c],
    pkc_version: PkcVersion,
    _reserved16: [u8; 0x0f4c],
    pkc_soft_rst: PkcSoftRst,
    _reserved17: [u8; 0x0c],
    pkc_access_err: PkcAccessErr,
    pkc_access_err_clr: PkcAccessErrClr,
    _reserved19: [u8; 0x10],
    pkc_int_clr_enable: PkcIntClrEnable,
    pkc_int_set_enable: PkcIntSetEnable,
    pkc_int_status: PkcIntStatus,
    pkc_int_enable: PkcIntEnable,
    pkc_int_clr_status: PkcIntClrStatus,
    pkc_int_set_status: PkcIntSetStatus,
    _reserved25: [u8; 0x0c],
    pkc_module_id: PkcModuleId,
}
impl RegisterBlock {
    #[doc = "0x00 - Status Register"]
    #[inline(always)]
    pub const fn pkc_status(&self) -> &PkcStatus {
        &self.pkc_status
    }
    #[doc = "0x04 - Control Register"]
    #[inline(always)]
    pub const fn pkc_ctrl(&self) -> &PkcCtrl {
        &self.pkc_ctrl
    }
    #[doc = "0x08 - Configuration register"]
    #[inline(always)]
    pub const fn pkc_cfg(&self) -> &PkcCfg {
        &self.pkc_cfg
    }
    #[doc = "0x10 - Mode register, parameter set 1"]
    #[inline(always)]
    pub const fn pkc_mode1(&self) -> &PkcMode1 {
        &self.pkc_mode1
    }
    #[doc = "0x14 - X+Y pointer register, parameter set 1"]
    #[inline(always)]
    pub const fn pkc_xyptr1(&self) -> &PkcXyptr1 {
        &self.pkc_xyptr1
    }
    #[doc = "0x18 - Z+R pointer register, parameter set 1"]
    #[inline(always)]
    pub const fn pkc_zrptr1(&self) -> &PkcZrptr1 {
        &self.pkc_zrptr1
    }
    #[doc = "0x1c - Length register, parameter set 1"]
    #[inline(always)]
    pub const fn pkc_len1(&self) -> &PkcLen1 {
        &self.pkc_len1
    }
    #[doc = "0x20 - Mode register, parameter set 2"]
    #[inline(always)]
    pub const fn pkc_mode2(&self) -> &PkcMode2 {
        &self.pkc_mode2
    }
    #[doc = "0x24 - X+Y pointer register, parameter set 2"]
    #[inline(always)]
    pub const fn pkc_xyptr2(&self) -> &PkcXyptr2 {
        &self.pkc_xyptr2
    }
    #[doc = "0x28 - Z+R pointer register, parameter set 2"]
    #[inline(always)]
    pub const fn pkc_zrptr2(&self) -> &PkcZrptr2 {
        &self.pkc_zrptr2
    }
    #[doc = "0x2c - Length register, parameter set 2"]
    #[inline(always)]
    pub const fn pkc_len2(&self) -> &PkcLen2 {
        &self.pkc_len2
    }
    #[doc = "0x40 - Universal pointer FUP program"]
    #[inline(always)]
    pub const fn pkc_uptr(&self) -> &PkcUptr {
        &self.pkc_uptr
    }
    #[doc = "0x44 - Universal pointer FUP table"]
    #[inline(always)]
    pub const fn pkc_uptrt(&self) -> &PkcUptrt {
        &self.pkc_uptrt
    }
    #[doc = "0x48 - Universal pointer length"]
    #[inline(always)]
    pub const fn pkc_ulen(&self) -> &PkcUlen {
        &self.pkc_ulen
    }
    #[doc = "0x50 - MC pattern data interface"]
    #[inline(always)]
    pub const fn pkc_mcdata(&self) -> &PkcMcdata {
        &self.pkc_mcdata
    }
    #[doc = "0x60 - PKC version register"]
    #[inline(always)]
    pub const fn pkc_version(&self) -> &PkcVersion {
        &self.pkc_version
    }
    #[doc = "0xfb0 - Software reset"]
    #[inline(always)]
    pub const fn pkc_soft_rst(&self) -> &PkcSoftRst {
        &self.pkc_soft_rst
    }
    #[doc = "0xfc0 - Access Error"]
    #[inline(always)]
    pub const fn pkc_access_err(&self) -> &PkcAccessErr {
        &self.pkc_access_err
    }
    #[doc = "0xfc4 - Clear Access Error"]
    #[inline(always)]
    pub const fn pkc_access_err_clr(&self) -> &PkcAccessErrClr {
        &self.pkc_access_err_clr
    }
    #[doc = "0xfd8 - Interrupt enable clear"]
    #[inline(always)]
    pub const fn pkc_int_clr_enable(&self) -> &PkcIntClrEnable {
        &self.pkc_int_clr_enable
    }
    #[doc = "0xfdc - Interrupt enable set"]
    #[inline(always)]
    pub const fn pkc_int_set_enable(&self) -> &PkcIntSetEnable {
        &self.pkc_int_set_enable
    }
    #[doc = "0xfe0 - Interrupt status"]
    #[inline(always)]
    pub const fn pkc_int_status(&self) -> &PkcIntStatus {
        &self.pkc_int_status
    }
    #[doc = "0xfe4 - Interrupt enable"]
    #[inline(always)]
    pub const fn pkc_int_enable(&self) -> &PkcIntEnable {
        &self.pkc_int_enable
    }
    #[doc = "0xfe8 - Interrupt status clear"]
    #[inline(always)]
    pub const fn pkc_int_clr_status(&self) -> &PkcIntClrStatus {
        &self.pkc_int_clr_status
    }
    #[doc = "0xfec - Interrupt status set"]
    #[inline(always)]
    pub const fn pkc_int_set_status(&self) -> &PkcIntSetStatus {
        &self.pkc_int_set_status
    }
    #[doc = "0xffc - Module ID"]
    #[inline(always)]
    pub const fn pkc_module_id(&self) -> &PkcModuleId {
        &self.pkc_module_id
    }
}
#[doc = "PKC_STATUS (r) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_status`] module"]
#[doc(alias = "PKC_STATUS")]
pub type PkcStatus = crate::Reg<pkc_status::PkcStatusSpec>;
#[doc = "Status Register"]
pub mod pkc_status;
#[doc = "PKC_CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_ctrl`] module"]
#[doc(alias = "PKC_CTRL")]
pub type PkcCtrl = crate::Reg<pkc_ctrl::PkcCtrlSpec>;
#[doc = "Control Register"]
pub mod pkc_ctrl;
#[doc = "PKC_CFG (rw) register accessor: Configuration register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_cfg`] module"]
#[doc(alias = "PKC_CFG")]
pub type PkcCfg = crate::Reg<pkc_cfg::PkcCfgSpec>;
#[doc = "Configuration register"]
pub mod pkc_cfg;
#[doc = "PKC_MODE1 (rw) register accessor: Mode register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mode1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mode1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_mode1`] module"]
#[doc(alias = "PKC_MODE1")]
pub type PkcMode1 = crate::Reg<pkc_mode1::PkcMode1Spec>;
#[doc = "Mode register, parameter set 1"]
pub mod pkc_mode1;
#[doc = "PKC_XYPTR1 (rw) register accessor: X+Y pointer register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_xyptr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_xyptr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_xyptr1`] module"]
#[doc(alias = "PKC_XYPTR1")]
pub type PkcXyptr1 = crate::Reg<pkc_xyptr1::PkcXyptr1Spec>;
#[doc = "X+Y pointer register, parameter set 1"]
pub mod pkc_xyptr1;
#[doc = "PKC_ZRPTR1 (rw) register accessor: Z+R pointer register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_zrptr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_zrptr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_zrptr1`] module"]
#[doc(alias = "PKC_ZRPTR1")]
pub type PkcZrptr1 = crate::Reg<pkc_zrptr1::PkcZrptr1Spec>;
#[doc = "Z+R pointer register, parameter set 1"]
pub mod pkc_zrptr1;
#[doc = "PKC_LEN1 (rw) register accessor: Length register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_len1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_len1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_len1`] module"]
#[doc(alias = "PKC_LEN1")]
pub type PkcLen1 = crate::Reg<pkc_len1::PkcLen1Spec>;
#[doc = "Length register, parameter set 1"]
pub mod pkc_len1;
#[doc = "PKC_MODE2 (rw) register accessor: Mode register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mode2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mode2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_mode2`] module"]
#[doc(alias = "PKC_MODE2")]
pub type PkcMode2 = crate::Reg<pkc_mode2::PkcMode2Spec>;
#[doc = "Mode register, parameter set 2"]
pub mod pkc_mode2;
#[doc = "PKC_XYPTR2 (rw) register accessor: X+Y pointer register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_xyptr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_xyptr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_xyptr2`] module"]
#[doc(alias = "PKC_XYPTR2")]
pub type PkcXyptr2 = crate::Reg<pkc_xyptr2::PkcXyptr2Spec>;
#[doc = "X+Y pointer register, parameter set 2"]
pub mod pkc_xyptr2;
#[doc = "PKC_ZRPTR2 (rw) register accessor: Z+R pointer register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_zrptr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_zrptr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_zrptr2`] module"]
#[doc(alias = "PKC_ZRPTR2")]
pub type PkcZrptr2 = crate::Reg<pkc_zrptr2::PkcZrptr2Spec>;
#[doc = "Z+R pointer register, parameter set 2"]
pub mod pkc_zrptr2;
#[doc = "PKC_LEN2 (rw) register accessor: Length register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_len2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_len2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_len2`] module"]
#[doc(alias = "PKC_LEN2")]
pub type PkcLen2 = crate::Reg<pkc_len2::PkcLen2Spec>;
#[doc = "Length register, parameter set 2"]
pub mod pkc_len2;
#[doc = "PKC_UPTR (rw) register accessor: Universal pointer FUP program\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_uptr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_uptr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_uptr`] module"]
#[doc(alias = "PKC_UPTR")]
pub type PkcUptr = crate::Reg<pkc_uptr::PkcUptrSpec>;
#[doc = "Universal pointer FUP program"]
pub mod pkc_uptr;
#[doc = "PKC_UPTRT (rw) register accessor: Universal pointer FUP table\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_uptrt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_uptrt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_uptrt`] module"]
#[doc(alias = "PKC_UPTRT")]
pub type PkcUptrt = crate::Reg<pkc_uptrt::PkcUptrtSpec>;
#[doc = "Universal pointer FUP table"]
pub mod pkc_uptrt;
#[doc = "PKC_ULEN (rw) register accessor: Universal pointer length\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_ulen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_ulen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_ulen`] module"]
#[doc(alias = "PKC_ULEN")]
pub type PkcUlen = crate::Reg<pkc_ulen::PkcUlenSpec>;
#[doc = "Universal pointer length"]
pub mod pkc_ulen;
#[doc = "PKC_MCDATA (rw) register accessor: MC pattern data interface\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mcdata::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mcdata::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_mcdata`] module"]
#[doc(alias = "PKC_MCDATA")]
pub type PkcMcdata = crate::Reg<pkc_mcdata::PkcMcdataSpec>;
#[doc = "MC pattern data interface"]
pub mod pkc_mcdata;
#[doc = "PKC_VERSION (r) register accessor: PKC version register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_version::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_version`] module"]
#[doc(alias = "PKC_VERSION")]
pub type PkcVersion = crate::Reg<pkc_version::PkcVersionSpec>;
#[doc = "PKC version register"]
pub mod pkc_version;
#[doc = "PKC_SOFT_RST (w) register accessor: Software reset\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_soft_rst::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_soft_rst`] module"]
#[doc(alias = "PKC_SOFT_RST")]
pub type PkcSoftRst = crate::Reg<pkc_soft_rst::PkcSoftRstSpec>;
#[doc = "Software reset"]
pub mod pkc_soft_rst;
#[doc = "PKC_ACCESS_ERR (r) register accessor: Access Error\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_access_err::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_access_err`] module"]
#[doc(alias = "PKC_ACCESS_ERR")]
pub type PkcAccessErr = crate::Reg<pkc_access_err::PkcAccessErrSpec>;
#[doc = "Access Error"]
pub mod pkc_access_err;
#[doc = "PKC_ACCESS_ERR_CLR (w) register accessor: Clear Access Error\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_access_err_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_access_err_clr`] module"]
#[doc(alias = "PKC_ACCESS_ERR_CLR")]
pub type PkcAccessErrClr = crate::Reg<pkc_access_err_clr::PkcAccessErrClrSpec>;
#[doc = "Clear Access Error"]
pub mod pkc_access_err_clr;
#[doc = "PKC_INT_CLR_ENABLE (w) register accessor: Interrupt enable clear\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_clr_enable::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_clr_enable`] module"]
#[doc(alias = "PKC_INT_CLR_ENABLE")]
pub type PkcIntClrEnable = crate::Reg<pkc_int_clr_enable::PkcIntClrEnableSpec>;
#[doc = "Interrupt enable clear"]
pub mod pkc_int_clr_enable;
#[doc = "PKC_INT_SET_ENABLE (w) register accessor: Interrupt enable set\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_set_enable::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_set_enable`] module"]
#[doc(alias = "PKC_INT_SET_ENABLE")]
pub type PkcIntSetEnable = crate::Reg<pkc_int_set_enable::PkcIntSetEnableSpec>;
#[doc = "Interrupt enable set"]
pub mod pkc_int_set_enable;
#[doc = "PKC_INT_STATUS (r) register accessor: Interrupt status\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_int_status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_status`] module"]
#[doc(alias = "PKC_INT_STATUS")]
pub type PkcIntStatus = crate::Reg<pkc_int_status::PkcIntStatusSpec>;
#[doc = "Interrupt status"]
pub mod pkc_int_status;
#[doc = "PKC_INT_ENABLE (r) register accessor: Interrupt enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_int_enable::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_enable`] module"]
#[doc(alias = "PKC_INT_ENABLE")]
pub type PkcIntEnable = crate::Reg<pkc_int_enable::PkcIntEnableSpec>;
#[doc = "Interrupt enable"]
pub mod pkc_int_enable;
#[doc = "PKC_INT_CLR_STATUS (w) register accessor: Interrupt status clear\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_clr_status::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_clr_status`] module"]
#[doc(alias = "PKC_INT_CLR_STATUS")]
pub type PkcIntClrStatus = crate::Reg<pkc_int_clr_status::PkcIntClrStatusSpec>;
#[doc = "Interrupt status clear"]
pub mod pkc_int_clr_status;
#[doc = "PKC_INT_SET_STATUS (w) register accessor: Interrupt status set\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_int_set_status::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_int_set_status`] module"]
#[doc(alias = "PKC_INT_SET_STATUS")]
pub type PkcIntSetStatus = crate::Reg<pkc_int_set_status::PkcIntSetStatusSpec>;
#[doc = "Interrupt status set"]
pub mod pkc_int_set_status;
#[doc = "PKC_MODULE_ID (r) register accessor: Module ID\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_module_id::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkc_module_id`] module"]
#[doc(alias = "PKC_MODULE_ID")]
pub type PkcModuleId = crate::Reg<pkc_module_id::PkcModuleIdSpec>;
#[doc = "Module ID"]
pub mod pkc_module_id;
