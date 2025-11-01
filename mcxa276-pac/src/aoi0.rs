#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    bfcrt010: Bfcrt010,
    bfcrt230: Bfcrt230,
    bfcrt011: Bfcrt011,
    bfcrt231: Bfcrt231,
    bfcrt012: Bfcrt012,
    bfcrt232: Bfcrt232,
    bfcrt013: Bfcrt013,
    bfcrt233: Bfcrt233,
}
impl RegisterBlock {
    #[doc = "0x00 - Boolean Function Term 0 and 1 Configuration for EVENT0"]
    #[inline(always)]
    pub const fn bfcrt010(&self) -> &Bfcrt010 {
        &self.bfcrt010
    }
    #[doc = "0x02 - Boolean Function Term 2 and 3 Configuration for EVENT0"]
    #[inline(always)]
    pub const fn bfcrt230(&self) -> &Bfcrt230 {
        &self.bfcrt230
    }
    #[doc = "0x04 - Boolean Function Term 0 and 1 Configuration for EVENT1"]
    #[inline(always)]
    pub const fn bfcrt011(&self) -> &Bfcrt011 {
        &self.bfcrt011
    }
    #[doc = "0x06 - Boolean Function Term 2 and 3 Configuration for EVENT1"]
    #[inline(always)]
    pub const fn bfcrt231(&self) -> &Bfcrt231 {
        &self.bfcrt231
    }
    #[doc = "0x08 - Boolean Function Term 0 and 1 Configuration for EVENT2"]
    #[inline(always)]
    pub const fn bfcrt012(&self) -> &Bfcrt012 {
        &self.bfcrt012
    }
    #[doc = "0x0a - Boolean Function Term 2 and 3 Configuration for EVENT2"]
    #[inline(always)]
    pub const fn bfcrt232(&self) -> &Bfcrt232 {
        &self.bfcrt232
    }
    #[doc = "0x0c - Boolean Function Term 0 and 1 Configuration for EVENT3"]
    #[inline(always)]
    pub const fn bfcrt013(&self) -> &Bfcrt013 {
        &self.bfcrt013
    }
    #[doc = "0x0e - Boolean Function Term 2 and 3 Configuration for EVENT3"]
    #[inline(always)]
    pub const fn bfcrt233(&self) -> &Bfcrt233 {
        &self.bfcrt233
    }
}
#[doc = "BFCRT010 (rw) register accessor: Boolean Function Term 0 and 1 Configuration for EVENT0\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt010::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt010::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt010`] module"]
#[doc(alias = "BFCRT010")]
pub type Bfcrt010 = crate::Reg<bfcrt010::Bfcrt010Spec>;
#[doc = "Boolean Function Term 0 and 1 Configuration for EVENT0"]
pub mod bfcrt010;
#[doc = "BFCRT230 (rw) register accessor: Boolean Function Term 2 and 3 Configuration for EVENT0\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt230::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt230::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt230`] module"]
#[doc(alias = "BFCRT230")]
pub type Bfcrt230 = crate::Reg<bfcrt230::Bfcrt230Spec>;
#[doc = "Boolean Function Term 2 and 3 Configuration for EVENT0"]
pub mod bfcrt230;
#[doc = "BFCRT011 (rw) register accessor: Boolean Function Term 0 and 1 Configuration for EVENT1\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt011::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt011::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt011`] module"]
#[doc(alias = "BFCRT011")]
pub type Bfcrt011 = crate::Reg<bfcrt011::Bfcrt011Spec>;
#[doc = "Boolean Function Term 0 and 1 Configuration for EVENT1"]
pub mod bfcrt011;
#[doc = "BFCRT231 (rw) register accessor: Boolean Function Term 2 and 3 Configuration for EVENT1\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt231::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt231::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt231`] module"]
#[doc(alias = "BFCRT231")]
pub type Bfcrt231 = crate::Reg<bfcrt231::Bfcrt231Spec>;
#[doc = "Boolean Function Term 2 and 3 Configuration for EVENT1"]
pub mod bfcrt231;
#[doc = "BFCRT012 (rw) register accessor: Boolean Function Term 0 and 1 Configuration for EVENT2\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt012::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt012::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt012`] module"]
#[doc(alias = "BFCRT012")]
pub type Bfcrt012 = crate::Reg<bfcrt012::Bfcrt012Spec>;
#[doc = "Boolean Function Term 0 and 1 Configuration for EVENT2"]
pub mod bfcrt012;
#[doc = "BFCRT232 (rw) register accessor: Boolean Function Term 2 and 3 Configuration for EVENT2\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt232::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt232::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt232`] module"]
#[doc(alias = "BFCRT232")]
pub type Bfcrt232 = crate::Reg<bfcrt232::Bfcrt232Spec>;
#[doc = "Boolean Function Term 2 and 3 Configuration for EVENT2"]
pub mod bfcrt232;
#[doc = "BFCRT013 (rw) register accessor: Boolean Function Term 0 and 1 Configuration for EVENT3\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt013::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt013::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt013`] module"]
#[doc(alias = "BFCRT013")]
pub type Bfcrt013 = crate::Reg<bfcrt013::Bfcrt013Spec>;
#[doc = "Boolean Function Term 0 and 1 Configuration for EVENT3"]
pub mod bfcrt013;
#[doc = "BFCRT233 (rw) register accessor: Boolean Function Term 2 and 3 Configuration for EVENT3\n\nYou can [`read`](crate::Reg::read) this register and get [`bfcrt233::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bfcrt233::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bfcrt233`] module"]
#[doc(alias = "BFCRT233")]
pub type Bfcrt233 = crate::Reg<bfcrt233::Bfcrt233Spec>;
#[doc = "Boolean Function Term 2 and 3 Configuration for EVENT3"]
pub mod bfcrt233;
