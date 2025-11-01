#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x20],
    bootadr: Bootadr,
    ctrl: Ctrl,
    pc: Pc,
    sp: Sp,
    break_addr: BreakAddr,
    break_vect: BreakVect,
    emer_vect: EmerVect,
    emer_sel: EmerSel,
    arm2ezh: Arm2ezh,
    ezh2arm: Ezh2arm,
    pendtrap: Pendtrap,
}
impl RegisterBlock {
    #[doc = "0x20 - Boot Address"]
    #[inline(always)]
    pub const fn bootadr(&self) -> &Bootadr {
        &self.bootadr
    }
    #[doc = "0x24 - Control"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x28 - Program Counter"]
    #[inline(always)]
    pub const fn pc(&self) -> &Pc {
        &self.pc
    }
    #[doc = "0x2c - Stack Pointer"]
    #[inline(always)]
    pub const fn sp(&self) -> &Sp {
        &self.sp
    }
    #[doc = "0x30 - Breakpoint Address"]
    #[inline(always)]
    pub const fn break_addr(&self) -> &BreakAddr {
        &self.break_addr
    }
    #[doc = "0x34 - Breakpoint Vector"]
    #[inline(always)]
    pub const fn break_vect(&self) -> &BreakVect {
        &self.break_vect
    }
    #[doc = "0x38 - Emergency Vector"]
    #[inline(always)]
    pub const fn emer_vect(&self) -> &EmerVect {
        &self.emer_vect
    }
    #[doc = "0x3c - Emergency Select"]
    #[inline(always)]
    pub const fn emer_sel(&self) -> &EmerSel {
        &self.emer_sel
    }
    #[doc = "0x40 - ARM to EZH Interrupt Control"]
    #[inline(always)]
    pub const fn arm2ezh(&self) -> &Arm2ezh {
        &self.arm2ezh
    }
    #[doc = "0x44 - EZH to ARM Trigger"]
    #[inline(always)]
    pub const fn ezh2arm(&self) -> &Ezh2arm {
        &self.ezh2arm
    }
    #[doc = "0x48 - Pending Trap Control"]
    #[inline(always)]
    pub const fn pendtrap(&self) -> &Pendtrap {
        &self.pendtrap
    }
}
#[doc = "BOOTADR (rw) register accessor: Boot Address\n\nYou can [`read`](crate::Reg::read) this register and get [`bootadr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bootadr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@bootadr`] module"]
#[doc(alias = "BOOTADR")]
pub type Bootadr = crate::Reg<bootadr::BootadrSpec>;
#[doc = "Boot Address"]
pub mod bootadr;
#[doc = "CTRL (rw) register accessor: Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Control"]
pub mod ctrl;
#[doc = "PC (r) register accessor: Program Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`pc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pc`] module"]
#[doc(alias = "PC")]
pub type Pc = crate::Reg<pc::PcSpec>;
#[doc = "Program Counter"]
pub mod pc;
#[doc = "SP (r) register accessor: Stack Pointer\n\nYou can [`read`](crate::Reg::read) this register and get [`sp::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sp`] module"]
#[doc(alias = "SP")]
pub type Sp = crate::Reg<sp::SpSpec>;
#[doc = "Stack Pointer"]
pub mod sp;
#[doc = "BREAK_ADDR (rw) register accessor: Breakpoint Address\n\nYou can [`read`](crate::Reg::read) this register and get [`break_addr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`break_addr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@break_addr`] module"]
#[doc(alias = "BREAK_ADDR")]
pub type BreakAddr = crate::Reg<break_addr::BreakAddrSpec>;
#[doc = "Breakpoint Address"]
pub mod break_addr;
#[doc = "BREAK_VECT (rw) register accessor: Breakpoint Vector\n\nYou can [`read`](crate::Reg::read) this register and get [`break_vect::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`break_vect::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@break_vect`] module"]
#[doc(alias = "BREAK_VECT")]
pub type BreakVect = crate::Reg<break_vect::BreakVectSpec>;
#[doc = "Breakpoint Vector"]
pub mod break_vect;
#[doc = "EMER_VECT (rw) register accessor: Emergency Vector\n\nYou can [`read`](crate::Reg::read) this register and get [`emer_vect::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emer_vect::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@emer_vect`] module"]
#[doc(alias = "EMER_VECT")]
pub type EmerVect = crate::Reg<emer_vect::EmerVectSpec>;
#[doc = "Emergency Vector"]
pub mod emer_vect;
#[doc = "EMER_SEL (rw) register accessor: Emergency Select\n\nYou can [`read`](crate::Reg::read) this register and get [`emer_sel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emer_sel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@emer_sel`] module"]
#[doc(alias = "EMER_SEL")]
pub type EmerSel = crate::Reg<emer_sel::EmerSelSpec>;
#[doc = "Emergency Select"]
pub mod emer_sel;
#[doc = "ARM2EZH (rw) register accessor: ARM to EZH Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`arm2ezh::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`arm2ezh::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@arm2ezh`] module"]
#[doc(alias = "ARM2EZH")]
pub type Arm2ezh = crate::Reg<arm2ezh::Arm2ezhSpec>;
#[doc = "ARM to EZH Interrupt Control"]
pub mod arm2ezh;
#[doc = "EZH2ARM (rw) register accessor: EZH to ARM Trigger\n\nYou can [`read`](crate::Reg::read) this register and get [`ezh2arm::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ezh2arm::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ezh2arm`] module"]
#[doc(alias = "EZH2ARM")]
pub type Ezh2arm = crate::Reg<ezh2arm::Ezh2armSpec>;
#[doc = "EZH to ARM Trigger"]
pub mod ezh2arm;
#[doc = "PENDTRAP (rw) register accessor: Pending Trap Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pendtrap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pendtrap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pendtrap`] module"]
#[doc(alias = "PENDTRAP")]
pub type Pendtrap = crate::Reg<pendtrap::PendtrapSpec>;
#[doc = "Pending Trap Control"]
pub mod pendtrap;
