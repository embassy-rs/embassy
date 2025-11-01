#[repr(C)]
#[doc = "Array of registers: CH_CSR, CH_ES, CH_INT, CH_MUX, CH_PRI, CH_SBR, TCD_ATTR, TCD_BITER_ELINKNO, TCD_BITER_ELINKYES, TCD_CITER_ELINKNO, TCD_CITER_ELINKYES, TCD_CSR, TCD_DADDR, TCD_DLAST_SGA, TCD_DOFF, TCD_NBYTES_MLOFFNO, TCD_NBYTES_MLOFFYES, TCD_SADDR, TCD_SLAST_SDA, TCD_SOFF"]
#[doc(alias = "TCD")]
pub struct Tcd {
    ch_csr: ChCsr,
    ch_es: ChEs,
    ch_int: ChInt,
    ch_sbr: ChSbr,
    ch_pri: ChPri,
    ch_mux: ChMux,
    _reserved6: [u8; 0x08],
    tcd_saddr: TcdSaddr,
    tcd_soff: TcdSoff,
    tcd_attr: TcdAttr,
    _reserved_9_mloffno_tcd_nbytes_mloffno: [u8; 0x04],
    tcd_slast_sda: TcdSlastSda,
    tcd_daddr: TcdDaddr,
    tcd_doff: TcdDoff,
    _reserved_13_elinkno_tcd_citer_elinkno: [u8; 0x02],
    tcd_dlast_sga: TcdDlastSga,
    tcd_csr: TcdCsr,
    _reserved_16_elinkno_tcd_biter_elinkno: [u8; 0x02],
}
impl Tcd {
    #[doc = "0x00 - Channel Control and Status"]
    #[inline(always)]
    pub const fn ch_csr(&self) -> &ChCsr {
        &self.ch_csr
    }
    #[doc = "0x04 - Channel Error Status"]
    #[inline(always)]
    pub const fn ch_es(&self) -> &ChEs {
        &self.ch_es
    }
    #[doc = "0x08 - Channel Interrupt Status"]
    #[inline(always)]
    pub const fn ch_int(&self) -> &ChInt {
        &self.ch_int
    }
    #[doc = "0x0c - Channel System Bus"]
    #[inline(always)]
    pub const fn ch_sbr(&self) -> &ChSbr {
        &self.ch_sbr
    }
    #[doc = "0x10 - Channel Priority"]
    #[inline(always)]
    pub const fn ch_pri(&self) -> &ChPri {
        &self.ch_pri
    }
    #[doc = "0x14 - Channel Multiplexor Configuration"]
    #[inline(always)]
    pub const fn ch_mux(&self) -> &ChMux {
        &self.ch_mux
    }
    #[doc = "0x20 - TCD Source Address"]
    #[inline(always)]
    pub const fn tcd_saddr(&self) -> &TcdSaddr {
        &self.tcd_saddr
    }
    #[doc = "0x24 - TCD Signed Source Address Offset"]
    #[inline(always)]
    pub const fn tcd_soff(&self) -> &TcdSoff {
        &self.tcd_soff
    }
    #[doc = "0x26 - TCD Transfer Attributes"]
    #[inline(always)]
    pub const fn tcd_attr(&self) -> &TcdAttr {
        &self.tcd_attr
    }
    #[doc = "0x28 - TCD Transfer Size with Minor Loop Offsets"]
    #[inline(always)]
    pub const fn mloffyes_tcd_nbytes_mloffyes(&self) -> &MloffyesTcdNbytesMloffyes {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(40).cast() }
    }
    #[doc = "0x28 - TCD Transfer Size Without Minor Loop Offsets"]
    #[inline(always)]
    pub const fn mloffno_tcd_nbytes_mloffno(&self) -> &MloffnoTcdNbytesMloffno {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(40).cast() }
    }
    #[doc = "0x2c - TCD Last Source Address Adjustment / Store DADDR Address"]
    #[inline(always)]
    pub const fn tcd_slast_sda(&self) -> &TcdSlastSda {
        &self.tcd_slast_sda
    }
    #[doc = "0x30 - TCD Destination Address"]
    #[inline(always)]
    pub const fn tcd_daddr(&self) -> &TcdDaddr {
        &self.tcd_daddr
    }
    #[doc = "0x34 - TCD Signed Destination Address Offset"]
    #[inline(always)]
    pub const fn tcd_doff(&self) -> &TcdDoff {
        &self.tcd_doff
    }
    #[doc = "0x36 - TCD Current Major Loop Count (Minor Loop Channel Linking Enabled)"]
    #[inline(always)]
    pub const fn elinkyes_tcd_citer_elinkyes(&self) -> &ElinkyesTcdCiterElinkyes {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(54).cast() }
    }
    #[doc = "0x36 - TCD Current Major Loop Count (Minor Loop Channel Linking Disabled)"]
    #[inline(always)]
    pub const fn elinkno_tcd_citer_elinkno(&self) -> &ElinknoTcdCiterElinkno {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(54).cast() }
    }
    #[doc = "0x38 - TCD Last Destination Address Adjustment / Scatter Gather Address"]
    #[inline(always)]
    pub const fn tcd_dlast_sga(&self) -> &TcdDlastSga {
        &self.tcd_dlast_sga
    }
    #[doc = "0x3c - TCD Control and Status"]
    #[inline(always)]
    pub const fn tcd_csr(&self) -> &TcdCsr {
        &self.tcd_csr
    }
    #[doc = "0x3e - TCD Beginning Major Loop Count (Minor Loop Channel Linking Enabled)"]
    #[inline(always)]
    pub const fn elinkyes_tcd_biter_elinkyes(&self) -> &ElinkyesTcdBiterElinkyes {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(62).cast() }
    }
    #[doc = "0x3e - TCD Beginning Major Loop Count (Minor Loop Channel Linking Disabled)"]
    #[inline(always)]
    pub const fn elinkno_tcd_biter_elinkno(&self) -> &ElinknoTcdBiterElinkno {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(62).cast() }
    }
}
#[doc = "CH_CSR (rw) register accessor: Channel Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_csr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_csr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_csr`] module"]
#[doc(alias = "CH_CSR")]
pub type ChCsr = crate::Reg<ch_csr::ChCsrSpec>;
#[doc = "Channel Control and Status"]
pub mod ch_csr;
#[doc = "CH_ES (rw) register accessor: Channel Error Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_es::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_es::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_es`] module"]
#[doc(alias = "CH_ES")]
pub type ChEs = crate::Reg<ch_es::ChEsSpec>;
#[doc = "Channel Error Status"]
pub mod ch_es;
#[doc = "CH_INT (rw) register accessor: Channel Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_int::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_int::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_int`] module"]
#[doc(alias = "CH_INT")]
pub type ChInt = crate::Reg<ch_int::ChIntSpec>;
#[doc = "Channel Interrupt Status"]
pub mod ch_int;
#[doc = "CH_SBR (rw) register accessor: Channel System Bus\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_sbr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_sbr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_sbr`] module"]
#[doc(alias = "CH_SBR")]
pub type ChSbr = crate::Reg<ch_sbr::ChSbrSpec>;
#[doc = "Channel System Bus"]
pub mod ch_sbr;
#[doc = "CH_PRI (rw) register accessor: Channel Priority\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_pri::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_pri::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_pri`] module"]
#[doc(alias = "CH_PRI")]
pub type ChPri = crate::Reg<ch_pri::ChPriSpec>;
#[doc = "Channel Priority"]
pub mod ch_pri;
#[doc = "CH_MUX (rw) register accessor: Channel Multiplexor Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_mux::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_mux::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch_mux`] module"]
#[doc(alias = "CH_MUX")]
pub type ChMux = crate::Reg<ch_mux::ChMuxSpec>;
#[doc = "Channel Multiplexor Configuration"]
pub mod ch_mux;
#[doc = "TCD_SADDR (rw) register accessor: TCD Source Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_saddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_saddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_saddr`] module"]
#[doc(alias = "TCD_SADDR")]
pub type TcdSaddr = crate::Reg<tcd_saddr::TcdSaddrSpec>;
#[doc = "TCD Source Address"]
pub mod tcd_saddr;
#[doc = "TCD_SOFF (rw) register accessor: TCD Signed Source Address Offset\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_soff::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_soff::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_soff`] module"]
#[doc(alias = "TCD_SOFF")]
pub type TcdSoff = crate::Reg<tcd_soff::TcdSoffSpec>;
#[doc = "TCD Signed Source Address Offset"]
pub mod tcd_soff;
#[doc = "TCD_ATTR (rw) register accessor: TCD Transfer Attributes\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_attr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_attr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_attr`] module"]
#[doc(alias = "TCD_ATTR")]
pub type TcdAttr = crate::Reg<tcd_attr::TcdAttrSpec>;
#[doc = "TCD Transfer Attributes"]
pub mod tcd_attr;
#[doc = "MLOFFNO_TCD_NBYTES_MLOFFNO (rw) register accessor: TCD Transfer Size Without Minor Loop Offsets\n\nYou can [`read`](crate::Reg::read) this register and get [`mloffno_tcd_nbytes_mloffno::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mloffno_tcd_nbytes_mloffno::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mloffno_tcd_nbytes_mloffno`] module"]
#[doc(alias = "MLOFFNO_TCD_NBYTES_MLOFFNO")]
pub type MloffnoTcdNbytesMloffno =
    crate::Reg<mloffno_tcd_nbytes_mloffno::MloffnoTcdNbytesMloffnoSpec>;
