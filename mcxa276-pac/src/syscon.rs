#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x0200],
    remap: Remap,
    _reserved1: [u8; 0x0c],
    ahbmatprio: Ahbmatprio,
    _reserved2: [u8; 0x28],
    cpu0nstckcal: Cpu0nstckcal,
    _reserved3: [u8; 0x08],
    nmisrc: Nmisrc,
    protlvl: Protlvl,
    _reserved5: [u8; 0x0128],
    slowclkdiv: Slowclkdiv,
    busclkdiv: Busclkdiv,
    ahbclkdiv: Ahbclkdiv,
    _reserved8: [u8; 0x04],
    frohfdiv: Frohfdiv,
    frolfdiv: Frolfdiv,
    _reserved10: [u8; 0x54],
    pll1clkdiv: Pll1clkdiv,
    _reserved11: [u8; 0x14],
    clkunlock: Clkunlock,
    nvm_ctrl: NvmCtrl,
    _reserved13: [u8; 0x10],
    smart_dmaint: SmartDmaint,
    _reserved14: [u8; 0x58],
    ram_interleave: RamInterleave,
    _reserved15: [u8; 0x0398],
    cpustat: Cpustat,
    _reserved16: [u8; 0x14],
    lpcac_ctrl: LpcacCtrl,
    _reserved17: [u8; 0x0110],
    pwm0subctl: Pwm0subctl,
    pwm1subctl: Pwm1subctl,
    ctimerglobalstarten: Ctimerglobalstarten,
    ram_ctrl: RamCtrl,
    _reserved21: [u8; 0x0218],
    gray_code_lsb: GrayCodeLsb,
    gray_code_msb: GrayCodeMsb,
    binary_code_lsb: BinaryCodeLsb,
    binary_code_msb: BinaryCodeMsb,
    _reserved25: [u8; 0x02a0],
    els_udf: ElsUdf,
    _reserved26: [u8; 0x08],
    msfcfg: Msfcfg,
    els_uid: [ElsUid; 4],
    _reserved28: [u8; 0x0c],
    rop_state: RopState,
    _reserved29: [u8; 0x18],
    sram_xen: SramXen,
    sram_xen_dp: SramXenDp,
    _reserved31: [u8; 0x20],
    els_otp_lc_state: ElsOtpLcState,
    els_otp_lc_state_dp: ElsOtpLcStateDp,
    _reserved33: [u8; 0x0118],
    debug_lock_en: DebugLockEn,
    debug_features: DebugFeatures,
    debug_features_dp: DebugFeaturesDp,
    _reserved36: [u8; 0x08],
    swd_access_cpu0: SwdAccessCpu0,
    _reserved37: [u8; 0x08],
    debug_auth_beacon: DebugAuthBeacon,
    _reserved38: [u8; 0x2c],
    jtag_id: JtagId,
    device_type: DeviceType,
    device_id0: DeviceId0,
    dieid: Dieid,
}
impl RegisterBlock {
    #[doc = "0x200 - AHB Matrix Remap Control"]
    #[inline(always)]
    pub const fn remap(&self) -> &Remap {
        &self.remap
    }
    #[doc = "0x210 - AHB Matrix Priority Control"]
    #[inline(always)]
    pub const fn ahbmatprio(&self) -> &Ahbmatprio {
        &self.ahbmatprio
    }
    #[doc = "0x23c - Non-Secure CPU0 System Tick Calibration"]
    #[inline(always)]
    pub const fn cpu0nstckcal(&self) -> &Cpu0nstckcal {
        &self.cpu0nstckcal
    }
    #[doc = "0x248 - NMI Source Select"]
    #[inline(always)]
    pub const fn nmisrc(&self) -> &Nmisrc {
        &self.nmisrc
    }
    #[doc = "0x24c - Protect Level Control"]
    #[inline(always)]
    pub const fn protlvl(&self) -> &Protlvl {
        &self.protlvl
    }
    #[doc = "0x378 - SLOW_CLK Clock Divider"]
    #[inline(always)]
    pub const fn slowclkdiv(&self) -> &Slowclkdiv {
        &self.slowclkdiv
    }
    #[doc = "0x37c - BUS_CLK Clock Divider"]
    #[inline(always)]
    pub const fn busclkdiv(&self) -> &Busclkdiv {
        &self.busclkdiv
    }
    #[doc = "0x380 - System Clock Divider"]
    #[inline(always)]
    pub const fn ahbclkdiv(&self) -> &Ahbclkdiv {
        &self.ahbclkdiv
    }
    #[doc = "0x388 - FRO_HF_DIV Clock Divider"]
    #[inline(always)]
    pub const fn frohfdiv(&self) -> &Frohfdiv {
        &self.frohfdiv
    }
    #[doc = "0x38c - FRO_LF_DIV Clock Divider"]
    #[inline(always)]
    pub const fn frolfdiv(&self) -> &Frolfdiv {
        &self.frolfdiv
    }
    #[doc = "0x3e4 - PLL1_CLK_DIV Clock Divider"]
    #[inline(always)]
    pub const fn pll1clkdiv(&self) -> &Pll1clkdiv {
        &self.pll1clkdiv
    }
    #[doc = "0x3fc - Clock Configuration Unlock"]
    #[inline(always)]
    pub const fn clkunlock(&self) -> &Clkunlock {
        &self.clkunlock
    }
    #[doc = "0x400 - NVM Control"]
    #[inline(always)]
    pub const fn nvm_ctrl(&self) -> &NvmCtrl {
        &self.nvm_ctrl
    }
    #[doc = "0x414 - SmartDMA Interrupt Hijack"]
    #[inline(always)]
    pub const fn smart_dmaint(&self) -> &SmartDmaint {
        &self.smart_dmaint
    }
    #[doc = "0x470 - Controls RAM Interleave Integration"]
    #[inline(always)]
    pub const fn ram_interleave(&self) -> &RamInterleave {
        &self.ram_interleave
    }
    #[doc = "0x80c - CPU Status"]
    #[inline(always)]
    pub const fn cpustat(&self) -> &Cpustat {
        &self.cpustat
    }
    #[doc = "0x824 - LPCAC Control"]
    #[inline(always)]
    pub const fn lpcac_ctrl(&self) -> &LpcacCtrl {
        &self.lpcac_ctrl
    }
    #[doc = "0x938 - PWM0 Submodule Control"]
    #[inline(always)]
    pub const fn pwm0subctl(&self) -> &Pwm0subctl {
        &self.pwm0subctl
    }
    #[doc = "0x93c - PWM1 Submodule Control"]
    #[inline(always)]
    pub const fn pwm1subctl(&self) -> &Pwm1subctl {
        &self.pwm1subctl
    }
    #[doc = "0x940 - CTIMER Global Start Enable"]
    #[inline(always)]
    pub const fn ctimerglobalstarten(&self) -> &Ctimerglobalstarten {
        &self.ctimerglobalstarten
    }
    #[doc = "0x944 - RAM Control"]
    #[inline(always)]
    pub const fn ram_ctrl(&self) -> &RamCtrl {
        &self.ram_ctrl
    }
    #[doc = "0xb60 - Gray to Binary Converter Gray Code \\[31:0\\]"]
    #[inline(always)]
    pub const fn gray_code_lsb(&self) -> &GrayCodeLsb {
        &self.gray_code_lsb
    }
    #[doc = "0xb64 - Gray to Binary Converter Gray Code \\[41:32\\]"]
    #[inline(always)]
    pub const fn gray_code_msb(&self) -> &GrayCodeMsb {
        &self.gray_code_msb
    }
    #[doc = "0xb68 - Gray to Binary Converter Binary Code \\[31:0\\]"]
    #[inline(always)]
    pub const fn binary_code_lsb(&self) -> &BinaryCodeLsb {
        &self.binary_code_lsb
    }
    #[doc = "0xb6c - Gray to Binary Converter Binary Code \\[41:32\\]"]
    #[inline(always)]
    pub const fn binary_code_msb(&self) -> &BinaryCodeMsb {
        &self.binary_code_msb
    }
    #[doc = "0xe10 - UDF Control"]
    #[inline(always)]
    pub const fn els_udf(&self) -> &ElsUdf {
        &self.els_udf
    }
    #[doc = "0xe1c - MSF Configuration"]
    #[inline(always)]
    pub const fn msfcfg(&self) -> &Msfcfg {
        &self.msfcfg
    }
    #[doc = "0xe20..0xe30 - Device UID n"]
    #[inline(always)]
    pub const fn els_uid(&self, n: usize) -> &ElsUid {
        &self.els_uid[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xe20..0xe30 - Device UID n"]
    #[inline(always)]
    pub fn els_uid_iter(&self) -> impl Iterator<Item = &ElsUid> {
        self.els_uid.iter()
    }
    #[doc = "0xe3c - ROP State Register"]
    #[inline(always)]
    pub const fn rop_state(&self) -> &RopState {
        &self.rop_state
    }
    #[doc = "0xe58 - RAM XEN Control"]
    #[inline(always)]
    pub const fn sram_xen(&self) -> &SramXen {
        &self.sram_xen
    }
    #[doc = "0xe5c - RAM XEN Control (Duplicate)"]
    #[inline(always)]
    pub const fn sram_xen_dp(&self) -> &SramXenDp {
        &self.sram_xen_dp
    }
    #[doc = "0xe80 - Life Cycle State Register"]
    #[inline(always)]
    pub const fn els_otp_lc_state(&self) -> &ElsOtpLcState {
        &self.els_otp_lc_state
    }
    #[doc = "0xe84 - Life Cycle State Register (Duplicate)"]
    #[inline(always)]
    pub const fn els_otp_lc_state_dp(&self) -> &ElsOtpLcStateDp {
        &self.els_otp_lc_state_dp
    }
    #[doc = "0xfa0 - Control Write Access to Security"]
    #[inline(always)]
    pub const fn debug_lock_en(&self) -> &DebugLockEn {
        &self.debug_lock_en
    }
    #[doc = "0xfa4 - Cortex Debug Features Control"]
    #[inline(always)]
    pub const fn debug_features(&self) -> &DebugFeatures {
        &self.debug_features
    }
    #[doc = "0xfa8 - Cortex Debug Features Control (Duplicate)"]
    #[inline(always)]
    pub const fn debug_features_dp(&self) -> &DebugFeaturesDp {
        &self.debug_features_dp
    }
    #[doc = "0xfb4 - CPU0 Software Debug Access"]
    #[inline(always)]
    pub const fn swd_access_cpu0(&self) -> &SwdAccessCpu0 {
        &self.swd_access_cpu0
    }
    #[doc = "0xfc0 - Debug Authentication BEACON"]
    #[inline(always)]
    pub const fn debug_auth_beacon(&self) -> &DebugAuthBeacon {
        &self.debug_auth_beacon
    }
    #[doc = "0xff0 - JTAG Chip ID"]
    #[inline(always)]
    pub const fn jtag_id(&self) -> &JtagId {
        &self.jtag_id
    }
    #[doc = "0xff4 - Device Type"]
    #[inline(always)]
    pub const fn device_type(&self) -> &DeviceType {
        &self.device_type
    }
    #[doc = "0xff8 - Device ID"]
    #[inline(always)]
    pub const fn device_id0(&self) -> &DeviceId0 {
        &self.device_id0
    }
    #[doc = "0xffc - Chip Revision ID and Number"]
    #[inline(always)]
    pub const fn dieid(&self) -> &Dieid {
        &self.dieid
    }
}
#[doc = "REMAP (rw) register accessor: AHB Matrix Remap Control\n\nYou can [`read`](crate::Reg::read) this register and get [`remap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`remap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@remap`] module"]
#[doc(alias = "REMAP")]
pub type Remap = crate::Reg<remap::RemapSpec>;
#[doc = "AHB Matrix Remap Control"]
pub mod remap;
#[doc = "AHBMATPRIO (rw) register accessor: AHB Matrix Priority Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ahbmatprio::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ahbmatprio::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ahbmatprio`] module"]
#[doc(alias = "AHBMATPRIO")]
pub type Ahbmatprio = crate::Reg<ahbmatprio::AhbmatprioSpec>;
#[doc = "AHB Matrix Priority Control"]
pub mod ahbmatprio;
#[doc = "CPU0NSTCKCAL (rw) register accessor: Non-Secure CPU0 System Tick Calibration\n\nYou can [`read`](crate::Reg::read) this register and get [`cpu0nstckcal::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cpu0nstckcal::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cpu0nstckcal`] module"]
#[doc(alias = "CPU0NSTCKCAL")]
pub type Cpu0nstckcal = crate::Reg<cpu0nstckcal::Cpu0nstckcalSpec>;
#[doc = "Non-Secure CPU0 System Tick Calibration"]
pub mod cpu0nstckcal;
#[doc = "NMISRC (rw) register accessor: NMI Source Select\n\nYou can [`read`](crate::Reg::read) this register and get [`nmisrc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nmisrc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@nmisrc`] module"]
#[doc(alias = "NMISRC")]
pub type Nmisrc = crate::Reg<nmisrc::NmisrcSpec>;
#[doc = "NMI Source Select"]
pub mod nmisrc;
#[doc = "PROTLVL (rw) register accessor: Protect Level Control\n\nYou can [`read`](crate::Reg::read) this register and get [`protlvl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`protlvl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@protlvl`] module"]
#[doc(alias = "PROTLVL")]
pub type Protlvl = crate::Reg<protlvl::ProtlvlSpec>;
#[doc = "Protect Level Control"]
pub mod protlvl;
#[doc = "SLOWCLKDIV (rw) register accessor: SLOW_CLK Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`slowclkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`slowclkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@slowclkdiv`] module"]
#[doc(alias = "SLOWCLKDIV")]
pub type Slowclkdiv = crate::Reg<slowclkdiv::SlowclkdivSpec>;
#[doc = "SLOW_CLK Clock Divider"]
pub mod slowclkdiv;
#[doc = "BUSCLKDIV (rw) register accessor: BUS_CLK Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`busclkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`busclkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@busclkdiv`] module"]
#[doc(alias = "BUSCLKDIV")]
pub type Busclkdiv = crate::Reg<busclkdiv::BusclkdivSpec>;
#[doc = "BUS_CLK Clock Divider"]
pub mod busclkdiv;
#[doc = "AHBCLKDIV (rw) register accessor: System Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`ahbclkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ahbclkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ahbclkdiv`] module"]
#[doc(alias = "AHBCLKDIV")]
pub type Ahbclkdiv = crate::Reg<ahbclkdiv::AhbclkdivSpec>;
#[doc = "System Clock Divider"]
pub mod ahbclkdiv;
#[doc = "FROHFDIV (rw) register accessor: FRO_HF_DIV Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`frohfdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frohfdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frohfdiv`] module"]
#[doc(alias = "FROHFDIV")]
pub type Frohfdiv = crate::Reg<frohfdiv::FrohfdivSpec>;
#[doc = "FRO_HF_DIV Clock Divider"]
pub mod frohfdiv;
#[doc = "FROLFDIV (rw) register accessor: FRO_LF_DIV Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`frolfdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frolfdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frolfdiv`] module"]
#[doc(alias = "FROLFDIV")]
pub type Frolfdiv = crate::Reg<frolfdiv::FrolfdivSpec>;
#[doc = "FRO_LF_DIV Clock Divider"]
pub mod frolfdiv;
#[doc = "PLL1CLKDIV (rw) register accessor: PLL1_CLK_DIV Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`pll1clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pll1clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pll1clkdiv`] module"]
#[doc(alias = "PLL1CLKDIV")]
pub type Pll1clkdiv = crate::Reg<pll1clkdiv::Pll1clkdivSpec>;
#[doc = "PLL1_CLK_DIV Clock Divider"]
pub mod pll1clkdiv;
#[doc = "CLKUNLOCK (rw) register accessor: Clock Configuration Unlock\n\nYou can [`read`](crate::Reg::read) this register and get [`clkunlock::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clkunlock::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@clkunlock`] module"]
#[doc(alias = "CLKUNLOCK")]
pub type Clkunlock = crate::Reg<clkunlock::ClkunlockSpec>;
#[doc = "Clock Configuration Unlock"]
pub mod clkunlock;
#[doc = "NVM_CTRL (rw) register accessor: NVM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`nvm_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nvm_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@nvm_ctrl`] module"]
#[doc(alias = "NVM_CTRL")]
pub type NvmCtrl = crate::Reg<nvm_ctrl::NvmCtrlSpec>;
#[doc = "NVM Control"]
pub mod nvm_ctrl;
#[doc = "SmartDMAINT (rw) register accessor: SmartDMA Interrupt Hijack\n\nYou can [`read`](crate::Reg::read) this register and get [`smart_dmaint::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`smart_dmaint::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@smart_dmaint`] module"]
#[doc(alias = "SmartDMAINT")]
pub type SmartDmaint = crate::Reg<smart_dmaint::SmartDmaintSpec>;
#[doc = "SmartDMA Interrupt Hijack"]
pub mod smart_dmaint;
#[doc = "RAM_INTERLEAVE (rw) register accessor: Controls RAM Interleave Integration\n\nYou can [`read`](crate::Reg::read) this register and get [`ram_interleave::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ram_interleave::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ram_interleave`] module"]
#[doc(alias = "RAM_INTERLEAVE")]
pub type RamInterleave = crate::Reg<ram_interleave::RamInterleaveSpec>;
#[doc = "Controls RAM Interleave Integration"]
pub mod ram_interleave;
#[doc = "CPUSTAT (r) register accessor: CPU Status\n\nYou can [`read`](crate::Reg::read) this register and get [`cpustat::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cpustat`] module"]
#[doc(alias = "CPUSTAT")]
pub type Cpustat = crate::Reg<cpustat::CpustatSpec>;
#[doc = "CPU Status"]
pub mod cpustat;
#[doc = "LPCAC_CTRL (rw) register accessor: LPCAC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`lpcac_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpcac_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpcac_ctrl`] module"]
#[doc(alias = "LPCAC_CTRL")]
pub type LpcacCtrl = crate::Reg<lpcac_ctrl::LpcacCtrlSpec>;
#[doc = "LPCAC Control"]
pub mod lpcac_ctrl;
#[doc = "PWM0SUBCTL (rw) register accessor: PWM0 Submodule Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm0subctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm0subctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pwm0subctl`] module"]
#[doc(alias = "PWM0SUBCTL")]
pub type Pwm0subctl = crate::Reg<pwm0subctl::Pwm0subctlSpec>;
#[doc = "PWM0 Submodule Control"]
pub mod pwm0subctl;
#[doc = "PWM1SUBCTL (rw) register accessor: PWM1 Submodule Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm1subctl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm1subctl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pwm1subctl`] module"]
#[doc(alias = "PWM1SUBCTL")]
pub type Pwm1subctl = crate::Reg<pwm1subctl::Pwm1subctlSpec>;
#[doc = "PWM1 Submodule Control"]
pub mod pwm1subctl;
#[doc = "CTIMERGLOBALSTARTEN (rw) register accessor: CTIMER Global Start Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimerglobalstarten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimerglobalstarten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimerglobalstarten`] module"]
#[doc(alias = "CTIMERGLOBALSTARTEN")]
pub type Ctimerglobalstarten = crate::Reg<ctimerglobalstarten::CtimerglobalstartenSpec>;
#[doc = "CTIMER Global Start Enable"]
pub mod ctimerglobalstarten;
#[doc = "RAM_CTRL (rw) register accessor: RAM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ram_ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ram_ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ram_ctrl`] module"]
#[doc(alias = "RAM_CTRL")]
pub type RamCtrl = crate::Reg<ram_ctrl::RamCtrlSpec>;
#[doc = "RAM Control"]
pub mod ram_ctrl;
#[doc = "GRAY_CODE_LSB (rw) register accessor: Gray to Binary Converter Gray Code \\[31:0\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`gray_code_lsb::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gray_code_lsb::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gray_code_lsb`] module"]
#[doc(alias = "GRAY_CODE_LSB")]
pub type GrayCodeLsb = crate::Reg<gray_code_lsb::GrayCodeLsbSpec>;
#[doc = "Gray to Binary Converter Gray Code \\[31:0\\]"]
pub mod gray_code_lsb;
#[doc = "GRAY_CODE_MSB (rw) register accessor: Gray to Binary Converter Gray Code \\[41:32\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`gray_code_msb::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gray_code_msb::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gray_code_msb`] module"]
#[doc(alias = "GRAY_CODE_MSB")]
pub type GrayCodeMsb = crate::Reg<gray_code_msb::GrayCodeMsbSpec>;
#[doc = "Gray to Binary Converter Gray Code \\[41:32\\]"]
pub mod gray_code_msb;
#[doc = "BINARY_CODE_LSB (r) register accessor: Gray to Binary Converter Binary Code \\[31:0\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`binary_code_lsb::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@binary_code_lsb`] module"]
#[doc(alias = "BINARY_CODE_LSB")]
pub type BinaryCodeLsb = crate::Reg<binary_code_lsb::BinaryCodeLsbSpec>;
#[doc = "Gray to Binary Converter Binary Code \\[31:0\\]"]
pub mod binary_code_lsb;
#[doc = "BINARY_CODE_MSB (r) register accessor: Gray to Binary Converter Binary Code \\[41:32\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`binary_code_msb::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@binary_code_msb`] module"]
#[doc(alias = "BINARY_CODE_MSB")]
pub type BinaryCodeMsb = crate::Reg<binary_code_msb::BinaryCodeMsbSpec>;
#[doc = "Gray to Binary Converter Binary Code \\[41:32\\]"]
pub mod binary_code_msb;
#[doc = "ELS_UDF (rw) register accessor: UDF Control\n\nYou can [`read`](crate::Reg::read) this register and get [`els_udf::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`els_udf::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@els_udf`] module"]
#[doc(alias = "ELS_UDF")]
pub type ElsUdf = crate::Reg<els_udf::ElsUdfSpec>;
#[doc = "UDF Control"]
pub mod els_udf;
#[doc = "MSFCFG (rw) register accessor: MSF Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`msfcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`msfcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@msfcfg`] module"]
#[doc(alias = "MSFCFG")]
pub type Msfcfg = crate::Reg<msfcfg::MsfcfgSpec>;
#[doc = "MSF Configuration"]
pub mod msfcfg;
#[doc = "ELS_UID (rw) register accessor: Device UID n\n\nYou can [`read`](crate::Reg::read) this register and get [`els_uid::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`els_uid::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@els_uid`] module"]
#[doc(alias = "ELS_UID")]
pub type ElsUid = crate::Reg<els_uid::ElsUidSpec>;
#[doc = "Device UID n"]
pub mod els_uid;
#[doc = "ROP_STATE (r) register accessor: ROP State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rop_state::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@rop_state`] module"]
#[doc(alias = "ROP_STATE")]
pub type RopState = crate::Reg<rop_state::RopStateSpec>;
#[doc = "ROP State Register"]
pub mod rop_state;
#[doc = "SRAM_XEN (rw) register accessor: RAM XEN Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sram_xen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sram_xen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sram_xen`] module"]
#[doc(alias = "SRAM_XEN")]
pub type SramXen = crate::Reg<sram_xen::SramXenSpec>;
#[doc = "RAM XEN Control"]
pub mod sram_xen;
#[doc = "SRAM_XEN_DP (rw) register accessor: RAM XEN Control (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`sram_xen_dp::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sram_xen_dp::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sram_xen_dp`] module"]
#[doc(alias = "SRAM_XEN_DP")]
pub type SramXenDp = crate::Reg<sram_xen_dp::SramXenDpSpec>;
#[doc = "RAM XEN Control (Duplicate)"]
pub mod sram_xen_dp;
#[doc = "ELS_OTP_LC_STATE (r) register accessor: Life Cycle State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`els_otp_lc_state::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@els_otp_lc_state`] module"]
#[doc(alias = "ELS_OTP_LC_STATE")]
pub type ElsOtpLcState = crate::Reg<els_otp_lc_state::ElsOtpLcStateSpec>;
#[doc = "Life Cycle State Register"]
pub mod els_otp_lc_state;
#[doc = "ELS_OTP_LC_STATE_DP (r) register accessor: Life Cycle State Register (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`els_otp_lc_state_dp::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@els_otp_lc_state_dp`] module"]
#[doc(alias = "ELS_OTP_LC_STATE_DP")]
pub type ElsOtpLcStateDp = crate::Reg<els_otp_lc_state_dp::ElsOtpLcStateDpSpec>;
#[doc = "Life Cycle State Register (Duplicate)"]
pub mod els_otp_lc_state_dp;
#[doc = "DEBUG_LOCK_EN (rw) register accessor: Control Write Access to Security\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_lock_en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_lock_en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debug_lock_en`] module"]
#[doc(alias = "DEBUG_LOCK_EN")]
pub type DebugLockEn = crate::Reg<debug_lock_en::DebugLockEnSpec>;
#[doc = "Control Write Access to Security"]
pub mod debug_lock_en;
#[doc = "DEBUG_FEATURES (rw) register accessor: Cortex Debug Features Control\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_features::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_features::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debug_features`] module"]
#[doc(alias = "DEBUG_FEATURES")]
pub type DebugFeatures = crate::Reg<debug_features::DebugFeaturesSpec>;
#[doc = "Cortex Debug Features Control"]
pub mod debug_features;
#[doc = "DEBUG_FEATURES_DP (rw) register accessor: Cortex Debug Features Control (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_features_dp::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_features_dp::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debug_features_dp`] module"]
#[doc(alias = "DEBUG_FEATURES_DP")]
pub type DebugFeaturesDp = crate::Reg<debug_features_dp::DebugFeaturesDpSpec>;
#[doc = "Cortex Debug Features Control (Duplicate)"]
pub mod debug_features_dp;
#[doc = "SWD_ACCESS_CPU0 (rw) register accessor: CPU0 Software Debug Access\n\nYou can [`read`](crate::Reg::read) this register and get [`swd_access_cpu0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swd_access_cpu0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swd_access_cpu0`] module"]
#[doc(alias = "SWD_ACCESS_CPU0")]
pub type SwdAccessCpu0 = crate::Reg<swd_access_cpu0::SwdAccessCpu0Spec>;
#[doc = "CPU0 Software Debug Access"]
pub mod swd_access_cpu0;
#[doc = "DEBUG_AUTH_BEACON (rw) register accessor: Debug Authentication BEACON\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_auth_beacon::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_auth_beacon::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debug_auth_beacon`] module"]
#[doc(alias = "DEBUG_AUTH_BEACON")]
pub type DebugAuthBeacon = crate::Reg<debug_auth_beacon::DebugAuthBeaconSpec>;
#[doc = "Debug Authentication BEACON"]
pub mod debug_auth_beacon;
#[doc = "JTAG_ID (r) register accessor: JTAG Chip ID\n\nYou can [`read`](crate::Reg::read) this register and get [`jtag_id::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@jtag_id`] module"]
#[doc(alias = "JTAG_ID")]
pub type JtagId = crate::Reg<jtag_id::JtagIdSpec>;
#[doc = "JTAG Chip ID"]
pub mod jtag_id;
#[doc = "DEVICE_TYPE (r) register accessor: Device Type\n\nYou can [`read`](crate::Reg::read) this register and get [`device_type::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@device_type`] module"]
#[doc(alias = "DEVICE_TYPE")]
pub type DeviceType = crate::Reg<device_type::DeviceTypeSpec>;
#[doc = "Device Type"]
pub mod device_type;
#[doc = "DEVICE_ID0 (r) register accessor: Device ID\n\nYou can [`read`](crate::Reg::read) this register and get [`device_id0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@device_id0`] module"]
#[doc(alias = "DEVICE_ID0")]
pub type DeviceId0 = crate::Reg<device_id0::DeviceId0Spec>;
#[doc = "Device ID"]
pub mod device_id0;
#[doc = "DIEID (r) register accessor: Chip Revision ID and Number\n\nYou can [`read`](crate::Reg::read) this register and get [`dieid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dieid`] module"]
#[doc(alias = "DIEID")]
pub type Dieid = crate::Reg<dieid::DieidSpec>;
#[doc = "Chip Revision ID and Number"]
pub mod dieid;
