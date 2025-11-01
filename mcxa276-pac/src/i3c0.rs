#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mconfig: Mconfig,
    sconfig: Sconfig,
    sstatus: Sstatus,
    sctrl: Sctrl,
    sintset: Sintset,
    sintclr: Sintclr,
    sintmasked: Sintmasked,
    serrwarn: Serrwarn,
    sdmactrl: Sdmactrl,
    _reserved9: [u8; 0x08],
    sdatactrl: Sdatactrl,
    swdatab: Swdatab,
    swdatabe: Swdatabe,
    swdatah: Swdatah,
    swdatahe: Swdatahe,
    srdatab: Srdatab,
    _reserved15: [u8; 0x04],
    srdatah: Srdatah,
    _reserved16: [u8; 0x08],
    _reserved_16_byte_swdatab1: [u8; 0x04],
    _reserved17: [u8; 0x04],
    scapabilities2: Scapabilities2,
    scapabilities: Scapabilities,
    sdynaddr: Sdynaddr,
    smaxlimits: Smaxlimits,
    sidpartno: Sidpartno,
    sidext: Sidext,
    svendorid: Svendorid,
    stcclock: Stcclock,
    smsgmapaddr: Smsgmapaddr,
    mconfig_ext: MconfigExt,
    mctrl: Mctrl,
    mstatus: Mstatus,
    mibirules: Mibirules,
    mintset: Mintset,
    mintclr: Mintclr,
    mintmasked: Mintmasked,
    merrwarn: Merrwarn,
    mdmactrl: Mdmactrl,
    _reserved35: [u8; 0x08],
    mdatactrl: Mdatactrl,
    mwdatab: Mwdatab,
    mwdatabe: Mwdatabe,
    mwdatah: Mwdatah,
    mwdatahe: Mwdatahe,
    mrdatab: Mrdatab,
    _reserved41: [u8; 0x04],
    mrdatah: Mrdatah,
    _reserved_42_byte_mwdatab1: [u8; 0x04],
    _reserved_43_data_mwmsg_sdr_data: [u8; 0x04],
    mrmsg_sdr: MrmsgSdr,
    _reserved_45_data_mwmsg_ddr_data: [u8; 0x04],
    mrmsg_ddr: MrmsgDdr,
    _reserved47: [u8; 0x04],
    mdynaddr: Mdynaddr,
    _reserved48: [u8; 0x34],
    smapctrl0: Smapctrl0,
    _reserved49: [u8; 0x20],
    ibiext1: Ibiext1,
    ibiext2: Ibiext2,
    _reserved51: [u8; 0x0eb4],
    sid: Sid,
}
impl RegisterBlock {
    #[doc = "0x00 - Controller Configuration"]
    #[inline(always)]
    pub const fn mconfig(&self) -> &Mconfig {
        &self.mconfig
    }
    #[doc = "0x04 - Target Configuration"]
    #[inline(always)]
    pub const fn sconfig(&self) -> &Sconfig {
        &self.sconfig
    }
    #[doc = "0x08 - Target Status"]
    #[inline(always)]
    pub const fn sstatus(&self) -> &Sstatus {
        &self.sstatus
    }
    #[doc = "0x0c - Target Control"]
    #[inline(always)]
    pub const fn sctrl(&self) -> &Sctrl {
        &self.sctrl
    }
    #[doc = "0x10 - Target Interrupt Set"]
    #[inline(always)]
    pub const fn sintset(&self) -> &Sintset {
        &self.sintset
    }
    #[doc = "0x14 - Target Interrupt Clear"]
    #[inline(always)]
    pub const fn sintclr(&self) -> &Sintclr {
        &self.sintclr
    }
    #[doc = "0x18 - Target Interrupt Mask"]
    #[inline(always)]
    pub const fn sintmasked(&self) -> &Sintmasked {
        &self.sintmasked
    }
    #[doc = "0x1c - Target Errors and Warnings"]
    #[inline(always)]
    pub const fn serrwarn(&self) -> &Serrwarn {
        &self.serrwarn
    }
    #[doc = "0x20 - Target DMA Control"]
    #[inline(always)]
    pub const fn sdmactrl(&self) -> &Sdmactrl {
        &self.sdmactrl
    }
    #[doc = "0x2c - Target Data Control"]
    #[inline(always)]
    pub const fn sdatactrl(&self) -> &Sdatactrl {
        &self.sdatactrl
    }
    #[doc = "0x30 - Target Write Data Byte"]
    #[inline(always)]
    pub const fn swdatab(&self) -> &Swdatab {
        &self.swdatab
    }
    #[doc = "0x34 - Target Write Data Byte End"]
    #[inline(always)]
    pub const fn swdatabe(&self) -> &Swdatabe {
        &self.swdatabe
    }
    #[doc = "0x38 - Target Write Data Halfword"]
    #[inline(always)]
    pub const fn swdatah(&self) -> &Swdatah {
        &self.swdatah
    }
    #[doc = "0x3c - Target Write Data Halfword End"]
    #[inline(always)]
    pub const fn swdatahe(&self) -> &Swdatahe {
        &self.swdatahe
    }
    #[doc = "0x40 - Target Read Data Byte"]
    #[inline(always)]
    pub const fn srdatab(&self) -> &Srdatab {
        &self.srdatab
    }
    #[doc = "0x48 - Target Read Data Halfword"]
    #[inline(always)]
    pub const fn srdatah(&self) -> &Srdatah {
        &self.srdatah
    }
    #[doc = "0x54 - Target Write Data Halfword"]
    #[inline(always)]
    pub const fn halfword_swdatah1(&self) -> &HalfwordSwdatah1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(84).cast() }
    }
    #[doc = "0x54 - Target Write Data Byte"]
    #[inline(always)]
    pub const fn byte_swdatab1(&self) -> &ByteSwdatab1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(84).cast() }
    }
    #[doc = "0x5c - Target Capabilities 2"]
    #[inline(always)]
    pub const fn scapabilities2(&self) -> &Scapabilities2 {
        &self.scapabilities2
    }
    #[doc = "0x60 - Target Capabilities"]
    #[inline(always)]
    pub const fn scapabilities(&self) -> &Scapabilities {
        &self.scapabilities
    }
    #[doc = "0x64 - Target Dynamic Address"]
    #[inline(always)]
    pub const fn sdynaddr(&self) -> &Sdynaddr {
        &self.sdynaddr
    }
    #[doc = "0x68 - Target Maximum Limits"]
    #[inline(always)]
    pub const fn smaxlimits(&self) -> &Smaxlimits {
        &self.smaxlimits
    }
    #[doc = "0x6c - Target ID Part Number"]
    #[inline(always)]
    pub const fn sidpartno(&self) -> &Sidpartno {
        &self.sidpartno
    }
    #[doc = "0x70 - Target ID Extension"]
    #[inline(always)]
    pub const fn sidext(&self) -> &Sidext {
        &self.sidext
    }
    #[doc = "0x74 - Target Vendor ID"]
    #[inline(always)]
    pub const fn svendorid(&self) -> &Svendorid {
        &self.svendorid
    }
    #[doc = "0x78 - Target Time Control Clock"]
    #[inline(always)]
    pub const fn stcclock(&self) -> &Stcclock {
        &self.stcclock
    }
    #[doc = "0x7c - Target Message Map Address"]
    #[inline(always)]
    pub const fn smsgmapaddr(&self) -> &Smsgmapaddr {
        &self.smsgmapaddr
    }
    #[doc = "0x80 - Controller Extended Configuration"]
    #[inline(always)]
    pub const fn mconfig_ext(&self) -> &MconfigExt {
        &self.mconfig_ext
    }
    #[doc = "0x84 - Controller Control"]
    #[inline(always)]
    pub const fn mctrl(&self) -> &Mctrl {
        &self.mctrl
    }
    #[doc = "0x88 - Controller Status"]
    #[inline(always)]
    pub const fn mstatus(&self) -> &Mstatus {
        &self.mstatus
    }
    #[doc = "0x8c - Controller In-band Interrupt Registry and Rules"]
    #[inline(always)]
    pub const fn mibirules(&self) -> &Mibirules {
        &self.mibirules
    }
    #[doc = "0x90 - Controller Interrupt Set"]
    #[inline(always)]
    pub const fn mintset(&self) -> &Mintset {
        &self.mintset
    }
    #[doc = "0x94 - Controller Interrupt Clear"]
    #[inline(always)]
    pub const fn mintclr(&self) -> &Mintclr {
        &self.mintclr
    }
    #[doc = "0x98 - Controller Interrupt Mask"]
    #[inline(always)]
    pub const fn mintmasked(&self) -> &Mintmasked {
        &self.mintmasked
    }
    #[doc = "0x9c - Controller Errors and Warnings"]
    #[inline(always)]
    pub const fn merrwarn(&self) -> &Merrwarn {
        &self.merrwarn
    }
    #[doc = "0xa0 - Controller DMA Control"]
    #[inline(always)]
    pub const fn mdmactrl(&self) -> &Mdmactrl {
        &self.mdmactrl
    }
    #[doc = "0xac - Controller Data Control"]
    #[inline(always)]
    pub const fn mdatactrl(&self) -> &Mdatactrl {
        &self.mdatactrl
    }
    #[doc = "0xb0 - Controller Write Data Byte"]
    #[inline(always)]
    pub const fn mwdatab(&self) -> &Mwdatab {
        &self.mwdatab
    }
    #[doc = "0xb4 - Controller Write Data Byte End"]
    #[inline(always)]
    pub const fn mwdatabe(&self) -> &Mwdatabe {
        &self.mwdatabe
    }
    #[doc = "0xb8 - Controller Write Data Halfword"]
    #[inline(always)]
    pub const fn mwdatah(&self) -> &Mwdatah {
        &self.mwdatah
    }
    #[doc = "0xbc - Controller Write Data Halfword End"]
    #[inline(always)]
    pub const fn mwdatahe(&self) -> &Mwdatahe {
        &self.mwdatahe
    }
    #[doc = "0xc0 - Controller Read Data Byte"]
    #[inline(always)]
    pub const fn mrdatab(&self) -> &Mrdatab {
        &self.mrdatab
    }
    #[doc = "0xc8 - Controller Read Data Halfword"]
    #[inline(always)]
    pub const fn mrdatah(&self) -> &Mrdatah {
        &self.mrdatah
    }
    #[doc = "0xcc - Controller Write Halfword Data (to Bus)"]
    #[inline(always)]
    pub const fn halfword_mwdatah1(&self) -> &HalfwordMwdatah1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xcc - Controller Write Byte Data 1 (to Bus)"]
    #[inline(always)]
    pub const fn byte_mwdatab1(&self) -> &ByteMwdatab1 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(204).cast() }
    }
    #[doc = "0xd0 - Controller Write Message Data in SDR mode"]
    #[inline(always)]
    pub const fn data_mwmsg_sdr_data(&self) -> &DataMwmsgSdrData {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd0 - Controller Write Message Control in SDR mode"]
    #[inline(always)]
    pub const fn control_mwmsg_sdr_control(&self) -> &ControlMwmsgSdrControl {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(208).cast() }
    }
    #[doc = "0xd4 - Controller Read Message in SDR mode"]
    #[inline(always)]
    pub const fn mrmsg_sdr(&self) -> &MrmsgSdr {
        &self.mrmsg_sdr
    }
    #[doc = "0xd8 - Controller Write Message Data in DDR mode"]
    #[inline(always)]
    pub const fn data_mwmsg_ddr_data(&self) -> &DataMwmsgDdrData {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Controller Write Message in DDR Mode Control 2"]
    #[inline(always)]
    pub const fn control2_mwmsg_ddr_control2(&self) -> &Control2MwmsgDdrControl2 {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xd8 - Controller Write Message in DDR mode: First Control Word"]
    #[inline(always)]
    pub const fn control_mwmsg_ddr_control(&self) -> &ControlMwmsgDdrControl {
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(216).cast() }
    }
    #[doc = "0xdc - Controller Read Message in DDR mode"]
    #[inline(always)]
    pub const fn mrmsg_ddr(&self) -> &MrmsgDdr {
        &self.mrmsg_ddr
    }
    #[doc = "0xe4 - Controller Dynamic Address"]
    #[inline(always)]
    pub const fn mdynaddr(&self) -> &Mdynaddr {
        &self.mdynaddr
    }
    #[doc = "0x11c - Map Feature Control 0"]
    #[inline(always)]
    pub const fn smapctrl0(&self) -> &Smapctrl0 {
        &self.smapctrl0
    }
    #[doc = "0x140 - Extended IBI Data 1"]
    #[inline(always)]
    pub const fn ibiext1(&self) -> &Ibiext1 {
        &self.ibiext1
    }
    #[doc = "0x144 - Extended IBI Data 2"]
    #[inline(always)]
    pub const fn ibiext2(&self) -> &Ibiext2 {
        &self.ibiext2
    }
    #[doc = "0xffc - Target Module ID"]
    #[inline(always)]
    pub const fn sid(&self) -> &Sid {
        &self.sid
    }
}
#[doc = "MCONFIG (rw) register accessor: Controller Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mconfig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mconfig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mconfig`] module"]
#[doc(alias = "MCONFIG")]
pub type Mconfig = crate::Reg<mconfig::MconfigSpec>;
#[doc = "Controller Configuration"]
pub mod mconfig;
#[doc = "SCONFIG (rw) register accessor: Target Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`sconfig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sconfig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sconfig`] module"]
#[doc(alias = "SCONFIG")]
pub type Sconfig = crate::Reg<sconfig::SconfigSpec>;
#[doc = "Target Configuration"]
pub mod sconfig;
#[doc = "SSTATUS (rw) register accessor: Target Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sstatus::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sstatus::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sstatus`] module"]
#[doc(alias = "SSTATUS")]
pub type Sstatus = crate::Reg<sstatus::SstatusSpec>;
#[doc = "Target Status"]
pub mod sstatus;
#[doc = "SCTRL (rw) register accessor: Target Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sctrl`] module"]
#[doc(alias = "SCTRL")]
pub type Sctrl = crate::Reg<sctrl::SctrlSpec>;
#[doc = "Target Control"]
pub mod sctrl;
#[doc = "SINTSET (rw) register accessor: Target Interrupt Set\n\nYou can [`read`](crate::Reg::read) this register and get [`sintset::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sintset::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sintset`] module"]
#[doc(alias = "SINTSET")]
pub type Sintset = crate::Reg<sintset::SintsetSpec>;
#[doc = "Target Interrupt Set"]
pub mod sintset;
#[doc = "SINTCLR (rw) register accessor: Target Interrupt Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`sintclr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sintclr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sintclr`] module"]
#[doc(alias = "SINTCLR")]
pub type Sintclr = crate::Reg<sintclr::SintclrSpec>;
#[doc = "Target Interrupt Clear"]
pub mod sintclr;
#[doc = "SINTMASKED (r) register accessor: Target Interrupt Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`sintmasked::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sintmasked`] module"]
#[doc(alias = "SINTMASKED")]
pub type Sintmasked = crate::Reg<sintmasked::SintmaskedSpec>;
#[doc = "Target Interrupt Mask"]
pub mod sintmasked;
#[doc = "SERRWARN (rw) register accessor: Target Errors and Warnings\n\nYou can [`read`](crate::Reg::read) this register and get [`serrwarn::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`serrwarn::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@serrwarn`] module"]
#[doc(alias = "SERRWARN")]
pub type Serrwarn = crate::Reg<serrwarn::SerrwarnSpec>;
#[doc = "Target Errors and Warnings"]
pub mod serrwarn;
#[doc = "SDMACTRL (rw) register accessor: Target DMA Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sdmactrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdmactrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sdmactrl`] module"]
#[doc(alias = "SDMACTRL")]
pub type Sdmactrl = crate::Reg<sdmactrl::SdmactrlSpec>;
#[doc = "Target DMA Control"]
pub mod sdmactrl;
#[doc = "SDATACTRL (rw) register accessor: Target Data Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sdatactrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdatactrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sdatactrl`] module"]
#[doc(alias = "SDATACTRL")]
pub type Sdatactrl = crate::Reg<sdatactrl::SdatactrlSpec>;
#[doc = "Target Data Control"]
pub mod sdatactrl;
#[doc = "SWDATAB (w) register accessor: Target Write Data Byte\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatab::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swdatab`] module"]
#[doc(alias = "SWDATAB")]
pub type Swdatab = crate::Reg<swdatab::SwdatabSpec>;
#[doc = "Target Write Data Byte"]
pub mod swdatab;
#[doc = "SWDATABE (w) register accessor: Target Write Data Byte End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatabe::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swdatabe`] module"]
#[doc(alias = "SWDATABE")]
pub type Swdatabe = crate::Reg<swdatabe::SwdatabeSpec>;
#[doc = "Target Write Data Byte End"]
pub mod swdatabe;
#[doc = "SWDATAH (w) register accessor: Target Write Data Halfword\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatah::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swdatah`] module"]
#[doc(alias = "SWDATAH")]
pub type Swdatah = crate::Reg<swdatah::SwdatahSpec>;
#[doc = "Target Write Data Halfword"]
pub mod swdatah;
#[doc = "SWDATAHE (w) register accessor: Target Write Data Halfword End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatahe::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swdatahe`] module"]
#[doc(alias = "SWDATAHE")]
pub type Swdatahe = crate::Reg<swdatahe::SwdataheSpec>;
#[doc = "Target Write Data Halfword End"]
pub mod swdatahe;
#[doc = "SRDATAB (r) register accessor: Target Read Data Byte\n\nYou can [`read`](crate::Reg::read) this register and get [`srdatab::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srdatab`] module"]
#[doc(alias = "SRDATAB")]
pub type Srdatab = crate::Reg<srdatab::SrdatabSpec>;
#[doc = "Target Read Data Byte"]
pub mod srdatab;
#[doc = "SRDATAH (r) register accessor: Target Read Data Halfword\n\nYou can [`read`](crate::Reg::read) this register and get [`srdatah::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@srdatah`] module"]
#[doc(alias = "SRDATAH")]
pub type Srdatah = crate::Reg<srdatah::SrdatahSpec>;
#[doc = "Target Read Data Halfword"]
pub mod srdatah;
#[doc = "Byte_SWDATAB1 (w) register accessor: Target Write Data Byte\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`byte_swdatab1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@byte_swdatab1`] module"]
#[doc(alias = "Byte_SWDATAB1")]
pub type ByteSwdatab1 = crate::Reg<byte_swdatab1::ByteSwdatab1Spec>;
#[doc = "Target Write Data Byte"]
pub mod byte_swdatab1;
#[doc = "Halfword_SWDATAH1 (w) register accessor: Target Write Data Halfword\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`halfword_swdatah1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@halfword_swdatah1`] module"]
#[doc(alias = "Halfword_SWDATAH1")]
pub type HalfwordSwdatah1 = crate::Reg<halfword_swdatah1::HalfwordSwdatah1Spec>;
#[doc = "Target Write Data Halfword"]
pub mod halfword_swdatah1;
#[doc = "SCAPABILITIES2 (r) register accessor: Target Capabilities 2\n\nYou can [`read`](crate::Reg::read) this register and get [`scapabilities2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scapabilities2`] module"]
#[doc(alias = "SCAPABILITIES2")]
pub type Scapabilities2 = crate::Reg<scapabilities2::Scapabilities2Spec>;
#[doc = "Target Capabilities 2"]
pub mod scapabilities2;
#[doc = "SCAPABILITIES (r) register accessor: Target Capabilities\n\nYou can [`read`](crate::Reg::read) this register and get [`scapabilities::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@scapabilities`] module"]
#[doc(alias = "SCAPABILITIES")]
pub type Scapabilities = crate::Reg<scapabilities::ScapabilitiesSpec>;
#[doc = "Target Capabilities"]
pub mod scapabilities;
#[doc = "SDYNADDR (rw) register accessor: Target Dynamic Address\n\nYou can [`read`](crate::Reg::read) this register and get [`sdynaddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdynaddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sdynaddr`] module"]
#[doc(alias = "SDYNADDR")]
pub type Sdynaddr = crate::Reg<sdynaddr::SdynaddrSpec>;
#[doc = "Target Dynamic Address"]
pub mod sdynaddr;
#[doc = "SMAXLIMITS (rw) register accessor: Target Maximum Limits\n\nYou can [`read`](crate::Reg::read) this register and get [`smaxlimits::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`smaxlimits::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@smaxlimits`] module"]
#[doc(alias = "SMAXLIMITS")]
pub type Smaxlimits = crate::Reg<smaxlimits::SmaxlimitsSpec>;
#[doc = "Target Maximum Limits"]
pub mod smaxlimits;
#[doc = "SIDPARTNO (rw) register accessor: Target ID Part Number\n\nYou can [`read`](crate::Reg::read) this register and get [`sidpartno::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sidpartno::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sidpartno`] module"]
#[doc(alias = "SIDPARTNO")]
pub type Sidpartno = crate::Reg<sidpartno::SidpartnoSpec>;
#[doc = "Target ID Part Number"]
pub mod sidpartno;
#[doc = "SIDEXT (rw) register accessor: Target ID Extension\n\nYou can [`read`](crate::Reg::read) this register and get [`sidext::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sidext::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sidext`] module"]
#[doc(alias = "SIDEXT")]
pub type Sidext = crate::Reg<sidext::SidextSpec>;
#[doc = "Target ID Extension"]
pub mod sidext;
#[doc = "SVENDORID (rw) register accessor: Target Vendor ID\n\nYou can [`read`](crate::Reg::read) this register and get [`svendorid::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`svendorid::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@svendorid`] module"]
#[doc(alias = "SVENDORID")]
pub type Svendorid = crate::Reg<svendorid::SvendoridSpec>;
#[doc = "Target Vendor ID"]
pub mod svendorid;
#[doc = "STCCLOCK (rw) register accessor: Target Time Control Clock\n\nYou can [`read`](crate::Reg::read) this register and get [`stcclock::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stcclock::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stcclock`] module"]
#[doc(alias = "STCCLOCK")]
pub type Stcclock = crate::Reg<stcclock::StcclockSpec>;
#[doc = "Target Time Control Clock"]
pub mod stcclock;
#[doc = "SMSGMAPADDR (r) register accessor: Target Message Map Address\n\nYou can [`read`](crate::Reg::read) this register and get [`smsgmapaddr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@smsgmapaddr`] module"]
#[doc(alias = "SMSGMAPADDR")]
pub type Smsgmapaddr = crate::Reg<smsgmapaddr::SmsgmapaddrSpec>;
#[doc = "Target Message Map Address"]
pub mod smsgmapaddr;
#[doc = "MCONFIG_EXT (rw) register accessor: Controller Extended Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mconfig_ext::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mconfig_ext::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mconfig_ext`] module"]
#[doc(alias = "MCONFIG_EXT")]
pub type MconfigExt = crate::Reg<mconfig_ext::MconfigExtSpec>;
#[doc = "Controller Extended Configuration"]
pub mod mconfig_ext;
#[doc = "MCTRL (rw) register accessor: Controller Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mctrl`] module"]
#[doc(alias = "MCTRL")]
pub type Mctrl = crate::Reg<mctrl::MctrlSpec>;
#[doc = "Controller Control"]
pub mod mctrl;
#[doc = "MSTATUS (rw) register accessor: Controller Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mstatus::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mstatus::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mstatus`] module"]
#[doc(alias = "MSTATUS")]
pub type Mstatus = crate::Reg<mstatus::MstatusSpec>;
#[doc = "Controller Status"]
pub mod mstatus;
#[doc = "MIBIRULES (rw) register accessor: Controller In-band Interrupt Registry and Rules\n\nYou can [`read`](crate::Reg::read) this register and get [`mibirules::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mibirules::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mibirules`] module"]
#[doc(alias = "MIBIRULES")]
pub type Mibirules = crate::Reg<mibirules::MibirulesSpec>;
#[doc = "Controller In-band Interrupt Registry and Rules"]
pub mod mibirules;
#[doc = "MINTSET (rw) register accessor: Controller Interrupt Set\n\nYou can [`read`](crate::Reg::read) this register and get [`mintset::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mintset::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mintset`] module"]
#[doc(alias = "MINTSET")]
pub type Mintset = crate::Reg<mintset::MintsetSpec>;
#[doc = "Controller Interrupt Set"]
pub mod mintset;
#[doc = "MINTCLR (rw) register accessor: Controller Interrupt Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`mintclr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mintclr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mintclr`] module"]
#[doc(alias = "MINTCLR")]
pub type Mintclr = crate::Reg<mintclr::MintclrSpec>;
#[doc = "Controller Interrupt Clear"]
pub mod mintclr;
#[doc = "MINTMASKED (r) register accessor: Controller Interrupt Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`mintmasked::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mintmasked`] module"]
#[doc(alias = "MINTMASKED")]
pub type Mintmasked = crate::Reg<mintmasked::MintmaskedSpec>;
#[doc = "Controller Interrupt Mask"]
pub mod mintmasked;
#[doc = "MERRWARN (rw) register accessor: Controller Errors and Warnings\n\nYou can [`read`](crate::Reg::read) this register and get [`merrwarn::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`merrwarn::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@merrwarn`] module"]
#[doc(alias = "MERRWARN")]
pub type Merrwarn = crate::Reg<merrwarn::MerrwarnSpec>;
#[doc = "Controller Errors and Warnings"]
pub mod merrwarn;
#[doc = "MDMACTRL (rw) register accessor: Controller DMA Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mdmactrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdmactrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mdmactrl`] module"]
#[doc(alias = "MDMACTRL")]
pub type Mdmactrl = crate::Reg<mdmactrl::MdmactrlSpec>;
#[doc = "Controller DMA Control"]
pub mod mdmactrl;
#[doc = "MDATACTRL (rw) register accessor: Controller Data Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mdatactrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdatactrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mdatactrl`] module"]
#[doc(alias = "MDATACTRL")]
pub type Mdatactrl = crate::Reg<mdatactrl::MdatactrlSpec>;
#[doc = "Controller Data Control"]
pub mod mdatactrl;
#[doc = "MWDATAB (w) register accessor: Controller Write Data Byte\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatab::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mwdatab`] module"]
#[doc(alias = "MWDATAB")]
pub type Mwdatab = crate::Reg<mwdatab::MwdatabSpec>;
#[doc = "Controller Write Data Byte"]
pub mod mwdatab;
#[doc = "MWDATABE (w) register accessor: Controller Write Data Byte End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatabe::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mwdatabe`] module"]
#[doc(alias = "MWDATABE")]
pub type Mwdatabe = crate::Reg<mwdatabe::MwdatabeSpec>;
#[doc = "Controller Write Data Byte End"]
pub mod mwdatabe;
#[doc = "MWDATAH (w) register accessor: Controller Write Data Halfword\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatah::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mwdatah`] module"]
#[doc(alias = "MWDATAH")]
pub type Mwdatah = crate::Reg<mwdatah::MwdatahSpec>;
#[doc = "Controller Write Data Halfword"]
pub mod mwdatah;
#[doc = "MWDATAHE (w) register accessor: Controller Write Data Halfword End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatahe::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mwdatahe`] module"]
#[doc(alias = "MWDATAHE")]
pub type Mwdatahe = crate::Reg<mwdatahe::MwdataheSpec>;
#[doc = "Controller Write Data Halfword End"]
pub mod mwdatahe;
#[doc = "MRDATAB (r) register accessor: Controller Read Data Byte\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdatab::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrdatab`] module"]
#[doc(alias = "MRDATAB")]
pub type Mrdatab = crate::Reg<mrdatab::MrdatabSpec>;
#[doc = "Controller Read Data Byte"]
pub mod mrdatab;
#[doc = "MRDATAH (r) register accessor: Controller Read Data Halfword\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdatah::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrdatah`] module"]
#[doc(alias = "MRDATAH")]
pub type Mrdatah = crate::Reg<mrdatah::MrdatahSpec>;
#[doc = "Controller Read Data Halfword"]
pub mod mrdatah;
#[doc = "BYTE_MWDATAB1 (w) register accessor: Controller Write Byte Data 1 (to Bus)\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`byte_mwdatab1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@byte_mwdatab1`] module"]
#[doc(alias = "BYTE_MWDATAB1")]
pub type ByteMwdatab1 = crate::Reg<byte_mwdatab1::ByteMwdatab1Spec>;
#[doc = "Controller Write Byte Data 1 (to Bus)"]
pub mod byte_mwdatab1;
#[doc = "HALFWORD_MWDATAH1 (w) register accessor: Controller Write Halfword Data (to Bus)\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`halfword_mwdatah1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@halfword_mwdatah1`] module"]
#[doc(alias = "HALFWORD_MWDATAH1")]
pub type HalfwordMwdatah1 = crate::Reg<halfword_mwdatah1::HalfwordMwdatah1Spec>;
#[doc = "Controller Write Halfword Data (to Bus)"]
pub mod halfword_mwdatah1;
#[doc = "CONTROL_MWMSG_SDR_CONTROL (w) register accessor: Controller Write Message Control in SDR mode\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control_mwmsg_sdr_control::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@control_mwmsg_sdr_control`] module"]
#[doc(alias = "CONTROL_MWMSG_SDR_CONTROL")]
pub type ControlMwmsgSdrControl = crate::Reg<control_mwmsg_sdr_control::ControlMwmsgSdrControlSpec>;
#[doc = "Controller Write Message Control in SDR mode"]
pub mod control_mwmsg_sdr_control;
#[doc = "DATA_MWMSG_SDR_DATA (w) register accessor: Controller Write Message Data in SDR mode\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data_mwmsg_sdr_data::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@data_mwmsg_sdr_data`] module"]
#[doc(alias = "DATA_MWMSG_SDR_DATA")]
pub type DataMwmsgSdrData = crate::Reg<data_mwmsg_sdr_data::DataMwmsgSdrDataSpec>;
#[doc = "Controller Write Message Data in SDR mode"]
pub mod data_mwmsg_sdr_data;
#[doc = "MRMSG_SDR (r) register accessor: Controller Read Message in SDR mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mrmsg_sdr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrmsg_sdr`] module"]
#[doc(alias = "MRMSG_SDR")]
pub type MrmsgSdr = crate::Reg<mrmsg_sdr::MrmsgSdrSpec>;
#[doc = "Controller Read Message in SDR mode"]
pub mod mrmsg_sdr;
#[doc = "CONTROL_MWMSG_DDR_CONTROL (w) register accessor: Controller Write Message in DDR mode: First Control Word\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control_mwmsg_ddr_control::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@control_mwmsg_ddr_control`] module"]
#[doc(alias = "CONTROL_MWMSG_DDR_CONTROL")]
pub type ControlMwmsgDdrControl = crate::Reg<control_mwmsg_ddr_control::ControlMwmsgDdrControlSpec>;
#[doc = "Controller Write Message in DDR mode: First Control Word"]
pub mod control_mwmsg_ddr_control;
#[doc = "CONTROL2_MWMSG_DDR_CONTROL2 (w) register accessor: Controller Write Message in DDR Mode Control 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control2_mwmsg_ddr_control2::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@control2_mwmsg_ddr_control2`] module"]
#[doc(alias = "CONTROL2_MWMSG_DDR_CONTROL2")]
pub type Control2MwmsgDdrControl2 =
    crate::Reg<control2_mwmsg_ddr_control2::Control2MwmsgDdrControl2Spec>;