#[doc = "TCD Transfer Size Without Minor Loop Offsets"]
pub mod mloffno_tcd_nbytes_mloffno;
#[doc = "MLOFFYES_TCD_NBYTES_MLOFFYES (rw) register accessor: TCD Transfer Size with Minor Loop Offsets\n\nYou can [`read`](crate::Reg::read) this register and get [`mloffyes_tcd_nbytes_mloffyes::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mloffyes_tcd_nbytes_mloffyes::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mloffyes_tcd_nbytes_mloffyes`] module"]
#[doc(alias = "MLOFFYES_TCD_NBYTES_MLOFFYES")]
pub type MloffyesTcdNbytesMloffyes =
    crate::Reg<mloffyes_tcd_nbytes_mloffyes::MloffyesTcdNbytesMloffyesSpec>;
#[doc = "TCD Transfer Size with Minor Loop Offsets"]
pub mod mloffyes_tcd_nbytes_mloffyes;
#[doc = "TCD_SLAST_SDA (rw) register accessor: TCD Last Source Address Adjustment / Store DADDR Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_slast_sda::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_slast_sda::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_slast_sda`] module"]
#[doc(alias = "TCD_SLAST_SDA")]
pub type TcdSlastSda = crate::Reg<tcd_slast_sda::TcdSlastSdaSpec>;
#[doc = "TCD Last Source Address Adjustment / Store DADDR Address"]
pub mod tcd_slast_sda;
#[doc = "TCD_DADDR (rw) register accessor: TCD Destination Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_daddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_daddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_daddr`] module"]
#[doc(alias = "TCD_DADDR")]
pub type TcdDaddr = crate::Reg<tcd_daddr::TcdDaddrSpec>;
#[doc = "TCD Destination Address"]
pub mod tcd_daddr;
#[doc = "TCD_DOFF (rw) register accessor: TCD Signed Destination Address Offset\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_doff::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_doff::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_doff`] module"]
#[doc(alias = "TCD_DOFF")]
pub type TcdDoff = crate::Reg<tcd_doff::TcdDoffSpec>;
#[doc = "TCD Signed Destination Address Offset"]
pub mod tcd_doff;
#[doc = "ELINKNO_TCD_CITER_ELINKNO (rw) register accessor: TCD Current Major Loop Count (Minor Loop Channel Linking Disabled)\n\nYou can [`read`](crate::Reg::read) this register and get [`elinkno_tcd_citer_elinkno::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`elinkno_tcd_citer_elinkno::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@elinkno_tcd_citer_elinkno`] module"]
#[doc(alias = "ELINKNO_TCD_CITER_ELINKNO")]
pub type ElinknoTcdCiterElinkno = crate::Reg<elinkno_tcd_citer_elinkno::ElinknoTcdCiterElinknoSpec>;
#[doc = "TCD Current Major Loop Count (Minor Loop Channel Linking Disabled)"]
pub mod elinkno_tcd_citer_elinkno;
#[doc = "ELINKYES_TCD_CITER_ELINKYES (rw) register accessor: TCD Current Major Loop Count (Minor Loop Channel Linking Enabled)\n\nYou can [`read`](crate::Reg::read) this register and get [`elinkyes_tcd_citer_elinkyes::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`elinkyes_tcd_citer_elinkyes::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@elinkyes_tcd_citer_elinkyes`] module"]
#[doc(alias = "ELINKYES_TCD_CITER_ELINKYES")]
pub type ElinkyesTcdCiterElinkyes =
    crate::Reg<elinkyes_tcd_citer_elinkyes::ElinkyesTcdCiterElinkyesSpec>;
