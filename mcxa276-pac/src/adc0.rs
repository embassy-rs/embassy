#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    _reserved2: [u8; 0x08],
    ctrl: Ctrl,
    stat: Stat,
    ie: Ie,
    de: De,
    cfg: Cfg,
    pause: Pause,
    _reserved8: [u8; 0x0c],
    swtrig: Swtrig,
    tstat: Tstat,
    _reserved10: [u8; 0x04],
    ofstrim: Ofstrim,
    _reserved11: [u8; 0x04],
    hstrim: Hstrim,
    _reserved12: [u8; 0x54],
    tctrl: [Tctrl; 4],
    _reserved13: [u8; 0x30],
    fctrl0: Fctrl0,
    _reserved14: [u8; 0x0c],
    gcc0: Gcc0,
    _reserved15: [u8; 0x04],
    gcr0: Gcr0,
    _reserved16: [u8; 0x04],
    cmdl1: Cmdl1,
    cmdh1: Cmdh1,
    cmdl2: Cmdl2,
    cmdh2: Cmdh2,
    cmdl3: Cmdl3,
    cmdh3: Cmdh3,
    cmdl4: Cmdl4,
    cmdh4: Cmdh4,
    cmdl5: Cmdl5,
    cmdh5: Cmdh5,
    cmdl6: Cmdl6,
    cmdh6: Cmdh6,
    cmdl7: Cmdl7,
    cmdh7: Cmdh7,
    _reserved30: [u8; 0xc8],
    cv: [Cv; 7],
    _reserved31: [u8; 0xe4],
    resfifo0: Resfifo0,
    _reserved32: [u8; 0xfc],
    cal_gar: [CalGar; 34],
    _reserved33: [u8; 0x0b70],
    cfg2: Cfg2,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID Register"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x04 - Parameter Register"]
    #[inline(always)]
    pub const fn param(&self) -> &Param {
        &self.param
    }
    #[doc = "0x10 - Control Register"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x14 - Status Register"]
    #[inline(always)]
    pub const fn stat(&self) -> &Stat {
        &self.stat
    }
    #[doc = "0x18 - Interrupt Enable Register"]
    #[inline(always)]
    pub const fn ie(&self) -> &Ie {
        &self.ie
    }
    #[doc = "0x1c - DMA Enable Register"]
    #[inline(always)]
    pub const fn de(&self) -> &De {
        &self.de
    }
    #[doc = "0x20 - Configuration Register"]
    #[inline(always)]
    pub const fn cfg(&self) -> &Cfg {
        &self.cfg
    }
    #[doc = "0x24 - Pause Register"]
    #[inline(always)]
    pub const fn pause(&self) -> &Pause {
        &self.pause
    }
    #[doc = "0x34 - Software Trigger Register"]
    #[inline(always)]
    pub const fn swtrig(&self) -> &Swtrig {
        &self.swtrig
    }
    #[doc = "0x38 - Trigger Status Register"]
    #[inline(always)]
    pub const fn tstat(&self) -> &Tstat {
        &self.tstat
    }
    #[doc = "0x40 - Offset Trim Register"]
    #[inline(always)]
    pub const fn ofstrim(&self) -> &Ofstrim {
        &self.ofstrim
    }
    #[doc = "0x48 - High Speed Trim Register"]
    #[inline(always)]
    pub const fn hstrim(&self) -> &Hstrim {
        &self.hstrim
    }
    #[doc = "0xa0..0xb0 - Trigger Control Register"]
    #[inline(always)]
    pub const fn tctrl(&self, n: usize) -> &Tctrl {
        &self.tctrl[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xa0..0xb0 - Trigger Control Register"]
    #[inline(always)]
    pub fn tctrl_iter(&self) -> impl Iterator<Item = &Tctrl> {
        self.tctrl.iter()
    }
    #[doc = "0xe0 - FIFO Control Register"]
    #[inline(always)]
    pub const fn fctrl0(&self) -> &Fctrl0 {
        &self.fctrl0
    }
    #[doc = "0xf0 - Gain Calibration Control"]
    #[inline(always)]
    pub const fn gcc0(&self) -> &Gcc0 {
        &self.gcc0
    }
    #[doc = "0xf8 - Gain Calculation Result"]
    #[inline(always)]
    pub const fn gcr0(&self) -> &Gcr0 {
        &self.gcr0
    }
    #[doc = "0x100 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl1(&self) -> &Cmdl1 {
        &self.cmdl1
    }
    #[doc = "0x104 - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh1(&self) -> &Cmdh1 {
        &self.cmdh1
    }
    #[doc = "0x108 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl2(&self) -> &Cmdl2 {
        &self.cmdl2
    }
    #[doc = "0x10c - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh2(&self) -> &Cmdh2 {
        &self.cmdh2
    }
    #[doc = "0x110 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl3(&self) -> &Cmdl3 {
        &self.cmdl3
    }
    #[doc = "0x114 - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh3(&self) -> &Cmdh3 {
        &self.cmdh3
    }
    #[doc = "0x118 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl4(&self) -> &Cmdl4 {
        &self.cmdl4
    }
    #[doc = "0x11c - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh4(&self) -> &Cmdh4 {
        &self.cmdh4
    }
    #[doc = "0x120 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl5(&self) -> &Cmdl5 {
        &self.cmdl5
    }
    #[doc = "0x124 - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh5(&self) -> &Cmdh5 {
        &self.cmdh5
    }
    #[doc = "0x128 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl6(&self) -> &Cmdl6 {
        &self.cmdl6
    }
    #[doc = "0x12c - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh6(&self) -> &Cmdh6 {
        &self.cmdh6
    }
    #[doc = "0x130 - Command Low Buffer Register"]
    #[inline(always)]
    pub const fn cmdl7(&self) -> &Cmdl7 {
        &self.cmdl7
    }
    #[doc = "0x134 - Command High Buffer Register"]
    #[inline(always)]
    pub const fn cmdh7(&self) -> &Cmdh7 {
        &self.cmdh7
    }
    #[doc = "0x200..0x21c - Compare Value Register"]
    #[doc = ""]
    #[doc = "<div class=\"warning\">`n` is the index of register in the array. `n == 0` corresponds to `CV1` register.</div>"]
    #[inline(always)]
    pub const fn cv(&self, n: usize) -> &Cv {
        &self.cv[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x200..0x21c - Compare Value Register"]
    #[inline(always)]
    pub fn cv_iter(&self) -> impl Iterator<Item = &Cv> {
        self.cv.iter()
    }
    #[doc = "0x200 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv1(&self) -> &Cv {
        self.cv(0)
    }
    #[doc = "0x204 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv2(&self) -> &Cv {
        self.cv(1)
    }
    #[doc = "0x208 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv3(&self) -> &Cv {
        self.cv(2)
    }
    #[doc = "0x20c - Compare Value Register"]
    #[inline(always)]
    pub const fn cv4(&self) -> &Cv {
        self.cv(3)
    }
    #[doc = "0x210 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv5(&self) -> &Cv {
        self.cv(4)
    }
    #[doc = "0x214 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv6(&self) -> &Cv {
        self.cv(5)
    }
    #[doc = "0x218 - Compare Value Register"]
    #[inline(always)]
    pub const fn cv7(&self) -> &Cv {
        self.cv(6)
    }
    #[doc = "0x300 - Data Result FIFO Register"]
    #[inline(always)]
    pub const fn resfifo0(&self) -> &Resfifo0 {
        &self.resfifo0
    }
    #[doc = "0x400..0x488 - Calibration General A-Side Registers"]
    #[inline(always)]
    pub const fn cal_gar(&self, n: usize) -> &CalGar {
        &self.cal_gar[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x400..0x488 - Calibration General A-Side Registers"]
    #[inline(always)]
    pub fn cal_gar_iter(&self) -> impl Iterator<Item = &CalGar> {
        self.cal_gar.iter()
    }
    #[doc = "0xff8 - Configuration 2 Register"]
    #[inline(always)]
    pub const fn cfg2(&self) -> &Cfg2 {
        &self.cfg2
    }
}
#[doc = "VERID (r) register accessor: Version ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID Register"]
pub mod verid;
#[doc = "PARAM (r) register accessor: Parameter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@param`] module"]
#[doc(alias = "PARAM")]
pub type Param = crate::Reg<param::ParamSpec>;
#[doc = "Parameter Register"]
pub mod param;
#[doc = "CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Control Register"]
pub mod ctrl;
#[doc = "STAT (rw) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stat`] module"]
#[doc(alias = "STAT")]
pub type Stat = crate::Reg<stat::StatSpec>;
#[doc = "Status Register"]
pub mod stat;
#[doc = "IE (rw) register accessor: Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ie::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ie::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ie`] module"]
#[doc(alias = "IE")]
pub type Ie = crate::Reg<ie::IeSpec>;
#[doc = "Interrupt Enable Register"]
pub mod ie;
#[doc = "DE (rw) register accessor: DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`de::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`de::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@de`] module"]
#[doc(alias = "DE")]
pub type De = crate::Reg<de::DeSpec>;
#[doc = "DMA Enable Register"]
pub mod de;
#[doc = "CFG (rw) register accessor: Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "Configuration Register"]
pub mod cfg;
#[doc = "PAUSE (rw) register accessor: Pause Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pause::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pause::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pause`] module"]
#[doc(alias = "PAUSE")]
pub type Pause = crate::Reg<pause::PauseSpec>;
#[doc = "Pause Register"]
pub mod pause;
#[doc = "SWTRIG (rw) register accessor: Software Trigger Register\n\nYou can [`read`](crate::Reg::read) this register and get [`swtrig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swtrig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swtrig`] module"]
#[doc(alias = "SWTRIG")]
pub type Swtrig = crate::Reg<swtrig::SwtrigSpec>;
#[doc = "Software Trigger Register"]
pub mod swtrig;
#[doc = "TSTAT (rw) register accessor: Trigger Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tstat`] module"]
#[doc(alias = "TSTAT")]
pub type Tstat = crate::Reg<tstat::TstatSpec>;
#[doc = "Trigger Status Register"]
pub mod tstat;
#[doc = "OFSTRIM (rw) register accessor: Offset Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ofstrim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ofstrim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ofstrim`] module"]
#[doc(alias = "OFSTRIM")]
pub type Ofstrim = crate::Reg<ofstrim::OfstrimSpec>;
#[doc = "Offset Trim Register"]
pub mod ofstrim;
#[doc = "HSTRIM (rw) register accessor: High Speed Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`hstrim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`hstrim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@hstrim`] module"]
#[doc(alias = "HSTRIM")]
pub type Hstrim = crate::Reg<hstrim::HstrimSpec>;
#[doc = "High Speed Trim Register"]
pub mod hstrim;
#[doc = "TCTRL (rw) register accessor: Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tctrl`] module"]
#[doc(alias = "TCTRL")]
pub type Tctrl = crate::Reg<tctrl::TctrlSpec>;
#[doc = "Trigger Control Register"]
pub mod tctrl;
#[doc = "FCTRL0 (rw) register accessor: FIFO Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fctrl0`] module"]
#[doc(alias = "FCTRL0")]
pub type Fctrl0 = crate::Reg<fctrl0::Fctrl0Spec>;
#[doc = "FIFO Control Register"]
pub mod fctrl0;
#[doc = "GCC0 (r) register accessor: Gain Calibration Control\n\nYou can [`read`](crate::Reg::read) this register and get [`gcc0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gcc0`] module"]
#[doc(alias = "GCC0")]
pub type Gcc0 = crate::Reg<gcc0::Gcc0Spec>;
#[doc = "Gain Calibration Control"]
pub mod gcc0;
#[doc = "GCR0 (rw) register accessor: Gain Calculation Result\n\nYou can [`read`](crate::Reg::read) this register and get [`gcr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gcr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gcr0`] module"]
#[doc(alias = "GCR0")]
pub type Gcr0 = crate::Reg<gcr0::Gcr0Spec>;
#[doc = "Gain Calculation Result"]
pub mod gcr0;
#[doc = "CMDL1 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl1`] module"]
#[doc(alias = "CMDL1")]
pub type Cmdl1 = crate::Reg<cmdl1::Cmdl1Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl1;
#[doc = "CMDH1 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh1`] module"]
#[doc(alias = "CMDH1")]
pub type Cmdh1 = crate::Reg<cmdh1::Cmdh1Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh1;
#[doc = "CMDL2 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl2`] module"]
#[doc(alias = "CMDL2")]
pub type Cmdl2 = crate::Reg<cmdl2::Cmdl2Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl2;
#[doc = "CMDH2 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh2`] module"]
#[doc(alias = "CMDH2")]
pub type Cmdh2 = crate::Reg<cmdh2::Cmdh2Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh2;
#[doc = "CMDL3 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl3`] module"]
#[doc(alias = "CMDL3")]
pub type Cmdl3 = crate::Reg<cmdl3::Cmdl3Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl3;
#[doc = "CMDH3 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh3`] module"]
#[doc(alias = "CMDH3")]
pub type Cmdh3 = crate::Reg<cmdh3::Cmdh3Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh3;
#[doc = "CMDL4 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl4`] module"]
#[doc(alias = "CMDL4")]
pub type Cmdl4 = crate::Reg<cmdl4::Cmdl4Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl4;
#[doc = "CMDH4 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh4`] module"]
#[doc(alias = "CMDH4")]
pub type Cmdh4 = crate::Reg<cmdh4::Cmdh4Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh4;
#[doc = "CMDL5 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl5`] module"]
#[doc(alias = "CMDL5")]
pub type Cmdl5 = crate::Reg<cmdl5::Cmdl5Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl5;
#[doc = "CMDH5 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh5`] module"]
#[doc(alias = "CMDH5")]
pub type Cmdh5 = crate::Reg<cmdh5::Cmdh5Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh5;
#[doc = "CMDL6 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl6`] module"]
#[doc(alias = "CMDL6")]
pub type Cmdl6 = crate::Reg<cmdl6::Cmdl6Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl6;
#[doc = "CMDH6 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh6`] module"]
#[doc(alias = "CMDH6")]
pub type Cmdh6 = crate::Reg<cmdh6::Cmdh6Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh6;
#[doc = "CMDL7 (rw) register accessor: Command Low Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdl7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdl7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdl7`] module"]
#[doc(alias = "CMDL7")]
pub type Cmdl7 = crate::Reg<cmdl7::Cmdl7Spec>;
#[doc = "Command Low Buffer Register"]
pub mod cmdl7;
#[doc = "CMDH7 (rw) register accessor: Command High Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cmdh7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmdh7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmdh7`] module"]
#[doc(alias = "CMDH7")]
pub type Cmdh7 = crate::Reg<cmdh7::Cmdh7Spec>;
#[doc = "Command High Buffer Register"]
pub mod cmdh7;
#[doc = "CV (rw) register accessor: Compare Value Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cv`] module"]
#[doc(alias = "CV")]
pub type Cv = crate::Reg<cv::CvSpec>;
#[doc = "Compare Value Register"]
pub mod cv;
#[doc = "RESFIFO0 (r) register accessor: Data Result FIFO Register\n\nYou can [`read`](crate::Reg::read) this register and get [`resfifo0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@resfifo0`] module"]
#[doc(alias = "RESFIFO0")]
pub type Resfifo0 = crate::Reg<resfifo0::Resfifo0Spec>;
#[doc = "Data Result FIFO Register"]
pub mod resfifo0;
#[doc = "CAL_GAR (rw) register accessor: Calibration General A-Side Registers\n\nYou can [`read`](crate::Reg::read) this register and get [`cal_gar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cal_gar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cal_gar`] module"]
#[doc(alias = "CAL_GAR")]
pub type CalGar = crate::Reg<cal_gar::CalGarSpec>;
#[doc = "Calibration General A-Side Registers"]
pub mod cal_gar;
#[doc = "CFG2 (rw) register accessor: Configuration 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg2`] module"]
#[doc(alias = "CFG2")]
pub type Cfg2 = crate::Reg<cfg2::Cfg2Spec>;
#[doc = "Configuration 2 Register"]
pub mod cfg2;