#[doc = "Controller Write Message in DDR Mode Control 2"]
pub mod control2_mwmsg_ddr_control2;
#[doc = "DATA_MWMSG_DDR_DATA (w) register accessor: Controller Write Message Data in DDR mode\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data_mwmsg_ddr_data::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@data_mwmsg_ddr_data`] module"]
#[doc(alias = "DATA_MWMSG_DDR_DATA")]
pub type DataMwmsgDdrData = crate::Reg<data_mwmsg_ddr_data::DataMwmsgDdrDataSpec>;
#[doc = "Controller Write Message Data in DDR mode"]
pub mod data_mwmsg_ddr_data;
#[doc = "MRMSG_DDR (r) register accessor: Controller Read Message in DDR mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mrmsg_ddr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrmsg_ddr`] module"]
#[doc(alias = "MRMSG_DDR")]
pub type MrmsgDdr = crate::Reg<mrmsg_ddr::MrmsgDdrSpec>;
#[doc = "Controller Read Message in DDR mode"]
pub mod mrmsg_ddr;
#[doc = "MDYNADDR (rw) register accessor: Controller Dynamic Address\n\nYou can [`read`](crate::Reg::read) this register and get [`mdynaddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdynaddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mdynaddr`] module"]
#[doc(alias = "MDYNADDR")]
pub type Mdynaddr = crate::Reg<mdynaddr::MdynaddrSpec>;
#[doc = "Controller Dynamic Address"]
pub mod mdynaddr;
#[doc = "SMAPCTRL0 (r) register accessor: Map Feature Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`smapctrl0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@smapctrl0`] module"]
#[doc(alias = "SMAPCTRL0")]
pub type Smapctrl0 = crate::Reg<smapctrl0::Smapctrl0Spec>;
#[doc = "Map Feature Control 0"]
pub mod smapctrl0;
#[doc = "IBIEXT1 (rw) register accessor: Extended IBI Data 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ibiext1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ibiext1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ibiext1`] module"]
#[doc(alias = "IBIEXT1")]
pub type Ibiext1 = crate::Reg<ibiext1::Ibiext1Spec>;
#[doc = "Extended IBI Data 1"]
pub mod ibiext1;
#[doc = "IBIEXT2 (rw) register accessor: Extended IBI Data 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ibiext2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ibiext2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ibiext2`] module"]
#[doc(alias = "IBIEXT2")]
pub type Ibiext2 = crate::Reg<ibiext2::Ibiext2Spec>;
#[doc = "Extended IBI Data 2"]
pub mod ibiext2;
#[doc = "SID (r) register accessor: Target Module ID\n\nYou can [`read`](crate::Reg::read) this register and get [`sid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sid`] module"]
#[doc(alias = "SID")]
pub type Sid = crate::Reg<sid::SidSpec>;
#[doc = "Target Module ID"]
pub mod sid;
