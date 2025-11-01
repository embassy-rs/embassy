#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x10],
    cr: Cr,
    sr: Sr,
    lr: Lr,
    ier: Ier,
    tsr: Tsr,
    ter: Ter,
    _reserved6: [u8; 0x04],
    ppr: Ppr,
    _reserved7: [u8; 0x10],
    pgfr: [Pgfr; 6],
}
impl RegisterBlock {
    #[doc = "0x10 - Control"]
    #[inline(always)]
    pub const fn cr(&self) -> &Cr {
        &self.cr
    }
    #[doc = "0x14 - Status"]
    #[inline(always)]
    pub const fn sr(&self) -> &Sr {
        &self.sr
    }
    #[doc = "0x18 - Lock"]
    #[inline(always)]
    pub const fn lr(&self) -> &Lr {
        &self.lr
    }
    #[doc = "0x1c - Interrupt Enable"]
    #[inline(always)]
    pub const fn ier(&self) -> &Ier {
        &self.ier
    }
    #[doc = "0x20 - Tamper Seconds"]
    #[inline(always)]
    pub const fn tsr(&self) -> &Tsr {
        &self.tsr
    }
    #[doc = "0x24 - Tamper Enable"]
    #[inline(always)]
    pub const fn ter(&self) -> &Ter {
        &self.ter
    }
    #[doc = "0x2c - Pin Polarity"]
    #[inline(always)]
    pub const fn ppr(&self) -> &Ppr {
        &self.ppr
    }
    #[doc = "0x40..0x58 - Pin Glitch Filter"]
    #[inline(always)]
    pub const fn pgfr(&self, n: usize) -> &Pgfr {
        &self.pgfr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x40..0x58 - Pin Glitch Filter"]
    #[inline(always)]
    pub fn pgfr_iter(&self) -> impl Iterator<Item = &Pgfr> {
        self.pgfr.iter()
    }
}
#[doc = "CR (rw) register accessor: Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cr`] module"]
#[doc(alias = "CR")]
pub type Cr = crate::Reg<cr::CrSpec>;
#[doc = "Control"]
pub mod cr;
#[doc = "SR (rw) register accessor: Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sr`] module"]
#[doc(alias = "SR")]
pub type Sr = crate::Reg<sr::SrSpec>;
#[doc = "Status"]
pub mod sr;
#[doc = "LR (rw) register accessor: Lock\n\nYou can [`read`](crate::Reg::read) this register and get [`lr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lr`] module"]
#[doc(alias = "LR")]
pub type Lr = crate::Reg<lr::LrSpec>;
#[doc = "Lock"]
pub mod lr;
#[doc = "IER (rw) register accessor: Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ier`] module"]
#[doc(alias = "IER")]
pub type Ier = crate::Reg<ier::IerSpec>;
#[doc = "Interrupt Enable"]
pub mod ier;
#[doc = "TSR (rw) register accessor: Tamper Seconds\n\nYou can [`read`](crate::Reg::read) this register and get [`tsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tsr`] module"]
#[doc(alias = "TSR")]
pub type Tsr = crate::Reg<tsr::TsrSpec>;
#[doc = "Tamper Seconds"]
pub mod tsr;
#[doc = "TER (rw) register accessor: Tamper Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ter::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ter::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ter`] module"]
#[doc(alias = "TER")]
pub type Ter = crate::Reg<ter::TerSpec>;
#[doc = "Tamper Enable"]
pub mod ter;
#[doc = "PPR (rw) register accessor: Pin Polarity\n\nYou can [`read`](crate::Reg::read) this register and get [`ppr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ppr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ppr`] module"]
#[doc(alias = "PPR")]
pub type Ppr = crate::Reg<ppr::PprSpec>;
#[doc = "Pin Polarity"]
pub mod ppr;
#[doc = "PGFR (rw) register accessor: Pin Glitch Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`pgfr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pgfr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pgfr`] module"]
#[doc(alias = "PGFR")]
pub type Pgfr = crate::Reg<pgfr::PgfrSpec>;
#[doc = "Pin Glitch Filter"]
pub mod pgfr;
