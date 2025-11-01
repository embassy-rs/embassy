#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    perid: Perid,
    _reserved1: [u8; 0x03],
    idcomp: Idcomp,
    _reserved2: [u8; 0x03],
    rev: Rev,
    _reserved3: [u8; 0x03],
    addinfo: Addinfo,
    _reserved4: [u8; 0x03],
    otgistat: Otgistat,
    _reserved5: [u8; 0x03],
    otgicr: Otgicr,
    _reserved6: [u8; 0x03],
    otgstat: Otgstat,
    _reserved7: [u8; 0x03],
    otgctl: Otgctl,
    _reserved8: [u8; 0x63],
    istat: Istat,
    _reserved9: [u8; 0x03],
    inten: Inten,
    _reserved10: [u8; 0x03],
    errstat: Errstat,
    _reserved11: [u8; 0x03],
    erren: Erren,
    _reserved12: [u8; 0x03],
    stat: Stat,
    _reserved13: [u8; 0x03],
    ctl: Ctl,
    _reserved14: [u8; 0x03],
    addr: Addr,
    _reserved15: [u8; 0x03],
    bdtpage1: Bdtpage1,
    _reserved16: [u8; 0x03],
    frmnuml: Frmnuml,
    _reserved17: [u8; 0x03],
    frmnumh: Frmnumh,
    _reserved18: [u8; 0x03],
    token: Token,
    _reserved19: [u8; 0x03],
    softhld: Softhld,
    _reserved20: [u8; 0x03],
    bdtpage2: Bdtpage2,
    _reserved21: [u8; 0x03],
    bdtpage3: Bdtpage3,
    _reserved22: [u8; 0x0b],
    endpoint: (),
    _reserved23: [u8; 0x40],
    usbctrl: Usbctrl,
    _reserved24: [u8; 0x03],
    observe: Observe,
    _reserved25: [u8; 0x03],
    control: Control,
    _reserved26: [u8; 0x03],
    usbtrc0: Usbtrc0,
    _reserved27: [u8; 0x07],
    usbfrmadjust: Usbfrmadjust,
    _reserved28: [u8; 0x0f],
    keep_alive_ctrl_rsvd: KeepAliveCtrlRsvd,
    _reserved29: [u8; 0x03],
    keep_alive_wkctrl_rsvd: KeepAliveWkctrlRsvd,
    _reserved30: [u8; 0x03],
    miscctrl: Miscctrl,
    _reserved31: [u8; 0x03],
    stall_il_dis: StallIlDis,
    _reserved32: [u8; 0x03],
    stall_ih_dis: StallIhDis,
    _reserved33: [u8; 0x03],
    stall_ol_dis: StallOlDis,
    _reserved34: [u8; 0x03],
    stall_oh_dis: StallOhDis,
    _reserved35: [u8; 0x03],
    clk_recover_ctrl: ClkRecoverCtrl,
    _reserved36: [u8; 0x03],
    clk_recover_irc_en: ClkRecoverIrcEn,
    _reserved37: [u8; 0x0f],
    clk_recover_int_en: ClkRecoverIntEn,
    _reserved38: [u8; 0x07],
    clk_recover_int_status: ClkRecoverIntStatus,
}
impl RegisterBlock {
    #[doc = "0x00 - Peripheral ID"]
    #[inline(always)]
    pub const fn perid(&self) -> &Perid {
        &self.perid
    }
    #[doc = "0x04 - Peripheral ID Complement"]
    #[inline(always)]
    pub const fn idcomp(&self) -> &Idcomp {
        &self.idcomp
    }
    #[doc = "0x08 - Peripheral Revision"]
    #[inline(always)]
    pub const fn rev(&self) -> &Rev {
        &self.rev
    }
    #[doc = "0x0c - Peripheral Additional Information"]
    #[inline(always)]
    pub const fn addinfo(&self) -> &Addinfo {
        &self.addinfo
    }
    #[doc = "0x10 - OTG Interrupt Status"]
    #[inline(always)]
    pub const fn otgistat(&self) -> &Otgistat {
        &self.otgistat
    }
    #[doc = "0x14 - OTG Interrupt Control"]
    #[inline(always)]
    pub const fn otgicr(&self) -> &Otgicr {
        &self.otgicr
    }
    #[doc = "0x18 - OTG Status"]
    #[inline(always)]
    pub const fn otgstat(&self) -> &Otgstat {
        &self.otgstat
    }
    #[doc = "0x1c - OTG Control"]
    #[inline(always)]
    pub const fn otgctl(&self) -> &Otgctl {
        &self.otgctl
    }
    #[doc = "0x80 - Interrupt Status"]
    #[inline(always)]
    pub const fn istat(&self) -> &Istat {
        &self.istat
    }
    #[doc = "0x84 - Interrupt Enable"]
    #[inline(always)]
    pub const fn inten(&self) -> &Inten {
        &self.inten
    }
    #[doc = "0x88 - Error Interrupt Status"]
    #[inline(always)]
    pub const fn errstat(&self) -> &Errstat {
        &self.errstat
    }
    #[doc = "0x8c - Error Interrupt Enable"]
    #[inline(always)]
    pub const fn erren(&self) -> &Erren {
        &self.erren
    }
    #[doc = "0x90 - Status"]
    #[inline(always)]
    pub const fn stat(&self) -> &Stat {
        &self.stat
    }
    #[doc = "0x94 - Control"]
    #[inline(always)]
    pub const fn ctl(&self) -> &Ctl {
        &self.ctl
    }
    #[doc = "0x98 - Address"]
    #[inline(always)]
    pub const fn addr(&self) -> &Addr {
        &self.addr
    }
    #[doc = "0x9c - BDT Page 1"]
    #[inline(always)]
    pub const fn bdtpage1(&self) -> &Bdtpage1 {
        &self.bdtpage1
    }
    #[doc = "0xa0 - Frame Number Register Low"]
    #[inline(always)]
    pub const fn frmnuml(&self) -> &Frmnuml {
        &self.frmnuml
    }
    #[doc = "0xa4 - Frame Number Register High"]
    #[inline(always)]
    pub const fn frmnumh(&self) -> &Frmnumh {
        &self.frmnumh
    }
    #[doc = "0xa8 - Token"]
    #[inline(always)]
    pub const fn token(&self) -> &Token {
        &self.token
    }
    #[doc = "0xac - SOF Threshold"]
    #[inline(always)]
    pub const fn softhld(&self) -> &Softhld {
        &self.softhld
    }
    #[doc = "0xb0 - BDT Page 2"]
    #[inline(always)]
    pub const fn bdtpage2(&self) -> &Bdtpage2 {
        &self.bdtpage2
    }
    #[doc = "0xb4 - BDT Page 3"]
    #[inline(always)]
    pub const fn bdtpage3(&self) -> &Bdtpage3 {
        &self.bdtpage3
    }
    #[doc = "0xc0..0xd0 - Array of registers: ENDPT"]
    #[inline(always)]
    pub const fn endpoint(&self, n: usize) -> &Endpoint {
        #[allow(clippy::no_effect)]
        [(); 16][n];
        unsafe {
            &*core::ptr::from_ref(self)
                .cast::<u8>()
                .add(192)
                .add(4 * n)
                .cast()
        }
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xc0..0xd0 - Array of registers: ENDPT"]
    #[inline(always)]
    pub fn endpoint_iter(&self) -> impl Iterator<Item = &Endpoint> {
        (0..16).map(move |n| unsafe {
            &*core::ptr::from_ref(self)
                .cast::<u8>()
                .add(192)
                .add(4 * n)
                .cast()
        })
    }
    #[doc = "0x100 - USB Control"]
    #[inline(always)]
    pub const fn usbctrl(&self) -> &Usbctrl {
        &self.usbctrl
    }
    #[doc = "0x104 - USB OTG Observe"]
    #[inline(always)]
    pub const fn observe(&self) -> &Observe {
        &self.observe
    }
    #[doc = "0x108 - USB OTG Control"]
    #[inline(always)]
    pub const fn control(&self) -> &Control {
        &self.control
    }
    #[doc = "0x10c - USB Transceiver Control 0"]
    #[inline(always)]
    pub const fn usbtrc0(&self) -> &Usbtrc0 {
        &self.usbtrc0
    }
    #[doc = "0x114 - Frame Adjust"]
    #[inline(always)]
    pub const fn usbfrmadjust(&self) -> &Usbfrmadjust {
        &self.usbfrmadjust
    }
    #[doc = "0x124 - Reserved"]
    #[inline(always)]
    pub const fn keep_alive_ctrl_rsvd(&self) -> &KeepAliveCtrlRsvd {
        &self.keep_alive_ctrl_rsvd
    }
    #[doc = "0x128 - Reserved"]
    #[inline(always)]
    pub const fn keep_alive_wkctrl_rsvd(&self) -> &KeepAliveWkctrlRsvd {
        &self.keep_alive_wkctrl_rsvd
    }
    #[doc = "0x12c - Miscellaneous Control"]
    #[inline(always)]
    pub const fn miscctrl(&self) -> &Miscctrl {
        &self.miscctrl
    }
    #[doc = "0x130 - Peripheral Mode Stall Disable for Endpoints 7 to 0 in IN Direction"]
    #[inline(always)]
    pub const fn stall_il_dis(&self) -> &StallIlDis {
        &self.stall_il_dis
    }
    #[doc = "0x134 - Peripheral Mode Stall Disable for Endpoints 15 to 8 in IN Direction"]
    #[inline(always)]
    pub const fn stall_ih_dis(&self) -> &StallIhDis {
        &self.stall_ih_dis
    }
    #[doc = "0x138 - Peripheral Mode Stall Disable for Endpoints 7 to 0 in OUT Direction"]
    #[inline(always)]
    pub const fn stall_ol_dis(&self) -> &StallOlDis {
        &self.stall_ol_dis
    }
    #[doc = "0x13c - Peripheral Mode Stall Disable for Endpoints 15 to 8 in OUT Direction"]
    #[inline(always)]
    pub const fn stall_oh_dis(&self) -> &StallOhDis {
        &self.stall_oh_dis
    }
    #[doc = "0x140 - USB Clock Recovery Control"]
    #[inline(always)]
    pub const fn clk_recover_ctrl(&self) -> &ClkRecoverCtrl {
        &self.clk_recover_ctrl
    }
    #[doc = "0x144 - FIRC Oscillator Enable"]
    #[inline(always)]
    pub const fn clk_recover_irc_en(&self) -> &ClkRecoverIrcEn {
        &self.clk_recover_irc_en
    }
    #[doc = "0x154 - Clock Recovery Combined Interrupt Enable"]
    #[inline(always)]
    pub const fn clk_recover_int_en(&self) -> &ClkRecoverIntEn {
        &self.clk_recover_int_en
    }
    #[doc = "0x15c - Clock Recovery Separated Interrupt Status"]
    #[inline(always)]
    pub const fn clk_recover_int_status(&self) -> &ClkRecoverIntStatus {
        &self.clk_recover_int_status
    }
}
#[doc = "PERID (r) register accessor: Peripheral ID\n\nYou can [`read`](crate::Reg::read) this register and get [`perid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@perid`] module"]
#[doc(alias = "PERID")]
pub type Perid = crate::Reg<perid::PeridSpec>;
#[doc = "Peripheral ID"]
pub mod perid;
#[doc = "IDCOMP (r) register accessor: Peripheral ID Complement\n\nYou can [`read`](crate::Reg::read) this register and get [`idcomp::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@idcomp`] module"]
#[doc(alias = "IDCOMP")]
pub type Idcomp = crate::Reg<idcomp::IdcompSpec>;
#[doc = "Peripheral ID Complement"]
pub mod idcomp;
#[doc = "REV (r) register accessor: Peripheral Revision\n\nYou can [`read`](crate::Reg::read) this register and get [`rev::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rev`] module"]
#[doc(alias = "REV")]
pub type Rev = crate::Reg<rev::RevSpec>;
#[doc = "Peripheral Revision"]
pub mod rev;
#[doc = "ADDINFO (r) register accessor: Peripheral Additional Information\n\nYou can [`read`](crate::Reg::read) this register and get [`addinfo::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@addinfo`] module"]
#[doc(alias = "ADDINFO")]
pub type Addinfo = crate::Reg<addinfo::AddinfoSpec>;
#[doc = "Peripheral Additional Information"]
pub mod addinfo;
#[doc = "OTGISTAT (rw) register accessor: OTG Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`otgistat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgistat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@otgistat`] module"]
#[doc(alias = "OTGISTAT")]
pub type Otgistat = crate::Reg<otgistat::OtgistatSpec>;
#[doc = "OTG Interrupt Status"]
pub mod otgistat;
#[doc = "OTGICR (rw) register accessor: OTG Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`otgicr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgicr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@otgicr`] module"]
#[doc(alias = "OTGICR")]
pub type Otgicr = crate::Reg<otgicr::OtgicrSpec>;
#[doc = "OTG Interrupt Control"]
pub mod otgicr;
#[doc = "OTGSTAT (r) register accessor: OTG Status\n\nYou can [`read`](crate::Reg::read) this register and get [`otgstat::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@otgstat`] module"]
#[doc(alias = "OTGSTAT")]
pub type Otgstat = crate::Reg<otgstat::OtgstatSpec>;
#[doc = "OTG Status"]
pub mod otgstat;
#[doc = "OTGCTL (rw) register accessor: OTG Control\n\nYou can [`read`](crate::Reg::read) this register and get [`otgctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@otgctl`] module"]
#[doc(alias = "OTGCTL")]
pub type Otgctl = crate::Reg<otgctl::OtgctlSpec>;
#[doc = "OTG Control"]
pub mod otgctl;
#[doc = "ISTAT (rw) register accessor: Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`istat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`istat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@istat`] module"]
#[doc(alias = "ISTAT")]
pub type Istat = crate::Reg<istat::IstatSpec>;
#[doc = "Interrupt Status"]
pub mod istat;
#[doc = "INTEN (rw) register accessor: Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`inten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`inten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@inten`] module"]
#[doc(alias = "INTEN")]
pub type Inten = crate::Reg<inten::IntenSpec>;
#[doc = "Interrupt Enable"]
pub mod inten;
#[doc = "ERRSTAT (rw) register accessor: Error Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`errstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`errstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@errstat`] module"]
#[doc(alias = "ERRSTAT")]
pub type Errstat = crate::Reg<errstat::ErrstatSpec>;
#[doc = "Error Interrupt Status"]
pub mod errstat;
#[doc = "ERREN (rw) register accessor: Error Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`erren::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erren::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@erren`] module"]
#[doc(alias = "ERREN")]
pub type Erren = crate::Reg<erren::ErrenSpec>;
#[doc = "Error Interrupt Enable"]
pub mod erren;
#[doc = "STAT (r) register accessor: Status\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stat`] module"]
#[doc(alias = "STAT")]
pub type Stat = crate::Reg<stat::StatSpec>;
#[doc = "Status"]
pub mod stat;
#[doc = "CTL (rw) register accessor: Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctl`] module"]
#[doc(alias = "CTL")]
pub type Ctl = crate::Reg<ctl::CtlSpec>;
#[doc = "Control"]
pub mod ctl;
#[doc = "ADDR (rw) register accessor: Address\n\nYou can [`read`](crate::Reg::read) this register and get [`addr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`addr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@addr`] module"]
#[doc(alias = "ADDR")]
pub type Addr = crate::Reg<addr::AddrSpec>;
#[doc = "Address"]
pub mod addr;
#[doc = "BDTPAGE1 (rw) register accessor: BDT Page 1\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bdtpage1`] module"]
#[doc(alias = "BDTPAGE1")]
pub type Bdtpage1 = crate::Reg<bdtpage1::Bdtpage1Spec>;
#[doc = "BDT Page 1"]
pub mod bdtpage1;
#[doc = "FRMNUML (r) register accessor: Frame Number Register Low\n\nYou can [`read`](crate::Reg::read) this register and get [`frmnuml::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frmnuml`] module"]
#[doc(alias = "FRMNUML")]
pub type Frmnuml = crate::Reg<frmnuml::FrmnumlSpec>;
#[doc = "Frame Number Register Low"]
pub mod frmnuml;
#[doc = "FRMNUMH (r) register accessor: Frame Number Register High\n\nYou can [`read`](crate::Reg::read) this register and get [`frmnumh::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frmnumh`] module"]
#[doc(alias = "FRMNUMH")]
pub type Frmnumh = crate::Reg<frmnumh::FrmnumhSpec>;
#[doc = "Frame Number Register High"]
pub mod frmnumh;
#[doc = "TOKEN (rw) register accessor: Token\n\nYou can [`read`](crate::Reg::read) this register and get [`token::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`token::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@token`] module"]
#[doc(alias = "TOKEN")]
pub type Token = crate::Reg<token::TokenSpec>;
#[doc = "Token"]
pub mod token;
#[doc = "SOFTHLD (rw) register accessor: SOF Threshold\n\nYou can [`read`](crate::Reg::read) this register and get [`softhld::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`softhld::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@softhld`] module"]
#[doc(alias = "SOFTHLD")]
pub type Softhld = crate::Reg<softhld::SofthldSpec>;
#[doc = "SOF Threshold"]
pub mod softhld;
#[doc = "BDTPAGE2 (rw) register accessor: BDT Page 2\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bdtpage2`] module"]
#[doc(alias = "BDTPAGE2")]
pub type Bdtpage2 = crate::Reg<bdtpage2::Bdtpage2Spec>;
#[doc = "BDT Page 2"]
pub mod bdtpage2;
#[doc = "BDTPAGE3 (rw) register accessor: BDT Page 3\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bdtpage3`] module"]
#[doc(alias = "BDTPAGE3")]
pub type Bdtpage3 = crate::Reg<bdtpage3::Bdtpage3Spec>;
#[doc = "BDT Page 3"]
pub mod bdtpage3;
#[doc = "Array of registers: ENDPT"]
pub use self::endpoint::Endpoint;
#[doc = r"Cluster"]
#[doc = "Array of registers: ENDPT"]
pub mod endpoint;
#[doc = "USBCTRL (rw) register accessor: USB Control\n\nYou can [`read`](crate::Reg::read) this register and get [`usbctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@usbctrl`] module"]
#[doc(alias = "USBCTRL")]
pub type Usbctrl = crate::Reg<usbctrl::UsbctrlSpec>;
#[doc = "USB Control"]
pub mod usbctrl;
#[doc = "OBSERVE (r) register accessor: USB OTG Observe\n\nYou can [`read`](crate::Reg::read) this register and get [`observe::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@observe`] module"]
#[doc(alias = "OBSERVE")]
pub type Observe = crate::Reg<observe::ObserveSpec>;
#[doc = "USB OTG Observe"]
pub mod observe;
#[doc = "CONTROL (rw) register accessor: USB OTG Control\n\nYou can [`read`](crate::Reg::read) this register and get [`control::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@control`] module"]
#[doc(alias = "CONTROL")]
pub type Control = crate::Reg<control::ControlSpec>;
#[doc = "USB OTG Control"]
pub mod control;
#[doc = "USBTRC0 (rw) register accessor: USB Transceiver Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`usbtrc0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbtrc0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@usbtrc0`] module"]
#[doc(alias = "USBTRC0")]
pub type Usbtrc0 = crate::Reg<usbtrc0::Usbtrc0Spec>;
#[doc = "USB Transceiver Control 0"]
pub mod usbtrc0;
#[doc = "USBFRMADJUST (rw) register accessor: Frame Adjust\n\nYou can [`read`](crate::Reg::read) this register and get [`usbfrmadjust::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbfrmadjust::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@usbfrmadjust`] module"]
#[doc(alias = "USBFRMADJUST")]
pub type Usbfrmadjust = crate::Reg<usbfrmadjust::UsbfrmadjustSpec>;
#[doc = "Frame Adjust"]
pub mod usbfrmadjust;
#[doc = "KEEP_ALIVE_CTRL_RSVD (rw) register accessor: Reserved\n\nYou can [`read`](crate::Reg::read) this register and get [`keep_alive_ctrl_rsvd::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`keep_alive_ctrl_rsvd::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@keep_alive_ctrl_rsvd`] module"]
#[doc(alias = "KEEP_ALIVE_CTRL_RSVD")]
pub type KeepAliveCtrlRsvd = crate::Reg<keep_alive_ctrl_rsvd::KeepAliveCtrlRsvdSpec>;
#[doc = "Reserved"]
pub mod keep_alive_ctrl_rsvd;
#[doc = "KEEP_ALIVE_WKCTRL_RSVD (rw) register accessor: Reserved\n\nYou can [`read`](crate::Reg::read) this register and get [`keep_alive_wkctrl_rsvd::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`keep_alive_wkctrl_rsvd::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@keep_alive_wkctrl_rsvd`] module"]
#[doc(alias = "KEEP_ALIVE_WKCTRL_RSVD")]
pub type KeepAliveWkctrlRsvd = crate::Reg<keep_alive_wkctrl_rsvd::KeepAliveWkctrlRsvdSpec>;
#[doc = "Reserved"]
pub mod keep_alive_wkctrl_rsvd;
#[doc = "MISCCTRL (rw) register accessor: Miscellaneous Control\n\nYou can [`read`](crate::Reg::read) this register and get [`miscctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`miscctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@miscctrl`] module"]
#[doc(alias = "MISCCTRL")]
pub type Miscctrl = crate::Reg<miscctrl::MiscctrlSpec>;
#[doc = "Miscellaneous Control"]
pub mod miscctrl;
#[doc = "STALL_IL_DIS (rw) register accessor: Peripheral Mode Stall Disable for Endpoints 7 to 0 in IN Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_il_dis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_il_dis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stall_il_dis`] module"]
#[doc(alias = "STALL_IL_DIS")]
pub type StallIlDis = crate::Reg<stall_il_dis::StallIlDisSpec>;
#[doc = "Peripheral Mode Stall Disable for Endpoints 7 to 0 in IN Direction"]
pub mod stall_il_dis;
#[doc = "STALL_IH_DIS (rw) register accessor: Peripheral Mode Stall Disable for Endpoints 15 to 8 in IN Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_ih_dis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_ih_dis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stall_ih_dis`] module"]
#[doc(alias = "STALL_IH_DIS")]
pub type StallIhDis = crate::Reg<stall_ih_dis::StallIhDisSpec>;
#[doc = "Peripheral Mode Stall Disable for Endpoints 15 to 8 in IN Direction"]
pub mod stall_ih_dis;
#[doc = "STALL_OL_DIS (rw) register accessor: Peripheral Mode Stall Disable for Endpoints 7 to 0 in OUT Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_ol_dis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_ol_dis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stall_ol_dis`] module"]
#[doc(alias = "STALL_OL_DIS")]
pub type StallOlDis = crate::Reg<stall_ol_dis::StallOlDisSpec>;
#[doc = "Peripheral Mode Stall Disable for Endpoints 7 to 0 in OUT Direction"]
pub mod stall_ol_dis;
#[doc = "STALL_OH_DIS (rw) register accessor: Peripheral Mode Stall Disable for Endpoints 15 to 8 in OUT Direction\n\nYou can [`read`](crate::Reg::read) this register and get [`stall_oh_dis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stall_oh_dis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stall_oh_dis`] module"]
#[doc(alias = "STALL_OH_DIS")]
pub type StallOhDis = crate::Reg<stall_oh_dis::StallOhDisSpec>;
#[doc = "Peripheral Mode Stall Disable for Endpoints 15 to 8 in OUT Direction"]
pub mod stall_oh_dis;
#[doc = "CLK_RECOVER_CTRL (rw) register accessor: USB Clock Recovery Control\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@clk_recover_ctrl`] module"]
#[doc(alias = "CLK_RECOVER_CTRL")]
pub type ClkRecoverCtrl = crate::Reg<clk_recover_ctrl::ClkRecoverCtrlSpec>;
#[doc = "USB Clock Recovery Control"]
pub mod clk_recover_ctrl;
#[doc = "CLK_RECOVER_IRC_EN (rw) register accessor: FIRC Oscillator Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_irc_en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_irc_en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@clk_recover_irc_en`] module"]
#[doc(alias = "CLK_RECOVER_IRC_EN")]
pub type ClkRecoverIrcEn = crate::Reg<clk_recover_irc_en::ClkRecoverIrcEnSpec>;
#[doc = "FIRC Oscillator Enable"]
pub mod clk_recover_irc_en;
#[doc = "CLK_RECOVER_INT_EN (rw) register accessor: Clock Recovery Combined Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_int_en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_int_en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@clk_recover_int_en`] module"]
#[doc(alias = "CLK_RECOVER_INT_EN")]
pub type ClkRecoverIntEn = crate::Reg<clk_recover_int_en::ClkRecoverIntEnSpec>;
#[doc = "Clock Recovery Combined Interrupt Enable"]
pub mod clk_recover_int_en;
#[doc = "CLK_RECOVER_INT_STATUS (rw) register accessor: Clock Recovery Separated Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_int_status::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_int_status::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@clk_recover_int_status`] module"]
#[doc(alias = "CLK_RECOVER_INT_STATUS")]
pub type ClkRecoverIntStatus = crate::Reg<clk_recover_int_status::ClkRecoverIntStatusSpec>;
#[doc = "Clock Recovery Separated Interrupt Status"]
pub mod clk_recover_int_status;
