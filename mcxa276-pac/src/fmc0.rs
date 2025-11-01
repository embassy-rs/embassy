#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x20],
    remap: Remap,
}
impl RegisterBlock {
    #[doc = "0x20 - Data Remap"]
    #[inline(always)]
    pub const fn remap(&self) -> &Remap {
        &self.remap
    }
}
#[doc = "REMAP (rw) register accessor: Data Remap\n\nYou can [`read`](crate::Reg::read) this register and get [`remap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`remap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@remap`] module"]
#[doc(alias = "REMAP")]
pub type Remap = crate::Reg<remap::RemapSpec>;
#[doc = "Data Remap"]
pub mod remap;
