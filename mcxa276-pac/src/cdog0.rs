#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    control: Control,
    reload: Reload,
    instruction_timer: InstructionTimer,
    _reserved3: [u8; 0x04],
    status: Status,
    status2: Status2,
    flags: Flags,
    persistent: Persistent,
    start: Start,
    stop: Stop,
    restart: Restart,
    add: Add,
    add1: Add1,
    add16: Add16,
    add256: Add256,
    sub: Sub,
    sub1: Sub1,
    sub16: Sub16,
    sub256: Sub256,
    assert16: Assert16,
}
impl RegisterBlock {
    #[doc = "0x00 - Control Register"]
    #[inline(always)]
    pub const fn control(&self) -> &Control {
        &self.control
    }
    #[doc = "0x04 - Instruction Timer Reload Register"]
    #[inline(always)]
    pub const fn reload(&self) -> &Reload {
        &self.reload
    }
    #[doc = "0x08 - Instruction Timer Register"]
    #[inline(always)]
    pub const fn instruction_timer(&self) -> &InstructionTimer {
        &self.instruction_timer
    }
    #[doc = "0x10 - Status 1 Register"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x14 - Status 2 Register"]
    #[inline(always)]
    pub const fn status2(&self) -> &Status2 {
        &self.status2
    }
    #[doc = "0x18 - Flags Register"]
    #[inline(always)]
    pub const fn flags(&self) -> &Flags {
        &self.flags
    }
    #[doc = "0x1c - Persistent Data Storage Register"]
    #[inline(always)]
    pub const fn persistent(&self) -> &Persistent {
        &self.persistent
    }
    #[doc = "0x20 - START Command Register"]
    #[inline(always)]
    pub const fn start(&self) -> &Start {
        &self.start
    }
    #[doc = "0x24 - STOP Command Register"]
    #[inline(always)]
    pub const fn stop(&self) -> &Stop {
        &self.stop
    }
    #[doc = "0x28 - RESTART Command Register"]
    #[inline(always)]
    pub const fn restart(&self) -> &Restart {
        &self.restart
    }
    #[doc = "0x2c - ADD Command Register"]
    #[inline(always)]
    pub const fn add(&self) -> &Add {
        &self.add
    }
    #[doc = "0x30 - ADD1 Command Register"]
    #[inline(always)]
    pub const fn add1(&self) -> &Add1 {
        &self.add1
    }
    #[doc = "0x34 - ADD16 Command Register"]
    #[inline(always)]
    pub const fn add16(&self) -> &Add16 {
        &self.add16
    }
    #[doc = "0x38 - ADD256 Command Register"]
    #[inline(always)]
    pub const fn add256(&self) -> &Add256 {
        &self.add256
    }
    #[doc = "0x3c - SUB Command Register"]
    #[inline(always)]
    pub const fn sub(&self) -> &Sub {
        &self.sub
    }
    #[doc = "0x40 - SUB1 Command Register"]
    #[inline(always)]
    pub const fn sub1(&self) -> &Sub1 {
        &self.sub1
    }
    #[doc = "0x44 - SUB16 Command Register"]
    #[inline(always)]
    pub const fn sub16(&self) -> &Sub16 {
        &self.sub16
    }
    #[doc = "0x48 - SUB256 Command Register"]
    #[inline(always)]
    pub const fn sub256(&self) -> &Sub256 {
        &self.sub256
    }
    #[doc = "0x4c - ASSERT16 Command Register"]
    #[inline(always)]
    pub const fn assert16(&self) -> &Assert16 {
        &self.assert16
    }
}
#[doc = "CONTROL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`control::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@control`] module"]
#[doc(alias = "CONTROL")]
pub type Control = crate::Reg<control::ControlSpec>;
#[doc = "Control Register"]
pub mod control;
#[doc = "RELOAD (rw) register accessor: Instruction Timer Reload Register\n\nYou can [`read`](crate::Reg::read) this register and get [`reload::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`reload::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@reload`] module"]
#[doc(alias = "RELOAD")]
pub type Reload = crate::Reg<reload::ReloadSpec>;
#[doc = "Instruction Timer Reload Register"]
pub mod reload;
#[doc = "INSTRUCTION_TIMER (r) register accessor: Instruction Timer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`instruction_timer::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instruction_timer`] module"]
#[doc(alias = "INSTRUCTION_TIMER")]
pub type InstructionTimer = crate::Reg<instruction_timer::InstructionTimerSpec>;
#[doc = "Instruction Timer Register"]
pub mod instruction_timer;
#[doc = "STATUS (r) register accessor: Status 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "Status 1 Register"]
pub mod status;
#[doc = "STATUS2 (r) register accessor: Status 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status2`] module"]
#[doc(alias = "STATUS2")]
pub type Status2 = crate::Reg<status2::Status2Spec>;
#[doc = "Status 2 Register"]
pub mod status2;
#[doc = "FLAGS (rw) register accessor: Flags Register\n\nYou can [`read`](crate::Reg::read) this register and get [`flags::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flags::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flags`] module"]
#[doc(alias = "FLAGS")]
pub type Flags = crate::Reg<flags::FlagsSpec>;
#[doc = "Flags Register"]
pub mod flags;
#[doc = "PERSISTENT (rw) register accessor: Persistent Data Storage Register\n\nYou can [`read`](crate::Reg::read) this register and get [`persistent::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`persistent::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@persistent`] module"]
#[doc(alias = "PERSISTENT")]
pub type Persistent = crate::Reg<persistent::PersistentSpec>;
#[doc = "Persistent Data Storage Register"]
pub mod persistent;
#[doc = "START (w) register accessor: START Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`start::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@start`] module"]
#[doc(alias = "START")]
pub type Start = crate::Reg<start::StartSpec>;
#[doc = "START Command Register"]
pub mod start;
#[doc = "STOP (w) register accessor: STOP Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stop::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@stop`] module"]
#[doc(alias = "STOP")]
pub type Stop = crate::Reg<stop::StopSpec>;
#[doc = "STOP Command Register"]
pub mod stop;
#[doc = "RESTART (w) register accessor: RESTART Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`restart::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@restart`] module"]
#[doc(alias = "RESTART")]
pub type Restart = crate::Reg<restart::RestartSpec>;
#[doc = "RESTART Command Register"]
pub mod restart;
#[doc = "ADD (w) register accessor: ADD Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@add`] module"]
#[doc(alias = "ADD")]
pub type Add = crate::Reg<add::AddSpec>;
#[doc = "ADD Command Register"]
pub mod add;
#[doc = "ADD1 (w) register accessor: ADD1 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@add1`] module"]
#[doc(alias = "ADD1")]
pub type Add1 = crate::Reg<add1::Add1Spec>;
#[doc = "ADD1 Command Register"]
pub mod add1;
#[doc = "ADD16 (w) register accessor: ADD16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add16::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@add16`] module"]
#[doc(alias = "ADD16")]
pub type Add16 = crate::Reg<add16::Add16Spec>;
#[doc = "ADD16 Command Register"]
pub mod add16;
#[doc = "ADD256 (w) register accessor: ADD256 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add256::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@add256`] module"]
#[doc(alias = "ADD256")]
pub type Add256 = crate::Reg<add256::Add256Spec>;
#[doc = "ADD256 Command Register"]
pub mod add256;
#[doc = "SUB (w) register accessor: SUB Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sub`] module"]
#[doc(alias = "SUB")]
pub type Sub = crate::Reg<sub::SubSpec>;
#[doc = "SUB Command Register"]
pub mod sub;
#[doc = "SUB1 (w) register accessor: SUB1 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub1::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sub1`] module"]
#[doc(alias = "SUB1")]
pub type Sub1 = crate::Reg<sub1::Sub1Spec>;
#[doc = "SUB1 Command Register"]
pub mod sub1;
#[doc = "SUB16 (w) register accessor: SUB16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub16::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sub16`] module"]
#[doc(alias = "SUB16")]
pub type Sub16 = crate::Reg<sub16::Sub16Spec>;
#[doc = "SUB16 Command Register"]
pub mod sub16;
#[doc = "SUB256 (w) register accessor: SUB256 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub256::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sub256`] module"]
#[doc(alias = "SUB256")]
pub type Sub256 = crate::Reg<sub256::Sub256Spec>;
#[doc = "SUB256 Command Register"]
pub mod sub256;
#[doc = "ASSERT16 (w) register accessor: ASSERT16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`assert16::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@assert16`] module"]
#[doc(alias = "ASSERT16")]
pub type Assert16 = crate::Reg<assert16::Assert16Spec>;
#[doc = "ASSERT16 Command Register"]
pub mod assert16;
