#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    fstat: Fstat,
    fcnfg: Fcnfg,
    fctrl: Fctrl,
    _reserved3: [u8; 0x04],
    fccob: [Fccob; 8],
}
impl RegisterBlock {
    #[doc = "0x00 - Flash Status Register"]
    #[inline(always)]
    pub const fn fstat(&self) -> &Fstat {
        &self.fstat
    }
    #[doc = "0x04 - Flash Configuration Register"]
    #[inline(always)]
    pub const fn fcnfg(&self) -> &Fcnfg {
        &self.fcnfg
    }
    #[doc = "0x08 - Flash Control Register"]
    #[inline(always)]
    pub const fn fctrl(&self) -> &Fctrl {
        &self.fctrl
    }
    #[doc = "0x10..0x30 - Flash Common Command Object Registers"]
    #[inline(always)]
    pub const fn fccob(&self, n: usize) -> &Fccob {
        &self.fccob[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x10..0x30 - Flash Common Command Object Registers"]
    #[inline(always)]
    pub fn fccob_iter(&self) -> impl Iterator<Item = &Fccob> {
        self.fccob.iter()
    }
}
#[doc = "FSTAT (rw) register accessor: Flash Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fstat`] module"]
#[doc(alias = "FSTAT")]
pub type Fstat = crate::Reg<fstat::FstatSpec>;
#[doc = "Flash Status Register"]
pub mod fstat;
#[doc = "FCNFG (rw) register accessor: Flash Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fcnfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fcnfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fcnfg`] module"]
#[doc(alias = "FCNFG")]
pub type Fcnfg = crate::Reg<fcnfg::FcnfgSpec>;
#[doc = "Flash Configuration Register"]
pub mod fcnfg;
#[doc = "FCTRL (rw) register accessor: Flash Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fctrl`] module"]
#[doc(alias = "FCTRL")]
pub type Fctrl = crate::Reg<fctrl::FctrlSpec>;
#[doc = "Flash Control Register"]
pub mod fctrl;
#[doc = "FCCOB (rw) register accessor: Flash Common Command Object Registers\n\nYou can [`read`](crate::Reg::read) this register and get [`fccob::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fccob::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fccob`] module"]
#[doc(alias = "FCCOB")]
pub type Fccob = crate::Reg<fccob::FccobSpec>;
#[doc = "Flash Common Command Object Registers"]
pub mod fccob;
