#[repr(C)]
#[doc = "Array of registers: ENDPT"]
#[doc(alias = "ENDPOINT")]
pub struct Endpoint {
    endpt: Endpt,
}
impl Endpoint {
    #[doc = "0x00 - Endpoint Control"]
    #[inline(always)]
    pub const fn endpt(&self) -> &Endpt {
        &self.endpt
    }
}
#[doc = "ENDPT (rw) register accessor: Endpoint Control\n\nYou can [`read`](crate::Reg::read) this register and get [`endpt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`endpt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@endpt`] module"]
#[doc(alias = "ENDPT")]
pub type Endpt = crate::Reg<endpt::EndptSpec>;
#[doc = "Endpoint Control"]
pub mod endpt;
