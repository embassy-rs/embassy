#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    evtimerl: Evtimerl,
    evtimerh: Evtimerh,
    capture_l: CaptureL,
    capture_h: CaptureH,
    match_l: MatchL,
    match_h: MatchH,
    _reserved6: [u8; 0x04],
    osevent_ctrl: OseventCtrl,
}
impl RegisterBlock {
    #[doc = "0x00 - EVTIMER Low"]
    #[inline(always)]
    pub const fn evtimerl(&self) -> &Evtimerl {
        &self.evtimerl
    }
    #[doc = "0x04 - EVTIMER High"]
    #[inline(always)]
    pub const fn evtimerh(&self) -> &Evtimerh {
        &self.evtimerh
    }
    #[doc = "0x08 - Local Capture Low for CPU"]
    #[inline(always)]
    pub const fn capture_l(&self) -> &CaptureL {
        &self.capture_l
    }
    #[doc = "0x0c - Local Capture High for CPU"]
    #[inline(always)]
    pub const fn capture_h(&self) -> &CaptureH {
        &self.capture_h
    }
    #[doc = "0x10 - Local Match Low for CPU"]
    #[inline(always)]
    pub const fn match_l(&self) -> &MatchL {
        &self.match_l
    }
    #[doc = "0x14 - Local Match High for CPU"]
    #[inline(always)]
    pub const fn match_h(&self) -> &MatchH {
        &self.match_h
    }
    #[doc = "0x1c - OSTIMER Control for CPU"]
    #[inline(always)]
    pub const fn osevent_ctrl(&self) -> &OseventCtrl {
        &self.osevent_ctrl
    }
}
#[doc = "EVTIMERL (r) register accessor: EVTIMER Low\n\nYou can [`read`](crate::Reg::read) this register and get [`evtimerl::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@evtimerl`] module"]
#[doc(alias = "EVTIMERL")]
pub type Evtimerl = crate::Reg<evtimerl::EvtimerlSpec>;
#[doc = "EVTIMER Low"]
pub mod evtimerl;
#[doc = "EVTIMERH (r) register accessor: EVTIMER High\n\nYou can [`read`](crate::Reg::read) this register and get [`evtimerh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@evtimerh`] module"]
#[doc(alias = "EVTIMERH")]
pub type Evtimerh = crate::Reg<evtimerh::EvtimerhSpec>;
#[doc = "EVTIMER High"]
pub mod evtimerh;
#[doc = "CAPTURE_L (r) register accessor: Local Capture Low for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`capture_l::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@capture_l`] module"]
#[doc(alias = "CAPTURE_L")]
pub type CaptureL = crate::Reg<capture_l::CaptureLSpec>;
#[doc = "Local Capture Low for CPU"]
pub mod capture_l;
#[doc = "CAPTURE_H (r) register accessor: Local Capture High for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`capture_h::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@capture_h`] module"]
#[doc(alias = "CAPTURE_H")]
pub type CaptureH = crate::Reg<capture_h::CaptureHSpec>;
#[doc = "Local Capture High for CPU"]
pub mod capture_h;
#[doc = "MATCH_L (rw) register accessor: Local Match Low for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`match_l::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_l::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@match_l`] module"]
#[doc(alias = "MATCH_L")]
pub type MatchL = crate::Reg<match_l::MatchLSpec>;
#[doc = "Local Match Low for CPU"]
pub mod match_l;
#[doc = "MATCH_H (rw) register accessor: Local Match High for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`match_h::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_h::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@match_h`] module"]
#[doc(alias = "MATCH_H")]
pub type MatchH = crate::Reg<match_h::MatchHSpec>;
#[doc = "Local Match High for CPU"]
pub mod match_h;
#[doc = "OSEVENT_CTRL (rw) register accessor: OSTIMER Control for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`osevent_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`osevent_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@osevent_ctrl`] module"]
#[doc(alias = "OSEVENT_CTRL")]
pub type OseventCtrl = crate::Reg<osevent_ctrl::OseventCtrlSpec>;
#[doc = "OSTIMER Control for CPU"]
pub mod osevent_ctrl;