#[doc = "TCD Current Major Loop Count (Minor Loop Channel Linking Enabled)"]
pub mod elinkyes_tcd_citer_elinkyes;
#[doc = "TCD_DLAST_SGA (rw) register accessor: TCD Last Destination Address Adjustment / Scatter Gather Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_dlast_sga::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_dlast_sga::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_dlast_sga`] module"]
#[doc(alias = "TCD_DLAST_SGA")]
pub type TcdDlastSga = crate::Reg<tcd_dlast_sga::TcdDlastSgaSpec>;
#[doc = "TCD Last Destination Address Adjustment / Scatter Gather Address"]
pub mod tcd_dlast_sga;
#[doc = "TCD_CSR (rw) register accessor: TCD Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_csr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_csr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcd_csr`] module"]
#[doc(alias = "TCD_CSR")]
pub type TcdCsr = crate::Reg<tcd_csr::TcdCsrSpec>;
#[doc = "TCD Control and Status"]
pub mod tcd_csr;
#[doc = "ELINKNO_TCD_BITER_ELINKNO (rw) register accessor: TCD Beginning Major Loop Count (Minor Loop Channel Linking Disabled)\n\nYou can [`read`](crate::Reg::read) this register and get [`elinkno_tcd_biter_elinkno::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`elinkno_tcd_biter_elinkno::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@elinkno_tcd_biter_elinkno`] module"]
#[doc(alias = "ELINKNO_TCD_BITER_ELINKNO")]
pub type ElinknoTcdBiterElinkno = crate::Reg<elinkno_tcd_biter_elinkno::ElinknoTcdBiterElinknoSpec>;
#[doc = "TCD Beginning Major Loop Count (Minor Loop Channel Linking Disabled)"]
pub mod elinkno_tcd_biter_elinkno;
#[doc = "ELINKYES_TCD_BITER_ELINKYES (rw) register accessor: TCD Beginning Major Loop Count (Minor Loop Channel Linking Enabled)\n\nYou can [`read`](crate::Reg::read) this register and get [`elinkyes_tcd_biter_elinkyes::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`elinkyes_tcd_biter_elinkyes::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@elinkyes_tcd_biter_elinkyes`] module"]
#[doc(alias = "ELINKYES_TCD_BITER_ELINKYES")]
pub type ElinkyesTcdBiterElinkyes =
    crate::Reg<elinkyes_tcd_biter_elinkyes::ElinkyesTcdBiterElinkyesSpec>;
#[doc = "TCD Beginning Major Loop Count (Minor Loop Channel Linking Enabled)"]
pub mod elinkyes_tcd_biter_elinkyes;
