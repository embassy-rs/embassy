#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    param: Param,
    global: Global,
    pincfg: Pincfg,
    baud: Baud,
    stat: Stat,
    ctrl: Ctrl,
    data: Data,
    match_: Match,
    modir: Modir,
    fifo: Fifo,
    water: Water,
    dataro: Dataro,
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
    #[doc = "0x08 - Global"]
    #[inline(always)]
    pub const fn global(&self) -> &Global {
        &self.global
    }
    #[doc = "0x0c - Pin Configuration"]
    #[inline(always)]
    pub const fn pincfg(&self) -> &Pincfg {
        &self.pincfg
    }
    #[doc = "0x10 - Baud Rate"]
    #[inline(always)]
    pub const fn baud(&self) -> &Baud {
        &self.baud
    }
    #[doc = "0x14 - Status"]
    #[inline(always)]
    pub const fn stat(&self) -> &Stat {
        &self.stat
    }
    #[doc = "0x18 - Control"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x1c - Data"]
    #[inline(always)]
    pub const fn data(&self) -> &Data {
        &self.data
    }
    #[doc = "0x20 - Match Address"]
    #[inline(always)]
    pub const fn match_(&self) -> &Match {
        &self.match_
    }
    #[doc = "0x24 - MODEM IrDA"]
    #[inline(always)]
    pub const fn modir(&self) -> &Modir {
        &self.modir
    }
    #[doc = "0x28 - FIFO"]
    #[inline(always)]
    pub const fn fifo(&self) -> &Fifo {
        &self.fifo
    }
    #[doc = "0x2c - Watermark"]
    #[inline(always)]
    pub const fn water(&self) -> &Water {
        &self.water
    }
    #[doc = "0x30 - Data Read-Only"]
    #[inline(always)]
    pub const fn dataro(&self) -> &Dataro {
        &self.dataro
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
#[doc = "GLOBAL (rw) register accessor: Global\n\nYou can [`read`](crate::Reg::read) this register and get [`global::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`global::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@global`] module"]
#[doc(alias = "GLOBAL")]
pub type Global = crate::Reg<global::GlobalSpec>;
#[doc = "Global"]
pub mod global;
#[doc = "PINCFG (rw) register accessor: Pin Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`pincfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pincfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pincfg`] module"]
#[doc(alias = "PINCFG")]
pub type Pincfg = crate::Reg<pincfg::PincfgSpec>;
#[doc = "Pin Configuration"]
pub mod pincfg;
#[doc = "BAUD (rw) register accessor: Baud Rate\n\nYou can [`read`](crate::Reg::read) this register and get [`baud::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`baud::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@baud`] module"]
#[doc(alias = "BAUD")]
pub type Baud = crate::Reg<baud::BaudSpec>;
#[doc = "Baud Rate"]
pub mod baud;
#[doc = "STAT (rw) register accessor: Status\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stat`] module"]
#[doc(alias = "STAT")]
pub type Stat = crate::Reg<stat::StatSpec>;
#[doc = "Status"]
pub mod stat;
#[doc = "CTRL (rw) register accessor: Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Control"]
pub mod ctrl;
#[doc = "DATA (rw) register accessor: Data\n\nYou can [`read`](crate::Reg::read) this register and get [`data::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@data`] module"]
#[doc(alias = "DATA")]
pub type Data = crate::Reg<data::DataSpec>;
#[doc = "Data"]
pub mod data;
#[doc = "MATCH (rw) register accessor: Match Address\n\nYou can [`read`](crate::Reg::read) this register and get [`match_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@match_`] module"]
#[doc(alias = "MATCH")]
pub type Match = crate::Reg<match_::MatchSpec>;
#[doc = "Match Address"]
pub mod match_;
#[doc = "MODIR (rw) register accessor: MODEM IrDA\n\nYou can [`read`](crate::Reg::read) this register and get [`modir::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modir::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@modir`] module"]
#[doc(alias = "MODIR")]
pub type Modir = crate::Reg<modir::ModirSpec>;
#[doc = "MODEM IrDA"]
pub mod modir;
#[doc = "FIFO (rw) register accessor: FIFO\n\nYou can [`read`](crate::Reg::read) this register and get [`fifo::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fifo::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fifo`] module"]
#[doc(alias = "FIFO")]
pub type Fifo = crate::Reg<fifo::FifoSpec>;
#[doc = "FIFO"]
pub mod fifo;
#[doc = "WATER (rw) register accessor: Watermark\n\nYou can [`read`](crate::Reg::read) this register and get [`water::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`water::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@water`] module"]
#[doc(alias = "WATER")]
pub type Water = crate::Reg<water::WaterSpec>;
#[doc = "Watermark"]
pub mod water;
#[doc = "DATARO (r) register accessor: Data Read-Only\n\nYou can [`read`](crate::Reg::read) this register and get [`dataro::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dataro`] module"]
#[doc(alias = "DATARO")]
pub type Dataro = crate::Reg<dataro::DataroSpec>;
#[doc = "Data Read-Only"]
pub mod dataro;
