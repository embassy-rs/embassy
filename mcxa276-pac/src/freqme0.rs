#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved_0_read_mode_ctrl_r: [u8; 0x04],
    ctrlstat: Ctrlstat,
    min: Min,
    max: Max,
}
impl RegisterBlock {
    #[doc = "0x00 - Control (in Write mode)"]
    #[inline(always)]
    pub const fn write_mode_ctrl_w(&self) -> &WriteModeCtrlW {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().cast() }
    }
    #[doc = "0x00 - Control (in Read mode)"]
    #[inline(always)]
    pub const fn read_mode_ctrl_r(&self) -> &ReadModeCtrlR {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().cast() }
    }
    #[doc = "0x04 - Control Status"]
    #[inline(always)]
    pub const fn ctrlstat(&self) -> &Ctrlstat {
        &self.ctrlstat
    }
    #[doc = "0x08 - Minimum"]
    #[inline(always)]
    pub const fn min(&self) -> &Min {
        &self.min
    }
    #[doc = "0x0c - Maximum"]
    #[inline(always)]
    pub const fn max(&self) -> &Max {
        &self.max
    }
}
#[doc = "READ_MODE_CTRL_R (r) register accessor: Control (in Read mode)\n\nYou can [`read`](crate::Reg::read) this register and get [`read_mode_ctrl_r::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@read_mode_ctrl_r`] module"]
#[doc(alias = "READ_MODE_CTRL_R")]
pub type ReadModeCtrlR = crate::Reg<read_mode_ctrl_r::ReadModeCtrlRSpec>;
#[doc = "Control (in Read mode)"]
pub mod read_mode_ctrl_r;
#[doc = "WRITE_MODE_CTRL_W (w) register accessor: Control (in Write mode)\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`write_mode_ctrl_w::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@write_mode_ctrl_w`] module"]
#[doc(alias = "WRITE_MODE_CTRL_W")]
pub type WriteModeCtrlW = crate::Reg<write_mode_ctrl_w::WriteModeCtrlWSpec>;
#[doc = "Control (in Write mode)"]
pub mod write_mode_ctrl_w;
#[doc = "CTRLSTAT (rw) register accessor: Control Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrlstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrlstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrlstat`] module"]
#[doc(alias = "CTRLSTAT")]
pub type Ctrlstat = crate::Reg<ctrlstat::CtrlstatSpec>;
#[doc = "Control Status"]
pub mod ctrlstat;
#[doc = "MIN (rw) register accessor: Minimum\n\nYou can [`read`](crate::Reg::read) this register and get [`min::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`min::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@min`] module"]
#[doc(alias = "MIN")]
pub type Min = crate::Reg<min::MinSpec>;
#[doc = "Minimum"]
pub mod min;
#[doc = "MAX (rw) register accessor: Maximum\n\nYou can [`read`](crate::Reg::read) this register and get [`max::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`max::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@max`] module"]
#[doc(alias = "MAX")]
pub type Max = crate::Reg<max::MaxSpec>;
#[doc = "Maximum"]
pub mod max;
