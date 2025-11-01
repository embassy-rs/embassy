#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ir: Ir,
    tcr: Tcr,
    tc: Tc,
    pr: Pr,
    pc: Pc,
    mcr: Mcr,
    mr: [Mr; 4],
    ccr: Ccr,
    cr: [Cr; 4],
    emr: Emr,
    _reserved10: [u8; 0x30],
    ctcr: Ctcr,
    pwmc: Pwmc,
    msr: [Msr; 4],
}
impl RegisterBlock {
    #[doc = "0x00 - Interrupt"]
    #[inline(always)]
    pub const fn ir(&self) -> &Ir {
        &self.ir
    }
    #[doc = "0x04 - Timer Control"]
    #[inline(always)]
    pub const fn tcr(&self) -> &Tcr {
        &self.tcr
    }
    #[doc = "0x08 - Timer Counter"]
    #[inline(always)]
    pub const fn tc(&self) -> &Tc {
        &self.tc
    }
    #[doc = "0x0c - Prescale"]
    #[inline(always)]
    pub const fn pr(&self) -> &Pr {
        &self.pr
    }
    #[doc = "0x10 - Prescale Counter"]
    #[inline(always)]
    pub const fn pc(&self) -> &Pc {
        &self.pc
    }
    #[doc = "0x14 - Match Control"]
    #[inline(always)]
    pub const fn mcr(&self) -> &Mcr {
        &self.mcr
    }
    #[doc = "0x18..0x28 - Match"]
    #[inline(always)]
    pub const fn mr(&self, n: usize) -> &Mr {
        &self.mr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x18..0x28 - Match"]
    #[inline(always)]
    pub fn mr_iter(&self) -> impl Iterator<Item = &Mr> {
        self.mr.iter()
    }
    #[doc = "0x28 - Capture Control"]
    #[inline(always)]
    pub const fn ccr(&self) -> &Ccr {
        &self.ccr
    }
    #[doc = "0x2c..0x3c - Capture"]
    #[inline(always)]
    pub const fn cr(&self, n: usize) -> &Cr {
        &self.cr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x2c..0x3c - Capture"]
    #[inline(always)]
    pub fn cr_iter(&self) -> impl Iterator<Item = &Cr> {
        self.cr.iter()
    }
    #[doc = "0x3c - External Match"]
    #[inline(always)]
    pub const fn emr(&self) -> &Emr {
        &self.emr
    }
    #[doc = "0x70 - Count Control"]
    #[inline(always)]
    pub const fn ctcr(&self) -> &Ctcr {
        &self.ctcr
    }
    #[doc = "0x74 - PWM Control"]
    #[inline(always)]
    pub const fn pwmc(&self) -> &Pwmc {
        &self.pwmc
    }
    #[doc = "0x78..0x88 - Match Shadow"]
    #[inline(always)]
    pub const fn msr(&self, n: usize) -> &Msr {
        &self.msr[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x78..0x88 - Match Shadow"]
    #[inline(always)]
    pub fn msr_iter(&self) -> impl Iterator<Item = &Msr> {
        self.msr.iter()
    }
}
#[doc = "IR (rw) register accessor: Interrupt\n\nYou can [`read`](crate::Reg::read) this register and get [`ir::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ir::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ir`] module"]
#[doc(alias = "IR")]
pub type Ir = crate::Reg<ir::IrSpec>;
#[doc = "Interrupt"]
pub mod ir;
#[doc = "TCR (rw) register accessor: Timer Control\n\nYou can [`read`](crate::Reg::read) this register and get [`tcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tcr`] module"]
#[doc(alias = "TCR")]
pub type Tcr = crate::Reg<tcr::TcrSpec>;
#[doc = "Timer Control"]
pub mod tcr;
#[doc = "TC (rw) register accessor: Timer Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`tc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@tc`] module"]
#[doc(alias = "TC")]
pub type Tc = crate::Reg<tc::TcSpec>;
#[doc = "Timer Counter"]
pub mod tc;
#[doc = "PR (rw) register accessor: Prescale\n\nYou can [`read`](crate::Reg::read) this register and get [`pr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pr`] module"]
#[doc(alias = "PR")]
pub type Pr = crate::Reg<pr::PrSpec>;
#[doc = "Prescale"]
pub mod pr;
#[doc = "PC (rw) register accessor: Prescale Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`pc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pc`] module"]
#[doc(alias = "PC")]
pub type Pc = crate::Reg<pc::PcSpec>;
#[doc = "Prescale Counter"]
pub mod pc;
#[doc = "MCR (rw) register accessor: Match Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mcr`] module"]
#[doc(alias = "MCR")]
pub type Mcr = crate::Reg<mcr::McrSpec>;
#[doc = "Match Control"]
pub mod mcr;
#[doc = "MR (rw) register accessor: Match\n\nYou can [`read`](crate::Reg::read) this register and get [`mr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mr`] module"]
#[doc(alias = "MR")]
pub type Mr = crate::Reg<mr::MrSpec>;
#[doc = "Match"]
pub mod mr;
#[doc = "CCR (rw) register accessor: Capture Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ccr`] module"]
#[doc(alias = "CCR")]
pub type Ccr = crate::Reg<ccr::CcrSpec>;
#[doc = "Capture Control"]
pub mod ccr;
#[doc = "CR (r) register accessor: Capture\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cr`] module"]
#[doc(alias = "CR")]
pub type Cr = crate::Reg<cr::CrSpec>;
#[doc = "Capture"]
pub mod cr;
#[doc = "EMR (rw) register accessor: External Match\n\nYou can [`read`](crate::Reg::read) this register and get [`emr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@emr`] module"]
#[doc(alias = "EMR")]
pub type Emr = crate::Reg<emr::EmrSpec>;
#[doc = "External Match"]
pub mod emr;
#[doc = "CTCR (rw) register accessor: Count Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctcr`] module"]
#[doc(alias = "CTCR")]
pub type Ctcr = crate::Reg<ctcr::CtcrSpec>;
#[doc = "Count Control"]
pub mod ctcr;
#[doc = "PWMC (rw) register accessor: PWM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pwmc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwmc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pwmc`] module"]
#[doc(alias = "PWMC")]
pub type Pwmc = crate::Reg<pwmc::PwmcSpec>;
#[doc = "PWM Control"]
pub mod pwmc;
#[doc = "MSR (rw) register accessor: Match Shadow\n\nYou can [`read`](crate::Reg::read) this register and get [`msr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`msr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@msr`] module"]
#[doc(alias = "MSR")]
pub type Msr = crate::Reg<msr::MsrSpec>;
#[doc = "Match Shadow"]
pub mod msr;
