#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    _reserved1: [u8; 0x0c],
    sc: Sc,
    _reserved2: [u8; 0x08],
    lpreq_cfg: LpreqCfg,
    _reserved3: [u8; 0x10],
    pd_status0: PdStatus0,
    _reserved4: [u8; 0x0c],
    sramctl: Sramctl,
    _reserved5: [u8; 0x10],
    sramretldo_reftrim: SramretldoReftrim,
    sramretldo_cntrl: SramretldoCntrl,
    _reserved7: [u8; 0xa4],
    active_cfg: ActiveCfg,
    active_cfg1: ActiveCfg1,
    lp_cfg: LpCfg,
    lp_cfg1: LpCfg1,
    _reserved11: [u8; 0x10],
    lpwkup_delay: LpwkupDelay,
    active_vdelay: ActiveVdelay,
    _reserved13: [u8; 0x08],
    vd_stat: VdStat,
    vd_core_cfg: VdCoreCfg,
    vd_sys_cfg: VdSysCfg,
    _reserved16: [u8; 0x04],
    evd_cfg: EvdCfg,
    _reserved17: [u8; 0x01bc],
    coreldo_cfg: CoreldoCfg,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x10 - Status Control"]
    #[inline(always)]
    pub const fn sc(&self) -> &Sc {
        &self.sc
    }
    #[doc = "0x1c - Low-Power Request Configuration"]
    #[inline(always)]
    pub const fn lpreq_cfg(&self) -> &LpreqCfg {
        &self.lpreq_cfg
    }
    #[doc = "0x30 - SPC Power Domain Mode Status"]
    #[inline(always)]
    pub const fn pd_status0(&self) -> &PdStatus0 {
        &self.pd_status0
    }
    #[doc = "0x40 - SRAM Control"]
    #[inline(always)]
    pub const fn sramctl(&self) -> &Sramctl {
        &self.sramctl
    }
    #[doc = "0x54 - SRAM Retention Reference Trim"]
    #[inline(always)]
    pub const fn sramretldo_reftrim(&self) -> &SramretldoReftrim {
        &self.sramretldo_reftrim
    }
    #[doc = "0x58 - SRAM Retention LDO Control"]
    #[inline(always)]
    pub const fn sramretldo_cntrl(&self) -> &SramretldoCntrl {
        &self.sramretldo_cntrl
    }
    #[doc = "0x100 - Active Power Mode Configuration"]
    #[inline(always)]
    pub const fn active_cfg(&self) -> &ActiveCfg {
        &self.active_cfg
    }
    #[doc = "0x104 - Active Power Mode Configuration 1"]
    #[inline(always)]
    pub const fn active_cfg1(&self) -> &ActiveCfg1 {
        &self.active_cfg1
    }
    #[doc = "0x108 - Low-Power Mode Configuration"]
    #[inline(always)]
    pub const fn lp_cfg(&self) -> &LpCfg {
        &self.lp_cfg
    }
    #[doc = "0x10c - Low Power Mode Configuration 1"]
    #[inline(always)]
    pub const fn lp_cfg1(&self) -> &LpCfg1 {
        &self.lp_cfg1
    }
    #[doc = "0x120 - Low Power Wake-Up Delay"]
    #[inline(always)]
    pub const fn lpwkup_delay(&self) -> &LpwkupDelay {
        &self.lpwkup_delay
    }
    #[doc = "0x124 - Active Voltage Trim Delay"]
    #[inline(always)]
    pub const fn active_vdelay(&self) -> &ActiveVdelay {
        &self.active_vdelay
    }
    #[doc = "0x130 - Voltage Detect Status"]
    #[inline(always)]
    pub const fn vd_stat(&self) -> &VdStat {
        &self.vd_stat
    }
    #[doc = "0x134 - Core Voltage Detect Configuration"]
    #[inline(always)]
    pub const fn vd_core_cfg(&self) -> &VdCoreCfg {
        &self.vd_core_cfg
    }
    #[doc = "0x138 - System Voltage Detect Configuration"]
    #[inline(always)]
    pub const fn vd_sys_cfg(&self) -> &VdSysCfg {
        &self.vd_sys_cfg
    }
    #[doc = "0x140 - External Voltage Domain Configuration"]
    #[inline(always)]
    pub const fn evd_cfg(&self) -> &EvdCfg {
        &self.evd_cfg
    }
    #[doc = "0x300 - LDO_CORE Configuration"]
    #[inline(always)]
    pub const fn coreldo_cfg(&self) -> &CoreldoCfg {
        &self.coreldo_cfg
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "SC (rw) register accessor: Status Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sc`] module"]
#[doc(alias = "SC")]
pub type Sc = crate::Reg<sc::ScSpec>;
#[doc = "Status Control"]
pub mod sc;
#[doc = "LPREQ_CFG (rw) register accessor: Low-Power Request Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`lpreq_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpreq_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpreq_cfg`] module"]
#[doc(alias = "LPREQ_CFG")]
pub type LpreqCfg = crate::Reg<lpreq_cfg::LpreqCfgSpec>;
#[doc = "Low-Power Request Configuration"]
pub mod lpreq_cfg;
#[doc = "PD_STATUS0 (rw) register accessor: SPC Power Domain Mode Status\n\nYou can [`read`](crate::Reg::read) this register and get [`pd_status0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pd_status0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pd_status0`] module"]
#[doc(alias = "PD_STATUS0")]
pub type PdStatus0 = crate::Reg<pd_status0::PdStatus0Spec>;
#[doc = "SPC Power Domain Mode Status"]
pub mod pd_status0;
#[doc = "SRAMCTL (rw) register accessor: SRAM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sramctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sramctl`] module"]
#[doc(alias = "SRAMCTL")]
pub type Sramctl = crate::Reg<sramctl::SramctlSpec>;
#[doc = "SRAM Control"]
pub mod sramctl;
#[doc = "SRAMRETLDO_REFTRIM (rw) register accessor: SRAM Retention Reference Trim\n\nYou can [`read`](crate::Reg::read) this register and get [`sramretldo_reftrim::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramretldo_reftrim::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sramretldo_reftrim`] module"]
#[doc(alias = "SRAMRETLDO_REFTRIM")]
pub type SramretldoReftrim = crate::Reg<sramretldo_reftrim::SramretldoReftrimSpec>;
#[doc = "SRAM Retention Reference Trim"]
pub mod sramretldo_reftrim;
#[doc = "SRAMRETLDO_CNTRL (rw) register accessor: SRAM Retention LDO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sramretldo_cntrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramretldo_cntrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sramretldo_cntrl`] module"]
#[doc(alias = "SRAMRETLDO_CNTRL")]
pub type SramretldoCntrl = crate::Reg<sramretldo_cntrl::SramretldoCntrlSpec>;
#[doc = "SRAM Retention LDO Control"]
pub mod sramretldo_cntrl;
#[doc = "ACTIVE_CFG (rw) register accessor: Active Power Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`active_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`active_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@active_cfg`] module"]
#[doc(alias = "ACTIVE_CFG")]
pub type ActiveCfg = crate::Reg<active_cfg::ActiveCfgSpec>;
#[doc = "Active Power Mode Configuration"]
pub mod active_cfg;
#[doc = "ACTIVE_CFG1 (rw) register accessor: Active Power Mode Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`active_cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`active_cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@active_cfg1`] module"]
#[doc(alias = "ACTIVE_CFG1")]
pub type ActiveCfg1 = crate::Reg<active_cfg1::ActiveCfg1Spec>;
#[doc = "Active Power Mode Configuration 1"]
pub mod active_cfg1;
#[doc = "LP_CFG (rw) register accessor: Low-Power Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`lp_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lp_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lp_cfg`] module"]
#[doc(alias = "LP_CFG")]
pub type LpCfg = crate::Reg<lp_cfg::LpCfgSpec>;
#[doc = "Low-Power Mode Configuration"]
pub mod lp_cfg;
#[doc = "LP_CFG1 (rw) register accessor: Low Power Mode Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lp_cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lp_cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lp_cfg1`] module"]
#[doc(alias = "LP_CFG1")]
pub type LpCfg1 = crate::Reg<lp_cfg1::LpCfg1Spec>;
#[doc = "Low Power Mode Configuration 1"]
pub mod lp_cfg1;
#[doc = "LPWKUP_DELAY (rw) register accessor: Low Power Wake-Up Delay\n\nYou can [`read`](crate::Reg::read) this register and get [`lpwkup_delay::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpwkup_delay::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpwkup_delay`] module"]
#[doc(alias = "LPWKUP_DELAY")]
pub type LpwkupDelay = crate::Reg<lpwkup_delay::LpwkupDelaySpec>;
#[doc = "Low Power Wake-Up Delay"]
pub mod lpwkup_delay;
#[doc = "ACTIVE_VDELAY (rw) register accessor: Active Voltage Trim Delay\n\nYou can [`read`](crate::Reg::read) this register and get [`active_vdelay::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`active_vdelay::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@active_vdelay`] module"]
#[doc(alias = "ACTIVE_VDELAY")]
pub type ActiveVdelay = crate::Reg<active_vdelay::ActiveVdelaySpec>;
#[doc = "Active Voltage Trim Delay"]
pub mod active_vdelay;
#[doc = "VD_STAT (rw) register accessor: Voltage Detect Status\n\nYou can [`read`](crate::Reg::read) this register and get [`vd_stat::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`vd_stat::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@vd_stat`] module"]
#[doc(alias = "VD_STAT")]
pub type VdStat = crate::Reg<vd_stat::VdStatSpec>;
#[doc = "Voltage Detect Status"]
pub mod vd_stat;
#[doc = "VD_CORE_CFG (rw) register accessor: Core Voltage Detect Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`vd_core_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`vd_core_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@vd_core_cfg`] module"]
#[doc(alias = "VD_CORE_CFG")]
pub type VdCoreCfg = crate::Reg<vd_core_cfg::VdCoreCfgSpec>;
#[doc = "Core Voltage Detect Configuration"]
pub mod vd_core_cfg;
#[doc = "VD_SYS_CFG (rw) register accessor: System Voltage Detect Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`vd_sys_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`vd_sys_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@vd_sys_cfg`] module"]
#[doc(alias = "VD_SYS_CFG")]
pub type VdSysCfg = crate::Reg<vd_sys_cfg::VdSysCfgSpec>;
#[doc = "System Voltage Detect Configuration"]
pub mod vd_sys_cfg;
#[doc = "EVD_CFG (rw) register accessor: External Voltage Domain Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`evd_cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`evd_cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@evd_cfg`] module"]
#[doc(alias = "EVD_CFG")]
pub type EvdCfg = crate::Reg<evd_cfg::EvdCfgSpec>;
#[doc = "External Voltage Domain Configuration"]
pub mod evd_cfg;
#[doc = "CORELDO_CFG (r) register accessor: LDO_CORE Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`coreldo_cfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@coreldo_cfg`] module"]
#[doc(alias = "CORELDO_CFG")]
pub type CoreldoCfg = crate::Reg<coreldo_cfg::CoreldoCfgSpec>;
#[doc = "LDO_CORE Configuration"]
pub mod coreldo_cfg;
