#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    udf_ctrl: UdfCtrl,
    udf_status: UdfStatus,
    udf_wr_data: UdfWrData,
    udf_rd_data: UdfRdData,
}
impl RegisterBlock {
    #[doc = "0x00 - Control register"]
    #[inline(always)]
    pub const fn udf_ctrl(&self) -> &UdfCtrl {
        &self.udf_ctrl
    }
    #[doc = "0x04 - Status register"]
    #[inline(always)]
    pub const fn udf_status(&self) -> &UdfStatus {
        &self.udf_status
    }
    #[doc = "0x08 - Data In Register"]
    #[inline(always)]
    pub const fn udf_wr_data(&self) -> &UdfWrData {
        &self.udf_wr_data
    }
    #[doc = "0x0c - Data Out Register"]
    #[inline(always)]
    pub const fn udf_rd_data(&self) -> &UdfRdData {
        &self.udf_rd_data
    }
}
#[doc = "udf_ctrl (rw) register accessor: Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`udf_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@udf_ctrl`] module"]
#[doc(alias = "udf_ctrl")]
pub type UdfCtrl = crate::Reg<udf_ctrl::UdfCtrlSpec>;
#[doc = "Control register"]
pub mod udf_ctrl;
#[doc = "udf_status (r) register accessor: Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@udf_status`] module"]
#[doc(alias = "udf_status")]
pub type UdfStatus = crate::Reg<udf_status::UdfStatusSpec>;
#[doc = "Status register"]
pub mod udf_status;
#[doc = "udf_wr_data (w) register accessor: Data In Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`udf_wr_data::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@udf_wr_data`] module"]
#[doc(alias = "udf_wr_data")]
pub type UdfWrData = crate::Reg<udf_wr_data::UdfWrDataSpec>;
#[doc = "Data In Register"]
pub mod udf_wr_data;
#[doc = "udf_rd_data (r) register accessor: Data Out Register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_rd_data::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@udf_rd_data`] module"]
#[doc(alias = "udf_rd_data")]
pub type UdfRdData = crate::Reg<udf_rd_data::UdfRdDataSpec>;
#[doc = "Data Out Register"]
pub mod udf_rd_data;
