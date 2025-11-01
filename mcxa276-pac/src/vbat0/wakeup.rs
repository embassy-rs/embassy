#[repr(C)]
#[doc = "Array of registers: WAKEUPA"]
#[doc(alias = "WAKEUP")]
pub struct Wakeup {
    wakeupa: Wakeupa,
}
impl Wakeup {
    #[doc = "0x00 - Wakeup 0 Register A"]
    #[inline(always)]
    pub const fn wakeupa(&self) -> &Wakeupa {
        &self.wakeupa
    }
}
#[doc = "WAKEUPA (rw) register accessor: Wakeup 0 Register A\n\nYou can [`read`](crate::Reg::read) this register and get [`wakeupa::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`wakeupa::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wakeupa`] module"]
#[doc(alias = "WAKEUPA")]
pub type Wakeupa = crate::Reg<wakeupa::WakeupaSpec>;
#[doc = "Wakeup 0 Register A"]
pub mod wakeupa;
