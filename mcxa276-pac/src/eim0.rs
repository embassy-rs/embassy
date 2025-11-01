#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    eimcr: Eimcr,
    eichen: Eichen,
    _reserved2: [u8; 0xf8],
    eichd0_word0: Eichd0Word0,
    eichd0_word1: Eichd0Word1,
}
impl RegisterBlock {
    #[doc = "0x00 - Error Injection Module Configuration Register"]
    #[inline(always)]
    pub const fn eimcr(&self) -> &Eimcr {
        &self.eimcr
    }
    #[doc = "0x04 - Error Injection Channel Enable register"]
    #[inline(always)]
    pub const fn eichen(&self) -> &Eichen {
        &self.eichen
    }
    #[doc = "0x100 - Error Injection Channel Descriptor 0, Word0"]
    #[inline(always)]
    pub const fn eichd0_word0(&self) -> &Eichd0Word0 {
        &self.eichd0_word0
    }
    #[doc = "0x104 - Error Injection Channel Descriptor 0, Word1"]
    #[inline(always)]
    pub const fn eichd0_word1(&self) -> &Eichd0Word1 {
        &self.eichd0_word1
    }
}
#[doc = "EIMCR (rw) register accessor: Error Injection Module Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`eimcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eimcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@eimcr`] module"]
#[doc(alias = "EIMCR")]
pub type Eimcr = crate::Reg<eimcr::EimcrSpec>;
#[doc = "Error Injection Module Configuration Register"]
pub mod eimcr;
#[doc = "EICHEN (rw) register accessor: Error Injection Channel Enable register\n\nYou can [`read`](crate::Reg::read) this register and get [`eichen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@eichen`] module"]
#[doc(alias = "EICHEN")]
pub type Eichen = crate::Reg<eichen::EichenSpec>;
#[doc = "Error Injection Channel Enable register"]
pub mod eichen;
#[doc = "EICHD0_WORD0 (rw) register accessor: Error Injection Channel Descriptor 0, Word0\n\nYou can [`read`](crate::Reg::read) this register and get [`eichd0_word0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichd0_word0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@eichd0_word0`] module"]
#[doc(alias = "EICHD0_WORD0")]
pub type Eichd0Word0 = crate::Reg<eichd0_word0::Eichd0Word0Spec>;
#[doc = "Error Injection Channel Descriptor 0, Word0"]
pub mod eichd0_word0;
#[doc = "EICHD0_WORD1 (rw) register accessor: Error Injection Channel Descriptor 0, Word1\n\nYou can [`read`](crate::Reg::read) this register and get [`eichd0_word1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichd0_word1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@eichd0_word1`] module"]
#[doc(alias = "EICHD0_WORD1")]
pub type Eichd0Word1 = crate::Reg<eichd0_word1::Eichd0Word1Spec>;
#[doc = "Error Injection Channel Descriptor 0, Word1"]
pub mod eichd0_word1;
