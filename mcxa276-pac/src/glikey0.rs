#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ctrl_0: Ctrl0,
    ctrl_1: Ctrl1,
    intr_ctrl: IntrCtrl,
    status: Status,
    _reserved4: [u8; 0xec],
    version: Version,
}
impl RegisterBlock {
    #[doc = "0x00 - Control Register 0 SFR"]
    #[inline(always)]
    pub const fn ctrl_0(&self) -> &Ctrl0 {
        &self.ctrl_0
    }
    #[doc = "0x04 - Control Register 1 SFR"]
    #[inline(always)]
    pub const fn ctrl_1(&self) -> &Ctrl1 {
        &self.ctrl_1
    }
    #[doc = "0x08 - Interrupt Control"]
    #[inline(always)]
    pub const fn intr_ctrl(&self) -> &IntrCtrl {
        &self.intr_ctrl
    }
    #[doc = "0x0c - Status"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0xfc - IP Version"]
    #[inline(always)]
    pub const fn version(&self) -> &Version {
        &self.version
    }
}
#[doc = "CTRL_0 (rw) register accessor: Control Register 0 SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl_0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl_0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl_0`] module"]
#[doc(alias = "CTRL_0")]
pub type Ctrl0 = crate::Reg<ctrl_0::Ctrl0Spec>;
#[doc = "Control Register 0 SFR"]
pub mod ctrl_0;
#[doc = "CTRL_1 (rw) register accessor: Control Register 1 SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl_1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl_1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl_1`] module"]
#[doc(alias = "CTRL_1")]
pub type Ctrl1 = crate::Reg<ctrl_1::Ctrl1Spec>;
#[doc = "Control Register 1 SFR"]
pub mod ctrl_1;
#[doc = "INTR_CTRL (rw) register accessor: Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`intr_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`intr_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@intr_ctrl`] module"]
#[doc(alias = "INTR_CTRL")]
pub type IntrCtrl = crate::Reg<intr_ctrl::IntrCtrlSpec>;
#[doc = "Interrupt Control"]
pub mod intr_ctrl;
#[doc = "STATUS (r) register accessor: Status\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "Status"]
pub mod status;
#[doc = "VERSION (r) register accessor: IP Version\n\nYou can [`read`](crate::Reg::read) this register and get [`version::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@version`] module"]
#[doc(alias = "VERSION")]
pub type Version = crate::Reg<version::VersionSpec>;
#[doc = "IP Version"]
pub mod version;
