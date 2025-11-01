#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ctrl: Ctrl,
    ctrl2: Ctrl2,
    filt: Filt,
    lastedge: Lastedge,
    posdper: Posdper,
    posdperbfr: Posdperbfr,
    upos: Upos,
    lpos: Lpos,
    posd: Posd,
    posdh: Posdh,
    uposh: Uposh,
    lposh: Lposh,
    lastedgeh: Lastedgeh,
    posdperh: Posdperh,
    revh: Revh,
    rev: Rev,
    uinit: Uinit,
    linit: Linit,
    umod: Umod,
    lmod: Lmod,
    ucomp0: Ucomp0,
    lcomp0: Lcomp0,
    _reserved_22_ucomp1_ucomp1: [u8; 0x02],
    _reserved_23_lcomp1_lcomp1: [u8; 0x02],
    _reserved_24_ucomp2_ucomp2: [u8; 0x02],
    _reserved_25_lcomp2_lcomp2: [u8; 0x02],
    _reserved_26_ucomp3_ucomp3: [u8; 0x02],
    _reserved_27_lcomp3_lcomp3: [u8; 0x02],
    intctrl: Intctrl,
    wtr: Wtr,
    imr: Imr,
    tst: Tst,
    _reserved32: [u8; 0x10],
    uverid: Uverid,
    lverid: Lverid,
}
impl RegisterBlock {
    #[doc = "0x00 - Control Register"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x02 - Control 2 Register"]
    #[inline(always)]
    pub const fn ctrl2(&self) -> &Ctrl2 {
        &self.ctrl2
    }
    #[doc = "0x04 - Input Filter Register"]
    #[inline(always)]
    pub const fn filt(&self) -> &Filt {
        &self.filt
    }
    #[doc = "0x06 - Last Edge Time Register"]
    #[inline(always)]
    pub const fn lastedge(&self) -> &Lastedge {
        &self.lastedge
    }
    #[doc = "0x08 - Position Difference Period Counter Register"]
    #[inline(always)]
    pub const fn posdper(&self) -> &Posdper {
        &self.posdper
    }
    #[doc = "0x0a - Position Difference Period Buffer Register"]
    #[inline(always)]
    pub const fn posdperbfr(&self) -> &Posdperbfr {
        &self.posdperbfr
    }
    #[doc = "0x0c - Upper Position Counter Register"]
    #[inline(always)]
    pub const fn upos(&self) -> &Upos {
        &self.upos
    }
    #[doc = "0x0e - Lower Position Counter Register"]
    #[inline(always)]
    pub const fn lpos(&self) -> &Lpos {
        &self.lpos
    }
    #[doc = "0x10 - Position Difference Counter Register"]
    #[inline(always)]
    pub const fn posd(&self) -> &Posd {
        &self.posd
    }
    #[doc = "0x12 - Position Difference Hold Register"]
    #[inline(always)]
    pub const fn posdh(&self) -> &Posdh {
        &self.posdh
    }
    #[doc = "0x14 - Upper Position Hold Register"]
    #[inline(always)]
    pub const fn uposh(&self) -> &Uposh {
        &self.uposh
    }
    #[doc = "0x16 - Lower Position Hold Register"]
    #[inline(always)]
    pub const fn lposh(&self) -> &Lposh {
        &self.lposh
    }
    #[doc = "0x18 - Last Edge Time Hold Register"]
    #[inline(always)]
    pub const fn lastedgeh(&self) -> &Lastedgeh {
        &self.lastedgeh
    }
    #[doc = "0x1a - Position Difference Period Hold Register"]
    #[inline(always)]
    pub const fn posdperh(&self) -> &Posdperh {
        &self.posdperh
    }
    #[doc = "0x1c - Revolution Hold Register"]
    #[inline(always)]
    pub const fn revh(&self) -> &Revh {
        &self.revh
    }
    #[doc = "0x1e - Revolution Counter Register"]
    #[inline(always)]
    pub const fn rev(&self) -> &Rev {
        &self.rev
    }
    #[doc = "0x20 - Upper Initialization Register"]
    #[inline(always)]
    pub const fn uinit(&self) -> &Uinit {
        &self.uinit
    }
    #[doc = "0x22 - Lower Initialization Register"]
    #[inline(always)]
    pub const fn linit(&self) -> &Linit {
        &self.linit
    }
    #[doc = "0x24 - Upper Modulus Register"]
    #[inline(always)]
    pub const fn umod(&self) -> &Umod {
        &self.umod
    }
    #[doc = "0x26 - Lower Modulus Register"]
    #[inline(always)]
    pub const fn lmod(&self) -> &Lmod {
        &self.lmod
    }
    #[doc = "0x28 - Upper Position Compare Register 0"]
    #[inline(always)]
    pub const fn ucomp0(&self) -> &Ucomp0 {
        &self.ucomp0
    }
    #[doc = "0x2a - Lower Position Compare Register 0"]
    #[inline(always)]
    pub const fn lcomp0(&self) -> &Lcomp0 {
        &self.lcomp0
    }
    #[doc = "0x2c - Upper Position Holder Register 1"]
    #[inline(always)]
    pub const fn uposh1_uposh1(&self) -> &Uposh1Uposh1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(44).cast() }
    }
    #[doc = "0x2c - Upper Position Compare 1"]
    #[inline(always)]
    pub const fn ucomp1_ucomp1(&self) -> &Ucomp1Ucomp1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(44).cast() }
    }
    #[doc = "0x2e - Lower Position Holder Register 1"]
    #[inline(always)]
    pub const fn lposh1_lposh1(&self) -> &Lposh1Lposh1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(46).cast() }
    }
    #[doc = "0x2e - Lower Position Compare 1"]
    #[inline(always)]
    pub const fn lcomp1_lcomp1(&self) -> &Lcomp1Lcomp1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(46).cast() }
    }
    #[doc = "0x30 - Upper Position Holder Register 3"]
    #[inline(always)]
    pub const fn uposh2_uposh2(&self) -> &Uposh2Uposh2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(48).cast() }
    }
    #[doc = "0x30 - Upper Position Compare 2"]
    #[inline(always)]
    pub const fn ucomp2_ucomp2(&self) -> &Ucomp2Ucomp2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(48).cast() }
    }
    #[doc = "0x32 - Lower Position Holder Register 2"]
    #[inline(always)]
    pub const fn lposh2_lposh2(&self) -> &Lposh2Lposh2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(50).cast() }
    }
    #[doc = "0x32 - Lower Position Compare 2"]
    #[inline(always)]
    pub const fn lcomp2_lcomp2(&self) -> &Lcomp2Lcomp2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(50).cast() }
    }
    #[doc = "0x34 - Upper Position Holder Register 3"]
    #[inline(always)]
    pub const fn uposh3_uposh3(&self) -> &Uposh3Uposh3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(52).cast() }
    }
    #[doc = "0x34 - Upper Position Compare 3"]
    #[inline(always)]
    pub const fn ucomp3_ucomp3(&self) -> &Ucomp3Ucomp3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(52).cast() }
    }
    #[doc = "0x36 - Lower Position Holder Register 3"]
    #[inline(always)]
    pub const fn lposh3_lposh3(&self) -> &Lposh3Lposh3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(54).cast() }
    }
    #[doc = "0x36 - Lower Position Compare 3"]
    #[inline(always)]
    pub const fn lcomp3_lcomp3(&self) -> &Lcomp3Lcomp3 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(54).cast() }
    }
    #[doc = "0x38 - Interrupt Control Register"]
    #[inline(always)]
    pub const fn intctrl(&self) -> &Intctrl {
        &self.intctrl
    }
    #[doc = "0x3a - Watchdog Timeout Register"]
    #[inline(always)]
    pub const fn wtr(&self) -> &Wtr {
        &self.wtr
    }
    #[doc = "0x3c - Input Monitor Register"]
    #[inline(always)]
    pub const fn imr(&self) -> &Imr {
        &self.imr
    }
    #[doc = "0x3e - Test Register"]
    #[inline(always)]
    pub const fn tst(&self) -> &Tst {
        &self.tst
    }
    #[doc = "0x50 - Upper VERID"]
    #[inline(always)]
    pub const fn uverid(&self) -> &Uverid {
        &self.uverid
    }
    #[doc = "0x52 - Lower VERID"]
    #[inline(always)]
    pub const fn lverid(&self) -> &Lverid {
        &self.lverid
    }
}
#[doc = "CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Control Register"]
pub mod ctrl;
#[doc = "CTRL2 (rw) register accessor: Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl2`] module"]
#[doc(alias = "CTRL2")]
pub type Ctrl2 = crate::Reg<ctrl2::Ctrl2Spec>;
#[doc = "Control 2 Register"]
pub mod ctrl2;
#[doc = "FILT (rw) register accessor: Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`filt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`filt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@filt`] module"]
#[doc(alias = "FILT")]
pub type Filt = crate::Reg<filt::FiltSpec>;
#[doc = "Input Filter Register"]
pub mod filt;
#[doc = "LASTEDGE (r) register accessor: Last Edge Time Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lastedge::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lastedge`] module"]
#[doc(alias = "LASTEDGE")]
pub type Lastedge = crate::Reg<lastedge::LastedgeSpec>;
#[doc = "Last Edge Time Register"]
pub mod lastedge;
#[doc = "POSDPER (r) register accessor: Position Difference Period Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdper::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@posdper`] module"]
#[doc(alias = "POSDPER")]
pub type Posdper = crate::Reg<posdper::PosdperSpec>;
#[doc = "Position Difference Period Counter Register"]
pub mod posdper;
#[doc = "POSDPERBFR (r) register accessor: Position Difference Period Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdperbfr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@posdperbfr`] module"]
#[doc(alias = "POSDPERBFR")]
pub type Posdperbfr = crate::Reg<posdperbfr::PosdperbfrSpec>;
#[doc = "Position Difference Period Buffer Register"]
pub mod posdperbfr;
#[doc = "UPOS (rw) register accessor: Upper Position Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`upos::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`upos::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@upos`] module"]
#[doc(alias = "UPOS")]
pub type Upos = crate::Reg<upos::UposSpec>;
#[doc = "Upper Position Counter Register"]
pub mod upos;
#[doc = "LPOS (rw) register accessor: Lower Position Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lpos::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpos::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpos`] module"]
#[doc(alias = "LPOS")]
pub type Lpos = crate::Reg<lpos::LposSpec>;
#[doc = "Lower Position Counter Register"]
pub mod lpos;
#[doc = "POSD (rw) register accessor: Position Difference Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posd::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`posd::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@posd`] module"]
#[doc(alias = "POSD")]
pub type Posd = crate::Reg<posd::PosdSpec>;
#[doc = "Position Difference Counter Register"]
pub mod posd;
#[doc = "POSDH (r) register accessor: Position Difference Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@posdh`] module"]
#[doc(alias = "POSDH")]
pub type Posdh = crate::Reg<posdh::PosdhSpec>;
#[doc = "Position Difference Hold Register"]
pub mod posdh;
#[doc = "UPOSH (r) register accessor: Upper Position Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uposh`] module"]
#[doc(alias = "UPOSH")]
pub type Uposh = crate::Reg<uposh::UposhSpec>;
#[doc = "Upper Position Hold Register"]
pub mod uposh;
#[doc = "LPOSH (r) register accessor: Lower Position Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lposh`] module"]
#[doc(alias = "LPOSH")]
pub type Lposh = crate::Reg<lposh::LposhSpec>;
#[doc = "Lower Position Hold Register"]
pub mod lposh;
#[doc = "LASTEDGEH (r) register accessor: Last Edge Time Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lastedgeh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lastedgeh`] module"]
#[doc(alias = "LASTEDGEH")]
pub type Lastedgeh = crate::Reg<lastedgeh::LastedgehSpec>;
#[doc = "Last Edge Time Hold Register"]
pub mod lastedgeh;
#[doc = "POSDPERH (r) register accessor: Position Difference Period Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdperh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@posdperh`] module"]
#[doc(alias = "POSDPERH")]
pub type Posdperh = crate::Reg<posdperh::PosdperhSpec>;
#[doc = "Position Difference Period Hold Register"]
pub mod posdperh;
#[doc = "REVH (r) register accessor: Revolution Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`revh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@revh`] module"]
#[doc(alias = "REVH")]
pub type Revh = crate::Reg<revh::RevhSpec>;
#[doc = "Revolution Hold Register"]
pub mod revh;
#[doc = "REV (rw) register accessor: Revolution Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rev::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rev::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rev`] module"]
#[doc(alias = "REV")]
pub type Rev = crate::Reg<rev::RevSpec>;
#[doc = "Revolution Counter Register"]
pub mod rev;
#[doc = "UINIT (rw) register accessor: Upper Initialization Register\n\nYou can [`read`](crate::Reg::read) this register and get [`uinit::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`uinit::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uinit`] module"]
#[doc(alias = "UINIT")]
pub type Uinit = crate::Reg<uinit::UinitSpec>;
#[doc = "Upper Initialization Register"]
pub mod uinit;
#[doc = "LINIT (rw) register accessor: Lower Initialization Register\n\nYou can [`read`](crate::Reg::read) this register and get [`linit::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`linit::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@linit`] module"]
#[doc(alias = "LINIT")]
pub type Linit = crate::Reg<linit::LinitSpec>;
#[doc = "Lower Initialization Register"]
pub mod linit;
#[doc = "UMOD (rw) register accessor: Upper Modulus Register\n\nYou can [`read`](crate::Reg::read) this register and get [`umod::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`umod::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@umod`] module"]
#[doc(alias = "UMOD")]
pub type Umod = crate::Reg<umod::UmodSpec>;
#[doc = "Upper Modulus Register"]
pub mod umod;
#[doc = "LMOD (rw) register accessor: Lower Modulus Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lmod::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lmod::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lmod`] module"]
#[doc(alias = "LMOD")]
pub type Lmod = crate::Reg<lmod::LmodSpec>;
#[doc = "Lower Modulus Register"]
pub mod lmod;
#[doc = "UCOMP0 (rw) register accessor: Upper Position Compare Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`ucomp0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ucomp0`] module"]
#[doc(alias = "UCOMP0")]
pub type Ucomp0 = crate::Reg<ucomp0::Ucomp0Spec>;
#[doc = "Upper Position Compare Register 0"]
pub mod ucomp0;
#[doc = "LCOMP0 (rw) register accessor: Lower Position Compare Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcomp0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcomp0`] module"]
#[doc(alias = "LCOMP0")]
pub type Lcomp0 = crate::Reg<lcomp0::Lcomp0Spec>;
#[doc = "Lower Position Compare Register 0"]
pub mod lcomp0;
#[doc = "UCOMP1_UCOMP1 (w) register accessor: Upper Position Compare 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp1_ucomp1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ucomp1_ucomp1`] module"]
#[doc(alias = "UCOMP1_UCOMP1")]
pub type Ucomp1Ucomp1 = crate::Reg<ucomp1_ucomp1::Ucomp1Ucomp1Spec>;
#[doc = "Upper Position Compare 1"]
pub mod ucomp1_ucomp1;
#[doc = "UPOSH1_UPOSH1 (r) register accessor: Upper Position Holder Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh1_uposh1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uposh1_uposh1`] module"]
#[doc(alias = "UPOSH1_UPOSH1")]
pub type Uposh1Uposh1 = crate::Reg<uposh1_uposh1::Uposh1Uposh1Spec>;
#[doc = "Upper Position Holder Register 1"]
pub mod uposh1_uposh1;
#[doc = "LCOMP1_LCOMP1 (w) register accessor: Lower Position Compare 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp1_lcomp1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcomp1_lcomp1`] module"]
#[doc(alias = "LCOMP1_LCOMP1")]
pub type Lcomp1Lcomp1 = crate::Reg<lcomp1_lcomp1::Lcomp1Lcomp1Spec>;
#[doc = "Lower Position Compare 1"]
pub mod lcomp1_lcomp1;
#[doc = "LPOSH1_LPOSH1 (r) register accessor: Lower Position Holder Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh1_lposh1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lposh1_lposh1`] module"]
#[doc(alias = "LPOSH1_LPOSH1")]
pub type Lposh1Lposh1 = crate::Reg<lposh1_lposh1::Lposh1Lposh1Spec>;
#[doc = "Lower Position Holder Register 1"]
pub mod lposh1_lposh1;
#[doc = "UCOMP2_UCOMP2 (w) register accessor: Upper Position Compare 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp2_ucomp2::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ucomp2_ucomp2`] module"]
#[doc(alias = "UCOMP2_UCOMP2")]
pub type Ucomp2Ucomp2 = crate::Reg<ucomp2_ucomp2::Ucomp2Ucomp2Spec>;
#[doc = "Upper Position Compare 2"]
pub mod ucomp2_ucomp2;
#[doc = "UPOSH2_UPOSH2 (r) register accessor: Upper Position Holder Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh2_uposh2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uposh2_uposh2`] module"]
#[doc(alias = "UPOSH2_UPOSH2")]
pub type Uposh2Uposh2 = crate::Reg<uposh2_uposh2::Uposh2Uposh2Spec>;
#[doc = "Upper Position Holder Register 3"]
pub mod uposh2_uposh2;
#[doc = "LCOMP2_LCOMP2 (w) register accessor: Lower Position Compare 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp2_lcomp2::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcomp2_lcomp2`] module"]
#[doc(alias = "LCOMP2_LCOMP2")]
pub type Lcomp2Lcomp2 = crate::Reg<lcomp2_lcomp2::Lcomp2Lcomp2Spec>;
#[doc = "Lower Position Compare 2"]
pub mod lcomp2_lcomp2;
#[doc = "LPOSH2_LPOSH2 (r) register accessor: Lower Position Holder Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh2_lposh2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lposh2_lposh2`] module"]
#[doc(alias = "LPOSH2_LPOSH2")]
pub type Lposh2Lposh2 = crate::Reg<lposh2_lposh2::Lposh2Lposh2Spec>;
#[doc = "Lower Position Holder Register 2"]
pub mod lposh2_lposh2;
#[doc = "UCOMP3_UCOMP3 (w) register accessor: Upper Position Compare 3\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp3_ucomp3::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ucomp3_ucomp3`] module"]
#[doc(alias = "UCOMP3_UCOMP3")]
pub type Ucomp3Ucomp3 = crate::Reg<ucomp3_ucomp3::Ucomp3Ucomp3Spec>;
#[doc = "Upper Position Compare 3"]
pub mod ucomp3_ucomp3;
#[doc = "UPOSH3_UPOSH3 (r) register accessor: Upper Position Holder Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh3_uposh3::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uposh3_uposh3`] module"]
#[doc(alias = "UPOSH3_UPOSH3")]
pub type Uposh3Uposh3 = crate::Reg<uposh3_uposh3::Uposh3Uposh3Spec>;
#[doc = "Upper Position Holder Register 3"]
pub mod uposh3_uposh3;
#[doc = "LCOMP3_LCOMP3 (w) register accessor: Lower Position Compare 3\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp3_lcomp3::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcomp3_lcomp3`] module"]
#[doc(alias = "LCOMP3_LCOMP3")]
pub type Lcomp3Lcomp3 = crate::Reg<lcomp3_lcomp3::Lcomp3Lcomp3Spec>;
#[doc = "Lower Position Compare 3"]
pub mod lcomp3_lcomp3;
#[doc = "LPOSH3_LPOSH3 (r) register accessor: Lower Position Holder Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh3_lposh3::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lposh3_lposh3`] module"]
#[doc(alias = "LPOSH3_LPOSH3")]
pub type Lposh3Lposh3 = crate::Reg<lposh3_lposh3::Lposh3Lposh3Spec>;
#[doc = "Lower Position Holder Register 3"]
pub mod lposh3_lposh3;
#[doc = "INTCTRL (rw) register accessor: Interrupt Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`intctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`intctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@intctrl`] module"]
#[doc(alias = "INTCTRL")]
pub type Intctrl = crate::Reg<intctrl::IntctrlSpec>;
#[doc = "Interrupt Control Register"]
pub mod intctrl;
#[doc = "WTR (rw) register accessor: Watchdog Timeout Register\n\nYou can [`read`](crate::Reg::read) this register and get [`wtr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wtr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wtr`] module"]
#[doc(alias = "WTR")]
pub type Wtr = crate::Reg<wtr::WtrSpec>;
#[doc = "Watchdog Timeout Register"]
pub mod wtr;
#[doc = "IMR (rw) register accessor: Input Monitor Register\n\nYou can [`read`](crate::Reg::read) this register and get [`imr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`imr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@imr`] module"]
#[doc(alias = "IMR")]
pub type Imr = crate::Reg<imr::ImrSpec>;
#[doc = "Input Monitor Register"]
pub mod imr;
#[doc = "TST (rw) register accessor: Test Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tst`] module"]
#[doc(alias = "TST")]
pub type Tst = crate::Reg<tst::TstSpec>;
#[doc = "Test Register"]
pub mod tst;
#[doc = "UVERID (r) register accessor: Upper VERID\n\nYou can [`read`](crate::Reg::read) this register and get [`uverid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@uverid`] module"]
#[doc(alias = "UVERID")]
pub type Uverid = crate::Reg<uverid::UveridSpec>;
#[doc = "Upper VERID"]
pub mod uverid;
#[doc = "LVERID (r) register accessor: Lower VERID\n\nYou can [`read`](crate::Reg::read) this register and get [`lverid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lverid`] module"]
#[doc(alias = "LVERID")]
pub type Lverid = crate::Reg<lverid::LveridSpec>;
#[doc = "Lower VERID"]
pub mod lverid;
