#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    ctrl: Ctrl,
    pin: Pin,
    shiftstat: Shiftstat,
    shifterr: Shifterr,
    timstat: Timstat,
    _reserved7: [u8; 0x04],
    shiftsien: Shiftsien,
    shifteien: Shifteien,
    timien: Timien,
    _reserved10: [u8; 0x04],
    shiftsden: Shiftsden,
    _reserved11: [u8; 0x04],
    timersden: Timersden,
    _reserved12: [u8; 0x04],
    shiftstate: Shiftstate,
    _reserved13: [u8; 0x04],
    trgstat: Trgstat,
    trigien: Trigien,
    pinstat: Pinstat,
    pinien: Pinien,
    pinren: Pinren,
    pinfen: Pinfen,
    pinoutd: Pinoutd,
    pinoute: Pinoute,
    pinoutdis: Pinoutdis,
    pinoutclr: Pinoutclr,
    pinoutset: Pinoutset,
    pinouttog: Pinouttog,
    _reserved25: [u8; 0x08],
    shiftctl: [Shiftctl; 4],
    _reserved26: [u8; 0x70],
    shiftcfg: [Shiftcfg; 4],
    _reserved27: [u8; 0xf0],
    shiftbuf: [Shiftbuf; 4],
    _reserved28: [u8; 0x70],
    shiftbufbis: [Shiftbufbis; 4],
    _reserved29: [u8; 0x70],
    shiftbufbys: [Shiftbufbys; 4],
    _reserved30: [u8; 0x70],
    shiftbufbbs: [Shiftbufbbs; 4],
    _reserved31: [u8; 0x70],
    timctl: [Timctl; 4],
    _reserved32: [u8; 0x70],
    timcfg: [Timcfg; 4],
    _reserved33: [u8; 0x70],
    timcmp: [Timcmp; 4],
    _reserved34: [u8; 0x0170],
    shiftbufnbs: [Shiftbufnbs; 4],
    _reserved35: [u8; 0x70],
    shiftbufhws: [Shiftbufhws; 4],
    _reserved36: [u8; 0x70],
    shiftbufnis: [Shiftbufnis; 4],
    _reserved37: [u8; 0x70],
    shiftbufoes: [Shiftbufoes; 4],
    _reserved38: [u8; 0x70],
    shiftbufeos: [Shiftbufeos; 4],
    _reserved39: [u8; 0x70],
    shiftbufhbs: [Shiftbufhbs; 4],
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x04 - Parameter"]
    #[inline(always)]
    pub const fn param(&self) -> &Param {
        &self.param
    }
    #[doc = "0x08 - FLEXIO Control"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x0c - Pin State"]
    #[inline(always)]
    pub const fn pin(&self) -> &Pin {
        &self.pin
    }
    #[doc = "0x10 - Shifter Status"]
    #[inline(always)]
    pub const fn shiftstat(&self) -> &Shiftstat {
        &self.shiftstat
    }
    #[doc = "0x14 - Shifter Error"]
    #[inline(always)]
    pub const fn shifterr(&self) -> &Shifterr {
        &self.shifterr
    }
    #[doc = "0x18 - Timer Status Flag"]
    #[inline(always)]
    pub const fn timstat(&self) -> &Timstat {
        &self.timstat
    }
    #[doc = "0x20 - Shifter Status Interrupt Enable"]
    #[inline(always)]
    pub const fn shiftsien(&self) -> &Shiftsien {
        &self.shiftsien
    }
    #[doc = "0x24 - Shifter Error Interrupt Enable"]
    #[inline(always)]
    pub const fn shifteien(&self) -> &Shifteien {
        &self.shifteien
    }
    #[doc = "0x28 - Timer Interrupt Enable"]
    #[inline(always)]
    pub const fn timien(&self) -> &Timien {
        &self.timien
    }
    #[doc = "0x30 - Shifter Status DMA Enable"]
    #[inline(always)]
    pub const fn shiftsden(&self) -> &Shiftsden {
        &self.shiftsden
    }
    #[doc = "0x38 - Timer Status DMA Enable"]
    #[inline(always)]
    pub const fn timersden(&self) -> &Timersden {
        &self.timersden
    }
    #[doc = "0x40 - Shifter State"]
    #[inline(always)]
    pub const fn shiftstate(&self) -> &Shiftstate {
        &self.shiftstate
    }
    #[doc = "0x48 - Trigger Status"]
    #[inline(always)]
    pub const fn trgstat(&self) -> &Trgstat {
        &self.trgstat
    }
    #[doc = "0x4c - External Trigger Interrupt Enable"]
    #[inline(always)]
    pub const fn trigien(&self) -> &Trigien {
        &self.trigien
    }
    #[doc = "0x50 - Pin Status"]
    #[inline(always)]
    pub const fn pinstat(&self) -> &Pinstat {
        &self.pinstat
    }
    #[doc = "0x54 - Pin Interrupt Enable"]
    #[inline(always)]
    pub const fn pinien(&self) -> &Pinien {
        &self.pinien
    }
    #[doc = "0x58 - Pin Rising Edge Enable"]
    #[inline(always)]
    pub const fn pinren(&self) -> &Pinren {
        &self.pinren
    }
    #[doc = "0x5c - Pin Falling Edge Enable"]
    #[inline(always)]
    pub const fn pinfen(&self) -> &Pinfen {
        &self.pinfen
    }
    #[doc = "0x60 - Pin Output Data"]
    #[inline(always)]
    pub const fn pinoutd(&self) -> &Pinoutd {
        &self.pinoutd
    }
    #[doc = "0x64 - Pin Output Enable"]
    #[inline(always)]
    pub const fn pinoute(&self) -> &Pinoute {
        &self.pinoute
    }
    #[doc = "0x68 - Pin Output Disable"]
    #[inline(always)]
    pub const fn pinoutdis(&self) -> &Pinoutdis {
        &self.pinoutdis
    }
    #[doc = "0x6c - Pin Output Clear"]
    #[inline(always)]
    pub const fn pinoutclr(&self) -> &Pinoutclr {
        &self.pinoutclr
    }
    #[doc = "0x70 - Pin Output Set"]
    #[inline(always)]
    pub const fn pinoutset(&self) -> &Pinoutset {
        &self.pinoutset
    }
    #[doc = "0x74 - Pin Output Toggle"]
    #[inline(always)]
    pub const fn pinouttog(&self) -> &Pinouttog {
        &self.pinouttog
    }
    #[doc = "0x80..0x90 - Shifter Control"]
    #[inline(always)]
    pub const fn shiftctl(&self, n: usize) -> &Shiftctl {
        &self.shiftctl[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x80..0x90 - Shifter Control"]
    #[inline(always)]
    pub fn shiftctl_iter(&self) -> impl Iterator<Item = &Shiftctl> {
        self.shiftctl.iter()
    }
    #[doc = "0x100..0x110 - Shifter Configuration"]
    #[inline(always)]
    pub const fn shiftcfg(&self, n: usize) -> &Shiftcfg {
        &self.shiftcfg[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x100..0x110 - Shifter Configuration"]
    #[inline(always)]
    pub fn shiftcfg_iter(&self) -> impl Iterator<Item = &Shiftcfg> {
        self.shiftcfg.iter()
    }
    #[doc = "0x200..0x210 - Shifter Buffer"]
    #[inline(always)]
    pub const fn shiftbuf(&self, n: usize) -> &Shiftbuf {
        &self.shiftbuf[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x200..0x210 - Shifter Buffer"]
    #[inline(always)]
    pub fn shiftbuf_iter(&self) -> impl Iterator<Item = &Shiftbuf> {
        self.shiftbuf.iter()
    }
    #[doc = "0x280..0x290 - Shifter Buffer Bit Swapped"]
    #[inline(always)]
    pub const fn shiftbufbis(&self, n: usize) -> &Shiftbufbis {
        &self.shiftbufbis[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x280..0x290 - Shifter Buffer Bit Swapped"]
    #[inline(always)]
    pub fn shiftbufbis_iter(&self) -> impl Iterator<Item = &Shiftbufbis> {
        self.shiftbufbis.iter()
    }
    #[doc = "0x300..0x310 - Shifter Buffer Byte Swapped"]
    #[inline(always)]
    pub const fn shiftbufbys(&self, n: usize) -> &Shiftbufbys {
        &self.shiftbufbys[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x300..0x310 - Shifter Buffer Byte Swapped"]
    #[inline(always)]
    pub fn shiftbufbys_iter(&self) -> impl Iterator<Item = &Shiftbufbys> {
        self.shiftbufbys.iter()
    }
    #[doc = "0x380..0x390 - Shifter Buffer Bit Byte Swapped"]
    #[inline(always)]
    pub const fn shiftbufbbs(&self, n: usize) -> &Shiftbufbbs {
        &self.shiftbufbbs[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x380..0x390 - Shifter Buffer Bit Byte Swapped"]
    #[inline(always)]
    pub fn shiftbufbbs_iter(&self) -> impl Iterator<Item = &Shiftbufbbs> {
        self.shiftbufbbs.iter()
    }
    #[doc = "0x400..0x410 - Timer Control"]
    #[inline(always)]
    pub const fn timctl(&self, n: usize) -> &Timctl {
        &self.timctl[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x400..0x410 - Timer Control"]
    #[inline(always)]
    pub fn timctl_iter(&self) -> impl Iterator<Item = &Timctl> {
        self.timctl.iter()
    }
    #[doc = "0x480..0x490 - Timer Configuration"]
    #[inline(always)]
    pub const fn timcfg(&self, n: usize) -> &Timcfg {
        &self.timcfg[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x480..0x490 - Timer Configuration"]
    #[inline(always)]
    pub fn timcfg_iter(&self) -> impl Iterator<Item = &Timcfg> {
        self.timcfg.iter()
    }
    #[doc = "0x500..0x510 - Timer Compare"]
    #[inline(always)]
    pub const fn timcmp(&self, n: usize) -> &Timcmp {
        &self.timcmp[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x500..0x510 - Timer Compare"]
    #[inline(always)]
    pub fn timcmp_iter(&self) -> impl Iterator<Item = &Timcmp> {
        self.timcmp.iter()
    }
    #[doc = "0x680..0x690 - Shifter Buffer Nibble Byte Swapped"]
    #[inline(always)]
    pub const fn shiftbufnbs(&self, n: usize) -> &Shiftbufnbs {
        &self.shiftbufnbs[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x680..0x690 - Shifter Buffer Nibble Byte Swapped"]
    #[inline(always)]
    pub fn shiftbufnbs_iter(&self) -> impl Iterator<Item = &Shiftbufnbs> {
        self.shiftbufnbs.iter()
    }
    #[doc = "0x700..0x710 - Shifter Buffer Halfword Swapped"]
    #[inline(always)]
    pub const fn shiftbufhws(&self, n: usize) -> &Shiftbufhws {
        &self.shiftbufhws[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x700..0x710 - Shifter Buffer Halfword Swapped"]
    #[inline(always)]
    pub fn shiftbufhws_iter(&self) -> impl Iterator<Item = &Shiftbufhws> {
        self.shiftbufhws.iter()
    }
    #[doc = "0x780..0x790 - Shifter Buffer Nibble Swapped"]
    #[inline(always)]
    pub const fn shiftbufnis(&self, n: usize) -> &Shiftbufnis {
        &self.shiftbufnis[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x780..0x790 - Shifter Buffer Nibble Swapped"]
    #[inline(always)]
    pub fn shiftbufnis_iter(&self) -> impl Iterator<Item = &Shiftbufnis> {
        self.shiftbufnis.iter()
    }
    #[doc = "0x800..0x810 - Shifter Buffer Odd Even Swapped"]
    #[inline(always)]
    pub const fn shiftbufoes(&self, n: usize) -> &Shiftbufoes {
        &self.shiftbufoes[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x800..0x810 - Shifter Buffer Odd Even Swapped"]
    #[inline(always)]
    pub fn shiftbufoes_iter(&self) -> impl Iterator<Item = &Shiftbufoes> {
        self.shiftbufoes.iter()
    }
    #[doc = "0x880..0x890 - Shifter Buffer Even Odd Swapped"]
    #[inline(always)]
    pub const fn shiftbufeos(&self, n: usize) -> &Shiftbufeos {
        &self.shiftbufeos[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x880..0x890 - Shifter Buffer Even Odd Swapped"]
    #[inline(always)]
    pub fn shiftbufeos_iter(&self) -> impl Iterator<Item = &Shiftbufeos> {
        self.shiftbufeos.iter()
    }
    #[doc = "0x900..0x910 - Shifter Buffer Halfword Byte Swapped"]
    #[inline(always)]
    pub const fn shiftbufhbs(&self, n: usize) -> &Shiftbufhbs {
        &self.shiftbufhbs[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x900..0x910 - Shifter Buffer Halfword Byte Swapped"]
    #[inline(always)]
    pub fn shiftbufhbs_iter(&self) -> impl Iterator<Item = &Shiftbufhbs> {
        self.shiftbufhbs.iter()
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "PARAM (r) register accessor: Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@param`] module"]
#[doc(alias = "PARAM")]
pub type Param = crate::Reg<param::ParamSpec>;
#[doc = "Parameter"]
pub mod param;
#[doc = "CTRL (rw) register accessor: FLEXIO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "FLEXIO Control"]
pub mod ctrl;
#[doc = "PIN (r) register accessor: Pin State\n\nYou can [`read`](crate::Reg::read) this register and get [`pin::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pin`] module"]
#[doc(alias = "PIN")]
pub type Pin = crate::Reg<pin::PinSpec>;
#[doc = "Pin State"]
pub mod pin;
#[doc = "SHIFTSTAT (rw) register accessor: Shifter Status\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftstat`] module"]
#[doc(alias = "SHIFTSTAT")]
pub type Shiftstat = crate::Reg<shiftstat::ShiftstatSpec>;
#[doc = "Shifter Status"]
pub mod shiftstat;
#[doc = "SHIFTERR (rw) register accessor: Shifter Error\n\nYou can [`read`](crate::Reg::read) this register and get [`shifterr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shifterr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shifterr`] module"]
#[doc(alias = "SHIFTERR")]
pub type Shifterr = crate::Reg<shifterr::ShifterrSpec>;
#[doc = "Shifter Error"]
pub mod shifterr;
#[doc = "TIMSTAT (rw) register accessor: Timer Status Flag\n\nYou can [`read`](crate::Reg::read) this register and get [`timstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timstat`] module"]
#[doc(alias = "TIMSTAT")]
pub type Timstat = crate::Reg<timstat::TimstatSpec>;
#[doc = "Timer Status Flag"]
pub mod timstat;
#[doc = "SHIFTSIEN (rw) register accessor: Shifter Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftsien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftsien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftsien`] module"]
#[doc(alias = "SHIFTSIEN")]
pub type Shiftsien = crate::Reg<shiftsien::ShiftsienSpec>;
#[doc = "Shifter Status Interrupt Enable"]
pub mod shiftsien;
#[doc = "SHIFTEIEN (rw) register accessor: Shifter Error Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shifteien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shifteien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shifteien`] module"]
#[doc(alias = "SHIFTEIEN")]
pub type Shifteien = crate::Reg<shifteien::ShifteienSpec>;
#[doc = "Shifter Error Interrupt Enable"]
pub mod shifteien;
#[doc = "TIMIEN (rw) register accessor: Timer Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`timien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timien`] module"]
#[doc(alias = "TIMIEN")]
pub type Timien = crate::Reg<timien::TimienSpec>;
#[doc = "Timer Interrupt Enable"]
pub mod timien;
#[doc = "SHIFTSDEN (rw) register accessor: Shifter Status DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftsden::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftsden::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftsden`] module"]
#[doc(alias = "SHIFTSDEN")]
pub type Shiftsden = crate::Reg<shiftsden::ShiftsdenSpec>;
#[doc = "Shifter Status DMA Enable"]
pub mod shiftsden;
#[doc = "TIMERSDEN (rw) register accessor: Timer Status DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`timersden::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timersden::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timersden`] module"]
#[doc(alias = "TIMERSDEN")]
pub type Timersden = crate::Reg<timersden::TimersdenSpec>;
#[doc = "Timer Status DMA Enable"]
pub mod timersden;
#[doc = "SHIFTSTATE (rw) register accessor: Shifter State\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftstate`] module"]
#[doc(alias = "SHIFTSTATE")]
pub type Shiftstate = crate::Reg<shiftstate::ShiftstateSpec>;
#[doc = "Shifter State"]
pub mod shiftstate;
#[doc = "TRGSTAT (rw) register accessor: Trigger Status\n\nYou can [`read`](crate::Reg::read) this register and get [`trgstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trgstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trgstat`] module"]
#[doc(alias = "TRGSTAT")]
pub type Trgstat = crate::Reg<trgstat::TrgstatSpec>;
#[doc = "Trigger Status"]
pub mod trgstat;
#[doc = "TRIGIEN (rw) register accessor: External Trigger Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`trigien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trigien`] module"]
#[doc(alias = "TRIGIEN")]
pub type Trigien = crate::Reg<trigien::TrigienSpec>;
#[doc = "External Trigger Interrupt Enable"]
pub mod trigien;
#[doc = "PINSTAT (rw) register accessor: Pin Status\n\nYou can [`read`](crate::Reg::read) this register and get [`pinstat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinstat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinstat`] module"]
#[doc(alias = "PINSTAT")]
pub type Pinstat = crate::Reg<pinstat::PinstatSpec>;
#[doc = "Pin Status"]
pub mod pinstat;
#[doc = "PINIEN (rw) register accessor: Pin Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinien`] module"]
#[doc(alias = "PINIEN")]
pub type Pinien = crate::Reg<pinien::PinienSpec>;
#[doc = "Pin Interrupt Enable"]
pub mod pinien;
#[doc = "PINREN (rw) register accessor: Pin Rising Edge Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinren::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinren::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinren`] module"]
#[doc(alias = "PINREN")]
pub type Pinren = crate::Reg<pinren::PinrenSpec>;
#[doc = "Pin Rising Edge Enable"]
pub mod pinren;
#[doc = "PINFEN (rw) register accessor: Pin Falling Edge Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinfen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinfen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinfen`] module"]
#[doc(alias = "PINFEN")]
pub type Pinfen = crate::Reg<pinfen::PinfenSpec>;
#[doc = "Pin Falling Edge Enable"]
pub mod pinfen;
#[doc = "PINOUTD (rw) register accessor: Pin Output Data\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutd::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutd::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinoutd`] module"]
#[doc(alias = "PINOUTD")]
pub type Pinoutd = crate::Reg<pinoutd::PinoutdSpec>;
#[doc = "Pin Output Data"]
pub mod pinoutd;
#[doc = "PINOUTE (rw) register accessor: Pin Output Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoute::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoute::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinoute`] module"]
#[doc(alias = "PINOUTE")]
pub type Pinoute = crate::Reg<pinoute::PinouteSpec>;
#[doc = "Pin Output Enable"]
pub mod pinoute;
#[doc = "PINOUTDIS (rw) register accessor: Pin Output Disable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutdis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutdis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinoutdis`] module"]
#[doc(alias = "PINOUTDIS")]
pub type Pinoutdis = crate::Reg<pinoutdis::PinoutdisSpec>;
#[doc = "Pin Output Disable"]
pub mod pinoutdis;
#[doc = "PINOUTCLR (rw) register accessor: Pin Output Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutclr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutclr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinoutclr`] module"]
#[doc(alias = "PINOUTCLR")]
pub type Pinoutclr = crate::Reg<pinoutclr::PinoutclrSpec>;
#[doc = "Pin Output Clear"]
pub mod pinoutclr;
#[doc = "PINOUTSET (rw) register accessor: Pin Output Set\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutset::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutset::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinoutset`] module"]
#[doc(alias = "PINOUTSET")]
pub type Pinoutset = crate::Reg<pinoutset::PinoutsetSpec>;
#[doc = "Pin Output Set"]
pub mod pinoutset;
#[doc = "PINOUTTOG (rw) register accessor: Pin Output Toggle\n\nYou can [`read`](crate::Reg::read) this register and get [`pinouttog::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinouttog::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pinouttog`] module"]
#[doc(alias = "PINOUTTOG")]
pub type Pinouttog = crate::Reg<pinouttog::PinouttogSpec>;
#[doc = "Pin Output Toggle"]
pub mod pinouttog;
#[doc = "SHIFTCTL (rw) register accessor: Shifter Control\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftctl`] module"]
#[doc(alias = "SHIFTCTL")]
pub type Shiftctl = crate::Reg<shiftctl::ShiftctlSpec>;
#[doc = "Shifter Control"]
pub mod shiftctl;
#[doc = "SHIFTCFG (rw) register accessor: Shifter Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftcfg`] module"]
#[doc(alias = "SHIFTCFG")]
pub type Shiftcfg = crate::Reg<shiftcfg::ShiftcfgSpec>;
#[doc = "Shifter Configuration"]
pub mod shiftcfg;
#[doc = "SHIFTBUF (rw) register accessor: Shifter Buffer\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbuf::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbuf::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbuf`] module"]
#[doc(alias = "SHIFTBUF")]
pub type Shiftbuf = crate::Reg<shiftbuf::ShiftbufSpec>;
#[doc = "Shifter Buffer"]
pub mod shiftbuf;
#[doc = "SHIFTBUFBIS (rw) register accessor: Shifter Buffer Bit Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufbis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufbis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufbis`] module"]
#[doc(alias = "SHIFTBUFBIS")]
pub type Shiftbufbis = crate::Reg<shiftbufbis::ShiftbufbisSpec>;
#[doc = "Shifter Buffer Bit Swapped"]
pub mod shiftbufbis;
#[doc = "SHIFTBUFBYS (rw) register accessor: Shifter Buffer Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufbys::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufbys::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufbys`] module"]
#[doc(alias = "SHIFTBUFBYS")]
pub type Shiftbufbys = crate::Reg<shiftbufbys::ShiftbufbysSpec>;
#[doc = "Shifter Buffer Byte Swapped"]
pub mod shiftbufbys;
#[doc = "SHIFTBUFBBS (rw) register accessor: Shifter Buffer Bit Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufbbs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufbbs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufbbs`] module"]
#[doc(alias = "SHIFTBUFBBS")]
pub type Shiftbufbbs = crate::Reg<shiftbufbbs::ShiftbufbbsSpec>;
#[doc = "Shifter Buffer Bit Byte Swapped"]
pub mod shiftbufbbs;
#[doc = "TIMCTL (rw) register accessor: Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`timctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timctl`] module"]
#[doc(alias = "TIMCTL")]
pub type Timctl = crate::Reg<timctl::TimctlSpec>;
#[doc = "Timer Control"]
pub mod timctl;
#[doc = "TIMCFG (rw) register accessor: Timer Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`timcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timcfg`] module"]
#[doc(alias = "TIMCFG")]
pub type Timcfg = crate::Reg<timcfg::TimcfgSpec>;
#[doc = "Timer Configuration"]
pub mod timcfg;
#[doc = "TIMCMP (rw) register accessor: Timer Compare\n\nYou can [`read`](crate::Reg::read) this register and get [`timcmp::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timcmp::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timcmp`] module"]
#[doc(alias = "TIMCMP")]
pub type Timcmp = crate::Reg<timcmp::TimcmpSpec>;
#[doc = "Timer Compare"]
pub mod timcmp;
#[doc = "SHIFTBUFNBS (rw) register accessor: Shifter Buffer Nibble Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufnbs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufnbs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufnbs`] module"]
#[doc(alias = "SHIFTBUFNBS")]
pub type Shiftbufnbs = crate::Reg<shiftbufnbs::ShiftbufnbsSpec>;
#[doc = "Shifter Buffer Nibble Byte Swapped"]
pub mod shiftbufnbs;
#[doc = "SHIFTBUFHWS (rw) register accessor: Shifter Buffer Halfword Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufhws::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufhws::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufhws`] module"]
#[doc(alias = "SHIFTBUFHWS")]
pub type Shiftbufhws = crate::Reg<shiftbufhws::ShiftbufhwsSpec>;
#[doc = "Shifter Buffer Halfword Swapped"]
pub mod shiftbufhws;
#[doc = "SHIFTBUFNIS (rw) register accessor: Shifter Buffer Nibble Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufnis::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufnis::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufnis`] module"]
#[doc(alias = "SHIFTBUFNIS")]
pub type Shiftbufnis = crate::Reg<shiftbufnis::ShiftbufnisSpec>;
#[doc = "Shifter Buffer Nibble Swapped"]
pub mod shiftbufnis;
#[doc = "SHIFTBUFOES (rw) register accessor: Shifter Buffer Odd Even Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufoes::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufoes::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufoes`] module"]
#[doc(alias = "SHIFTBUFOES")]
pub type Shiftbufoes = crate::Reg<shiftbufoes::ShiftbufoesSpec>;
#[doc = "Shifter Buffer Odd Even Swapped"]
pub mod shiftbufoes;
#[doc = "SHIFTBUFEOS (rw) register accessor: Shifter Buffer Even Odd Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufeos::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufeos::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufeos`] module"]
#[doc(alias = "SHIFTBUFEOS")]
pub type Shiftbufeos = crate::Reg<shiftbufeos::ShiftbufeosSpec>;
#[doc = "Shifter Buffer Even Odd Swapped"]
pub mod shiftbufeos;
#[doc = "SHIFTBUFHBS (rw) register accessor: Shifter Buffer Halfword Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufhbs::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufhbs::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@shiftbufhbs`] module"]
#[doc(alias = "SHIFTBUFHBS")]
pub type Shiftbufhbs = crate::Reg<shiftbufhbs::ShiftbufhbsSpec>;
#[doc = "Shifter Buffer Halfword Byte Swapped"]
pub mod shiftbufhbs;
