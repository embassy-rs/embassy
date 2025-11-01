#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mp_csr: MpCsr,
    mp_es: MpEs,
    mp_int: MpInt,
    mp_hrs: MpHrs,
    _reserved4: [u8; 0xf0],
    ch_grpri: [ChGrpri; 8],
}
impl RegisterBlock {
    #[doc = "0x00 - Management Page Control"]
    #[inline(always)]
    pub const fn mp_csr(&self) -> &MpCsr {
        &self.mp_csr
    }
    #[doc = "0x04 - Management Page Error Status"]
    #[inline(always)]
    pub const fn mp_es(&self) -> &MpEs {
        &self.mp_es
    }
    #[doc = "0x08 - Management Page Interrupt Request Status"]
    #[inline(always)]
    pub const fn mp_int(&self) -> &MpInt {
        &self.mp_int
    }
    #[doc = "0x0c - Management Page Hardware Request Status"]
    #[inline(always)]
    pub const fn mp_hrs(&self) -> &MpHrs {
        &self.mp_hrs
    }
    #[doc = "0x100..0x120 - Channel Arbitration Group"]
    #[inline(always)]
    pub const fn ch_grpri(&self, n: usize) -> &ChGrpri {
        &self.ch_grpri[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x100..0x120 - Channel Arbitration Group"]
    #[inline(always)]
    pub fn ch_grpri_iter(&self) -> impl Iterator<Item = &ChGrpri> {
        self.ch_grpri.iter()
    }
}
#[doc = "MP_CSR (rw) register accessor: Management Page Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_csr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mp_csr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mp_csr`] module"]
#[doc(alias = "MP_CSR")]
pub type MpCsr = crate::Reg<mp_csr::MpCsrSpec>;
#[doc = "Management Page Control"]
pub mod mp_csr;
#[doc = "MP_ES (r) register accessor: Management Page Error Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_es::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mp_es`] module"]
#[doc(alias = "MP_ES")]
pub type MpEs = crate::Reg<mp_es::MpEsSpec>;
#[doc = "Management Page Error Status"]
pub mod mp_es;
#[doc = "MP_INT (r) register accessor: Management Page Interrupt Request Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_int::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mp_int`] module"]
#[doc(alias = "MP_INT")]
pub type MpInt = crate::Reg<mp_int::MpIntSpec>;
#[doc = "Management Page Interrupt Request Status"]
pub mod mp_int;
#[doc = "MP_HRS (r) register accessor: Management Page Hardware Request Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_hrs::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mp_hrs`] module"]
#[doc(alias = "MP_HRS")]
pub type MpHrs = crate::Reg<mp_hrs::MpHrsSpec>;
#[doc = "Management Page Hardware Request Status"]
pub mod mp_hrs;
#[doc = "CH_GRPRI (rw) register accessor: Channel Arbitration Group\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_grpri::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_grpri::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_grpri`] module"]
#[doc(alias = "CH_GRPRI")]
pub type ChGrpri = crate::Reg<ch_grpri::ChGrpriSpec>;
#[doc = "Channel Arbitration Group"]
pub mod ch_grpri;
