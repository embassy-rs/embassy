#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mod_: Mod,
    tc: Tc,
    feed: Feed,
    tv: Tv,
    _reserved4: [u8; 0x04],
    warnint: Warnint,
    window: Window,
}
impl RegisterBlock {
    #[doc = "0x00 - Mode"]
    #[inline(always)]
    pub const fn mod_(&self) -> &Mod {
        &self.mod_
    }
    #[doc = "0x04 - Timer Constant"]
    #[inline(always)]
    pub const fn tc(&self) -> &Tc {
        &self.tc
    }
    #[doc = "0x08 - Feed Sequence"]
    #[inline(always)]
    pub const fn feed(&self) -> &Feed {
        &self.feed
    }
    #[doc = "0x0c - Timer Value"]
    #[inline(always)]
    pub const fn tv(&self) -> &Tv {
        &self.tv
    }
    #[doc = "0x14 - Warning Interrupt Compare Value"]
    #[inline(always)]
    pub const fn warnint(&self) -> &Warnint {
        &self.warnint
    }
    #[doc = "0x18 - Window Compare Value"]
    #[inline(always)]
    pub const fn window(&self) -> &Window {
        &self.window
    }
}
#[doc = "MOD (rw) register accessor: Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mod_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mod_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mod_`] module"]
#[doc(alias = "MOD")]
pub type Mod = crate::Reg<mod_::ModSpec>;
#[doc = "Mode"]
pub mod mod_;
#[doc = "TC (rw) register accessor: Timer Constant\n\nYou can [`read`](crate::Reg::read) this register and get [`tc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tc`] module"]
#[doc(alias = "TC")]
pub type Tc = crate::Reg<tc::TcSpec>;
#[doc = "Timer Constant"]
pub mod tc;
#[doc = "FEED (w) register accessor: Feed Sequence\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`feed::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@feed`] module"]
#[doc(alias = "FEED")]
pub type Feed = crate::Reg<feed::FeedSpec>;
#[doc = "Feed Sequence"]
pub mod feed;
#[doc = "TV (r) register accessor: Timer Value\n\nYou can [`read`](crate::Reg::read) this register and get [`tv::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tv`] module"]
#[doc(alias = "TV")]
pub type Tv = crate::Reg<tv::TvSpec>;
#[doc = "Timer Value"]
pub mod tv;
#[doc = "WARNINT (rw) register accessor: Warning Interrupt Compare Value\n\nYou can [`read`](crate::Reg::read) this register and get [`warnint::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`warnint::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@warnint`] module"]
#[doc(alias = "WARNINT")]
pub type Warnint = crate::Reg<warnint::WarnintSpec>;
#[doc = "Warning Interrupt Compare Value"]
pub mod warnint;
#[doc = "WINDOW (rw) register accessor: Window Compare Value\n\nYou can [`read`](crate::Reg::read) this register and get [`window::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`window::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@window`] module"]
#[doc(alias = "WINDOW")]
pub type Window = crate::Reg<window::WindowSpec>;
#[doc = "Window Compare Value"]
pub mod window;
