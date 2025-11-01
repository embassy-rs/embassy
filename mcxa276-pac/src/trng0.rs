#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mctl: Mctl,
    scmisc: Scmisc,
    pkrrng: Pkrrng,
    _reserved_3_pkrsq_pkrsq: [u8; 0x04],
    sdctl: Sdctl,
    _reserved_5_sblim_sblim: [u8; 0x04],
    _reserved_6_frqmin_frqmin: [u8; 0x04],
    _reserved_7_frqcnt_frqcnt: [u8; 0x04],
    _reserved_8_scmc_scmc: [u8; 0x04],
    _reserved_9_scr: [u8; 0x04],
    _reserved_10_scr: [u8; 0x04],
    _reserved_11_scr: [u8; 0x04],
    _reserved_12_scr: [u8; 0x04],
    _reserved_13_scr: [u8; 0x04],
    _reserved_14_scr: [u8; 0x04],
    status: Status,
    ent0: Ent0,
    _reserved17: [u8; 0x3c],
    pkrcnt10: Pkrcnt10,
    pkrcnt32: Pkrcnt32,
    pkrcnt54: Pkrcnt54,
    pkrcnt76: Pkrcnt76,
    pkrcnt98: Pkrcnt98,
    pkrcntba: Pkrcntba,
    pkrcntdc: Pkrcntdc,
    pkrcntfe: Pkrcntfe,
    sec_cfg: SecCfg,
    int_ctrl: IntCtrl,
    int_mask: IntMask,
    int_status: IntStatus,
    cser: Cser,
    csclr: Csclr,
    _reserved31: [u8; 0x34],
    osc2_ctl: Osc2Ctl,
    vid1: Vid1,
    vid2: Vid2,
}
impl RegisterBlock {
    #[doc = "0x00 - Miscellaneous Control Register"]
    #[inline(always)]
    pub const fn mctl(&self) -> &Mctl {
        &self.mctl
    }
    #[doc = "0x04 - Statistical Check Miscellaneous Register"]
    #[inline(always)]
    pub const fn scmisc(&self) -> &Scmisc {
        &self.scmisc
    }
    #[doc = "0x08 - Poker Range Register"]
    #[inline(always)]
    pub const fn pkrrng(&self) -> &Pkrrng {
        &self.pkrrng
    }
    #[doc = "0x0c - Poker Square Calculation Result Register"]
    #[inline(always)]
    pub const fn pkrsq_pkrsq(&self) -> &PkrsqPkrsq {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(12).cast() }
    }
    #[doc = "0x0c - Poker Maximum Limit Register"]
    #[inline(always)]
    pub const fn pkrmax_pkrmax(&self) -> &PkrmaxPkrmax {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(12).cast() }
    }
    #[doc = "0x10 - Seed Control Register"]
    #[inline(always)]
    pub const fn sdctl(&self) -> &Sdctl {
        &self.sdctl
    }
    #[doc = "0x14 - Total Samples Register"]
    #[inline(always)]
    pub const fn totsam_totsam(&self) -> &TotsamTotsam {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(20).cast() }
    }
    #[doc = "0x14 - Sparse Bit Limit Register"]
    #[inline(always)]
    pub const fn sblim_sblim(&self) -> &SblimSblim {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(20).cast() }
    }
    #[doc = "0x18 - Oscillator-2 Frequency Count Register"]
    #[inline(always)]
    pub const fn osc2_frqcnt_osc2_frqcnt(&self) -> &Osc2FrqcntOsc2Frqcnt {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(24).cast() }
    }
    #[doc = "0x18 - Frequency Count Minimum Limit Register"]
    #[inline(always)]
    pub const fn frqmin_frqmin(&self) -> &FrqminFrqmin {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(24).cast() }
    }
    #[doc = "0x1c - Frequency Count Maximum Limit Register"]
    #[inline(always)]
    pub const fn frqmax_frqmax(&self) -> &FrqmaxFrqmax {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(28).cast() }
    }
    #[doc = "0x1c - Frequency Count Register"]
    #[inline(always)]
    pub const fn frqcnt_frqcnt(&self) -> &FrqcntFrqcnt {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(28).cast() }
    }
    #[doc = "0x20 - Statistical Check Monobit Limit Register"]
    #[inline(always)]
    pub const fn scml_scml(&self) -> &ScmlScml {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(32).cast() }
    }
    #[doc = "0x20 - Statistical Check Monobit Count Register"]
    #[inline(always)]
    pub const fn scmc_scmc(&self) -> &ScmcScmc {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(32).cast() }
    }
    #[doc = "0x24 - Statistical Check Run Length 1 Limit Register"]
    #[inline(always)]
    pub const fn scr1l_scr1l(&self) -> &Scr1lScr1l {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(36).cast() }
    }
    #[doc = "0x24 - Statistical Check Run Length 1 Count Register"]
    #[inline(always)]
    pub const fn scr1c_scr1c(&self) -> &Scr1cScr1c {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(36).cast() }
    }
    #[doc = "0x28 - Statistical Check Run Length 2 Limit Register"]
    #[inline(always)]
    pub const fn scr2l_scr2l(&self) -> &Scr2lScr2l {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(40).cast() }
    }
    #[doc = "0x28 - Statistical Check Run Length 2 Count Register"]
    #[inline(always)]
    pub const fn scr2c_scr2c(&self) -> &Scr2cScr2c {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(40).cast() }
    }
    #[doc = "0x2c - Statistical Check Run Length 3 Limit Register"]
    #[inline(always)]
    pub const fn scr3l_scr3l(&self) -> &Scr3lScr3l {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(44).cast() }
    }
    #[doc = "0x2c - Statistical Check Run Length 3 Count Register"]
    #[inline(always)]
    pub const fn scr3c_scr3c(&self) -> &Scr3cScr3c {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(44).cast() }
    }
    #[doc = "0x30 - Statistical Check Run Length 4 Limit Register"]
    #[inline(always)]
    pub const fn scr4l_scr4l(&self) -> &Scr4lScr4l {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(48).cast() }
    }
    #[doc = "0x30 - Statistical Check Run Length 4 Count Register"]
    #[inline(always)]
    pub const fn scr4c_scr4c(&self) -> &Scr4cScr4c {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(48).cast() }
    }
    #[doc = "0x34 - Statistical Check Run Length 5 Limit Register"]
    #[inline(always)]
    pub const fn scr5l_scr5l(&self) -> &Scr5lScr5l {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(52).cast() }
    }
    #[doc = "0x34 - Statistical Check Run Length 5 Count Register"]
    #[inline(always)]
    pub const fn scr5c_scr5c(&self) -> &Scr5cScr5c {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(52).cast() }
    }
    #[doc = "0x38 - Statistical Check Run Length 6+ Limit Register"]
    #[inline(always)]
    pub const fn scr6pl_scr6pl(&self) -> &Scr6plScr6pl {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(56).cast() }
    }
    #[doc = "0x38 - Statistical Check Run Length 6+ Count Register"]
    #[inline(always)]
    pub const fn scr6pc_scr6pc(&self) -> &Scr6pcScr6pc {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(56).cast() }
    }
    #[doc = "0x3c - Status Register"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x40 - Entropy Read Register"]
    #[inline(always)]
    pub const fn ent0(&self) -> &Ent0 {
        &self.ent0
    }
    #[doc = "0x80 - Statistical Check Poker Count 1 and 0 Register"]
    #[inline(always)]
    pub const fn pkrcnt10(&self) -> &Pkrcnt10 {
        &self.pkrcnt10
    }
    #[doc = "0x84 - Statistical Check Poker Count 3 and 2 Register"]
    #[inline(always)]
    pub const fn pkrcnt32(&self) -> &Pkrcnt32 {
        &self.pkrcnt32
    }
    #[doc = "0x88 - Statistical Check Poker Count 5 and 4 Register"]
    #[inline(always)]
    pub const fn pkrcnt54(&self) -> &Pkrcnt54 {
        &self.pkrcnt54
    }
    #[doc = "0x8c - Statistical Check Poker Count 7 and 6 Register"]
    #[inline(always)]
    pub const fn pkrcnt76(&self) -> &Pkrcnt76 {
        &self.pkrcnt76
    }
    #[doc = "0x90 - Statistical Check Poker Count 9 and 8 Register"]
    #[inline(always)]
    pub const fn pkrcnt98(&self) -> &Pkrcnt98 {
        &self.pkrcnt98
    }
    #[doc = "0x94 - Statistical Check Poker Count B and A Register"]
    #[inline(always)]
    pub const fn pkrcntba(&self) -> &Pkrcntba {
        &self.pkrcntba
    }
    #[doc = "0x98 - Statistical Check Poker Count D and C Register"]
    #[inline(always)]
    pub const fn pkrcntdc(&self) -> &Pkrcntdc {
        &self.pkrcntdc
    }
    #[doc = "0x9c - Statistical Check Poker Count F and E Register"]
    #[inline(always)]
    pub const fn pkrcntfe(&self) -> &Pkrcntfe {
        &self.pkrcntfe
    }
    #[doc = "0xa0 - Security Configuration Register"]
    #[inline(always)]
    pub const fn sec_cfg(&self) -> &SecCfg {
        &self.sec_cfg
    }
    #[doc = "0xa4 - Interrupt Control Register"]
    #[inline(always)]
    pub const fn int_ctrl(&self) -> &IntCtrl {
        &self.int_ctrl
    }
    #[doc = "0xa8 - Mask Register"]
    #[inline(always)]
    pub const fn int_mask(&self) -> &IntMask {
        &self.int_mask
    }
    #[doc = "0xac - Interrupt Status Register"]
    #[inline(always)]
    pub const fn int_status(&self) -> &IntStatus {
        &self.int_status
    }
    #[doc = "0xb0 - Common Security Error Register"]
    #[inline(always)]
    pub const fn cser(&self) -> &Cser {
        &self.cser
    }
    #[doc = "0xb4 - Common Security Clear Register"]
    #[inline(always)]
    pub const fn csclr(&self) -> &Csclr {
        &self.csclr
    }
    #[doc = "0xec - TRNG Oscillator 2 Control Register"]
    #[inline(always)]
    pub const fn osc2_ctl(&self) -> &Osc2Ctl {
        &self.osc2_ctl
    }
    #[doc = "0xf0 - Version ID Register (MS)"]
    #[inline(always)]
    pub const fn vid1(&self) -> &Vid1 {
        &self.vid1
    }
    #[doc = "0xf4 - Version ID Register (LS)"]
    #[inline(always)]
    pub const fn vid2(&self) -> &Vid2 {
        &self.vid2
    }
}
#[doc = "MCTL (rw) register accessor: Miscellaneous Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mctl`] module"]
#[doc(alias = "MCTL")]
pub type Mctl = crate::Reg<mctl::MctlSpec>;
#[doc = "Miscellaneous Control Register"]
pub mod mctl;
#[doc = "SCMISC (rw) register accessor: Statistical Check Miscellaneous Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scmisc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scmisc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scmisc`] module"]
#[doc(alias = "SCMISC")]
pub type Scmisc = crate::Reg<scmisc::ScmiscSpec>;
#[doc = "Statistical Check Miscellaneous Register"]
pub mod scmisc;
#[doc = "PKRRNG (rw) register accessor: Poker Range Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrrng::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkrrng::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrrng`] module"]
#[doc(alias = "PKRRNG")]
pub type Pkrrng = crate::Reg<pkrrng::PkrrngSpec>;
#[doc = "Poker Range Register"]
pub mod pkrrng;
#[doc = "PKRMAX_PKRMAX (rw) register accessor: Poker Maximum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrmax_pkrmax::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkrmax_pkrmax::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrmax_pkrmax`] module"]
#[doc(alias = "PKRMAX_PKRMAX")]
pub type PkrmaxPkrmax = crate::Reg<pkrmax_pkrmax::PkrmaxPkrmaxSpec>;
#[doc = "Poker Maximum Limit Register"]
pub mod pkrmax_pkrmax;
#[doc = "PKRSQ_PKRSQ (r) register accessor: Poker Square Calculation Result Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrsq_pkrsq::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrsq_pkrsq`] module"]
#[doc(alias = "PKRSQ_PKRSQ")]
pub type PkrsqPkrsq = crate::Reg<pkrsq_pkrsq::PkrsqPkrsqSpec>;
#[doc = "Poker Square Calculation Result Register"]
pub mod pkrsq_pkrsq;
#[doc = "SDCTL (rw) register accessor: Seed Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sdctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sdctl`] module"]
#[doc(alias = "SDCTL")]
pub type Sdctl = crate::Reg<sdctl::SdctlSpec>;
#[doc = "Seed Control Register"]
pub mod sdctl;
#[doc = "SBLIM_SBLIM (rw) register accessor: Sparse Bit Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sblim_sblim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sblim_sblim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sblim_sblim`] module"]
#[doc(alias = "SBLIM_SBLIM")]
pub type SblimSblim = crate::Reg<sblim_sblim::SblimSblimSpec>;
#[doc = "Sparse Bit Limit Register"]
pub mod sblim_sblim;
#[doc = "TOTSAM_TOTSAM (r) register accessor: Total Samples Register\n\nYou can [`read`](crate::Reg::read) this register and get [`totsam_totsam::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@totsam_totsam`] module"]
#[doc(alias = "TOTSAM_TOTSAM")]
pub type TotsamTotsam = crate::Reg<totsam_totsam::TotsamTotsamSpec>;
#[doc = "Total Samples Register"]
pub mod totsam_totsam;
#[doc = "FRQMIN_FRQMIN (rw) register accessor: Frequency Count Minimum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqmin_frqmin::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frqmin_frqmin::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frqmin_frqmin`] module"]
#[doc(alias = "FRQMIN_FRQMIN")]
pub type FrqminFrqmin = crate::Reg<frqmin_frqmin::FrqminFrqminSpec>;
#[doc = "Frequency Count Minimum Limit Register"]
pub mod frqmin_frqmin;
#[doc = "OSC2_FRQCNT_OSC2_FRQCNT (r) register accessor: Oscillator-2 Frequency Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`osc2_frqcnt_osc2_frqcnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@osc2_frqcnt_osc2_frqcnt`] module"]
#[doc(alias = "OSC2_FRQCNT_OSC2_FRQCNT")]
pub type Osc2FrqcntOsc2Frqcnt = crate::Reg<osc2_frqcnt_osc2_frqcnt::Osc2FrqcntOsc2FrqcntSpec>;
#[doc = "Oscillator-2 Frequency Count Register"]
pub mod osc2_frqcnt_osc2_frqcnt;
#[doc = "FRQCNT_FRQCNT (r) register accessor: Frequency Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqcnt_frqcnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frqcnt_frqcnt`] module"]
#[doc(alias = "FRQCNT_FRQCNT")]
pub type FrqcntFrqcnt = crate::Reg<frqcnt_frqcnt::FrqcntFrqcntSpec>;
#[doc = "Frequency Count Register"]
pub mod frqcnt_frqcnt;
#[doc = "FRQMAX_FRQMAX (rw) register accessor: Frequency Count Maximum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqmax_frqmax::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frqmax_frqmax::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frqmax_frqmax`] module"]
#[doc(alias = "FRQMAX_FRQMAX")]
pub type FrqmaxFrqmax = crate::Reg<frqmax_frqmax::FrqmaxFrqmaxSpec>;
#[doc = "Frequency Count Maximum Limit Register"]
pub mod frqmax_frqmax;
#[doc = "SCMC_SCMC (r) register accessor: Statistical Check Monobit Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scmc_scmc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scmc_scmc`] module"]
#[doc(alias = "SCMC_SCMC")]
pub type ScmcScmc = crate::Reg<scmc_scmc::ScmcScmcSpec>;
#[doc = "Statistical Check Monobit Count Register"]
pub mod scmc_scmc;
#[doc = "SCML_SCML (rw) register accessor: Statistical Check Monobit Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scml_scml::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scml_scml::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scml_scml`] module"]
#[doc(alias = "SCML_SCML")]
pub type ScmlScml = crate::Reg<scml_scml::ScmlScmlSpec>;
#[doc = "Statistical Check Monobit Limit Register"]
pub mod scml_scml;
#[doc = "SCR1C_SCR1C (r) register accessor: Statistical Check Run Length 1 Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr1c_scr1c::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr1c_scr1c`] module"]
#[doc(alias = "SCR1C_SCR1C")]
pub type Scr1cScr1c = crate::Reg<scr1c_scr1c::Scr1cScr1cSpec>;
#[doc = "Statistical Check Run Length 1 Count Register"]
pub mod scr1c_scr1c;
#[doc = "SCR1L_SCR1L (rw) register accessor: Statistical Check Run Length 1 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr1l_scr1l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr1l_scr1l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr1l_scr1l`] module"]
#[doc(alias = "SCR1L_SCR1L")]
pub type Scr1lScr1l = crate::Reg<scr1l_scr1l::Scr1lScr1lSpec>;
#[doc = "Statistical Check Run Length 1 Limit Register"]
pub mod scr1l_scr1l;
#[doc = "SCR2C_SCR2C (r) register accessor: Statistical Check Run Length 2 Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr2c_scr2c::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr2c_scr2c`] module"]
#[doc(alias = "SCR2C_SCR2C")]
pub type Scr2cScr2c = crate::Reg<scr2c_scr2c::Scr2cScr2cSpec>;
#[doc = "Statistical Check Run Length 2 Count Register"]
pub mod scr2c_scr2c;
#[doc = "SCR2L_SCR2L (rw) register accessor: Statistical Check Run Length 2 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr2l_scr2l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr2l_scr2l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr2l_scr2l`] module"]
#[doc(alias = "SCR2L_SCR2L")]
pub type Scr2lScr2l = crate::Reg<scr2l_scr2l::Scr2lScr2lSpec>;
#[doc = "Statistical Check Run Length 2 Limit Register"]
pub mod scr2l_scr2l;
#[doc = "SCR3C_SCR3C (r) register accessor: Statistical Check Run Length 3 Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr3c_scr3c::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr3c_scr3c`] module"]
#[doc(alias = "SCR3C_SCR3C")]
pub type Scr3cScr3c = crate::Reg<scr3c_scr3c::Scr3cScr3cSpec>;
#[doc = "Statistical Check Run Length 3 Count Register"]
pub mod scr3c_scr3c;
#[doc = "SCR3L_SCR3L (rw) register accessor: Statistical Check Run Length 3 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr3l_scr3l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr3l_scr3l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr3l_scr3l`] module"]
#[doc(alias = "SCR3L_SCR3L")]
pub type Scr3lScr3l = crate::Reg<scr3l_scr3l::Scr3lScr3lSpec>;
#[doc = "Statistical Check Run Length 3 Limit Register"]
pub mod scr3l_scr3l;
#[doc = "SCR4C_SCR4C (r) register accessor: Statistical Check Run Length 4 Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr4c_scr4c::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr4c_scr4c`] module"]
#[doc(alias = "SCR4C_SCR4C")]
pub type Scr4cScr4c = crate::Reg<scr4c_scr4c::Scr4cScr4cSpec>;
#[doc = "Statistical Check Run Length 4 Count Register"]
pub mod scr4c_scr4c;
#[doc = "SCR4L_SCR4L (rw) register accessor: Statistical Check Run Length 4 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr4l_scr4l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr4l_scr4l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr4l_scr4l`] module"]
#[doc(alias = "SCR4L_SCR4L")]
pub type Scr4lScr4l = crate::Reg<scr4l_scr4l::Scr4lScr4lSpec>;
#[doc = "Statistical Check Run Length 4 Limit Register"]
pub mod scr4l_scr4l;
#[doc = "SCR5C_SCR5C (r) register accessor: Statistical Check Run Length 5 Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr5c_scr5c::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr5c_scr5c`] module"]
#[doc(alias = "SCR5C_SCR5C")]
pub type Scr5cScr5c = crate::Reg<scr5c_scr5c::Scr5cScr5cSpec>;
#[doc = "Statistical Check Run Length 5 Count Register"]
pub mod scr5c_scr5c;
#[doc = "SCR5L_SCR5L (rw) register accessor: Statistical Check Run Length 5 Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr5l_scr5l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr5l_scr5l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr5l_scr5l`] module"]
#[doc(alias = "SCR5L_SCR5L")]
pub type Scr5lScr5l = crate::Reg<scr5l_scr5l::Scr5lScr5lSpec>;
#[doc = "Statistical Check Run Length 5 Limit Register"]
pub mod scr5l_scr5l;
#[doc = "SCR6PC_SCR6PC (r) register accessor: Statistical Check Run Length 6+ Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr6pc_scr6pc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr6pc_scr6pc`] module"]
#[doc(alias = "SCR6PC_SCR6PC")]
pub type Scr6pcScr6pc = crate::Reg<scr6pc_scr6pc::Scr6pcScr6pcSpec>;
#[doc = "Statistical Check Run Length 6+ Count Register"]
pub mod scr6pc_scr6pc;
#[doc = "SCR6PL_SCR6PL (rw) register accessor: Statistical Check Run Length 6+ Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`scr6pl_scr6pl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scr6pl_scr6pl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scr6pl_scr6pl`] module"]
#[doc(alias = "SCR6PL_SCR6PL")]
pub type Scr6plScr6pl = crate::Reg<scr6pl_scr6pl::Scr6plScr6plSpec>;
#[doc = "Statistical Check Run Length 6+ Limit Register"]
pub mod scr6pl_scr6pl;
#[doc = "STATUS (r) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "Status Register"]
pub mod status;
#[doc = "ENT0 (r) register accessor: Entropy Read Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ent0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ent0`] module"]
#[doc(alias = "ENT0")]
pub type Ent0 = crate::Reg<ent0::Ent0Spec>;
#[doc = "Entropy Read Register"]
pub mod ent0;
#[doc = "PKRCNT10 (r) register accessor: Statistical Check Poker Count 1 and 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt10::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcnt10`] module"]
#[doc(alias = "PKRCNT10")]
pub type Pkrcnt10 = crate::Reg<pkrcnt10::Pkrcnt10Spec>;
#[doc = "Statistical Check Poker Count 1 and 0 Register"]
pub mod pkrcnt10;
#[doc = "PKRCNT32 (r) register accessor: Statistical Check Poker Count 3 and 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt32::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcnt32`] module"]
#[doc(alias = "PKRCNT32")]
pub type Pkrcnt32 = crate::Reg<pkrcnt32::Pkrcnt32Spec>;
#[doc = "Statistical Check Poker Count 3 and 2 Register"]
pub mod pkrcnt32;
#[doc = "PKRCNT54 (r) register accessor: Statistical Check Poker Count 5 and 4 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt54::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcnt54`] module"]
#[doc(alias = "PKRCNT54")]
pub type Pkrcnt54 = crate::Reg<pkrcnt54::Pkrcnt54Spec>;
#[doc = "Statistical Check Poker Count 5 and 4 Register"]
pub mod pkrcnt54;
#[doc = "PKRCNT76 (r) register accessor: Statistical Check Poker Count 7 and 6 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt76::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcnt76`] module"]
#[doc(alias = "PKRCNT76")]
pub type Pkrcnt76 = crate::Reg<pkrcnt76::Pkrcnt76Spec>;
#[doc = "Statistical Check Poker Count 7 and 6 Register"]
pub mod pkrcnt76;
#[doc = "PKRCNT98 (r) register accessor: Statistical Check Poker Count 9 and 8 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt98::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcnt98`] module"]
#[doc(alias = "PKRCNT98")]
pub type Pkrcnt98 = crate::Reg<pkrcnt98::Pkrcnt98Spec>;
#[doc = "Statistical Check Poker Count 9 and 8 Register"]
pub mod pkrcnt98;
#[doc = "PKRCNTBA (r) register accessor: Statistical Check Poker Count B and A Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntba::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcntba`] module"]
#[doc(alias = "PKRCNTBA")]
pub type Pkrcntba = crate::Reg<pkrcntba::PkrcntbaSpec>;
#[doc = "Statistical Check Poker Count B and A Register"]
pub mod pkrcntba;
#[doc = "PKRCNTDC (r) register accessor: Statistical Check Poker Count D and C Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntdc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcntdc`] module"]
#[doc(alias = "PKRCNTDC")]
pub type Pkrcntdc = crate::Reg<pkrcntdc::PkrcntdcSpec>;
#[doc = "Statistical Check Poker Count D and C Register"]
pub mod pkrcntdc;
#[doc = "PKRCNTFE (r) register accessor: Statistical Check Poker Count F and E Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntfe::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pkrcntfe`] module"]
#[doc(alias = "PKRCNTFE")]
pub type Pkrcntfe = crate::Reg<pkrcntfe::PkrcntfeSpec>;
#[doc = "Statistical Check Poker Count F and E Register"]
pub mod pkrcntfe;
#[doc = "SEC_CFG (rw) register accessor: Security Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sec_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sec_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sec_cfg`] module"]
#[doc(alias = "SEC_CFG")]
pub type SecCfg = crate::Reg<sec_cfg::SecCfgSpec>;
#[doc = "Security Configuration Register"]
pub mod sec_cfg;
#[doc = "INT_CTRL (rw) register accessor: Interrupt Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`int_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@int_ctrl`] module"]
#[doc(alias = "INT_CTRL")]
pub type IntCtrl = crate::Reg<int_ctrl::IntCtrlSpec>;
#[doc = "Interrupt Control Register"]
pub mod int_ctrl;
#[doc = "INT_MASK (rw) register accessor: Mask Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_mask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`int_mask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@int_mask`] module"]
#[doc(alias = "INT_MASK")]
pub type IntMask = crate::Reg<int_mask::IntMaskSpec>;
#[doc = "Mask Register"]
pub mod int_mask;
#[doc = "INT_STATUS (r) register accessor: Interrupt Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`int_status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@int_status`] module"]
#[doc(alias = "INT_STATUS")]
pub type IntStatus = crate::Reg<int_status::IntStatusSpec>;
#[doc = "Interrupt Status Register"]
pub mod int_status;
#[doc = "CSER (r) register accessor: Common Security Error Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cser::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cser`] module"]
#[doc(alias = "CSER")]
pub type Cser = crate::Reg<cser::CserSpec>;
#[doc = "Common Security Error Register"]
pub mod cser;
#[doc = "CSCLR (w) register accessor: Common Security Clear Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csclr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@csclr`] module"]
#[doc(alias = "CSCLR")]
pub type Csclr = crate::Reg<csclr::CsclrSpec>;
#[doc = "Common Security Clear Register"]
pub mod csclr;
#[doc = "OSC2_CTL (rw) register accessor: TRNG Oscillator 2 Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`osc2_ctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`osc2_ctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@osc2_ctl`] module"]
#[doc(alias = "OSC2_CTL")]
pub type Osc2Ctl = crate::Reg<osc2_ctl::Osc2CtlSpec>;
#[doc = "TRNG Oscillator 2 Control Register"]
pub mod osc2_ctl;
#[doc = "VID1 (r) register accessor: Version ID Register (MS)\n\nYou can [`read`](crate::Reg::read) this register and get [`vid1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@vid1`] module"]
#[doc(alias = "VID1")]
pub type Vid1 = crate::Reg<vid1::Vid1Spec>;
#[doc = "Version ID Register (MS)"]
pub mod vid1;
#[doc = "VID2 (r) register accessor: Version ID Register (LS)\n\nYou can [`read`](crate::Reg::read) this register and get [`vid2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@vid2`] module"]
#[doc(alias = "VID2")]
pub type Vid2 = crate::Reg<vid2::Vid2Spec>;
#[doc = "Version ID Register (LS)"]
pub mod vid2;
