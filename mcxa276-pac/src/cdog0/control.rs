#[doc = "Register `CONTROL` reader"]
pub type R = crate::R<ControlSpec>;
#[doc = "Register `CONTROL` writer"]
pub type W = crate::W<ControlSpec>;
#[doc = "Lock control\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LockCtrl {
    #[doc = "1: Locked"]
    Locked = 1,
    #[doc = "2: Unlocked"]
    Unlocked = 2,
}
impl From<LockCtrl> for u8 {
    #[inline(always)]
    fn from(variant: LockCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for LockCtrl {
    type Ux = u8;
}
impl crate::IsEnum for LockCtrl {}
#[doc = "Field `LOCK_CTRL` reader - Lock control"]
pub type LockCtrlR = crate::FieldReader<LockCtrl>;
impl LockCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<LockCtrl> {
        match self.bits {
            1 => Some(LockCtrl::Locked),
            2 => Some(LockCtrl::Unlocked),
            _ => None,
        }
    }
    #[doc = "Locked"]
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        *self == LockCtrl::Locked
    }
    #[doc = "Unlocked"]
    #[inline(always)]
    pub fn is_unlocked(&self) -> bool {
        *self == LockCtrl::Unlocked
    }
}
#[doc = "Field `LOCK_CTRL` writer - Lock control"]
pub type LockCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 2, LockCtrl>;
impl<'a, REG> LockCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Locked"]
    #[inline(always)]
    pub fn locked(self) -> &'a mut crate::W<REG> {
        self.variant(LockCtrl::Locked)
    }
    #[doc = "Unlocked"]
    #[inline(always)]
    pub fn unlocked(self) -> &'a mut crate::W<REG> {
        self.variant(LockCtrl::Unlocked)
    }
}
#[doc = "TIMEOUT fault control\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeoutCtrl {
    #[doc = "1: Enable reset"]
    EnableReset = 1,
    #[doc = "2: Enable interrupt"]
    EnableInterrupt = 2,
    #[doc = "4: Disable both reset and interrupt"]
    DisableBoth = 4,
}
impl From<TimeoutCtrl> for u8 {
    #[inline(always)]
    fn from(variant: TimeoutCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for TimeoutCtrl {
    type Ux = u8;
}
impl crate::IsEnum for TimeoutCtrl {}
#[doc = "Field `TIMEOUT_CTRL` reader - TIMEOUT fault control"]
pub type TimeoutCtrlR = crate::FieldReader<TimeoutCtrl>;
impl TimeoutCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<TimeoutCtrl> {
        match self.bits {
            1 => Some(TimeoutCtrl::EnableReset),
            2 => Some(TimeoutCtrl::EnableInterrupt),
            4 => Some(TimeoutCtrl::DisableBoth),
            _ => None,
        }
    }
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn is_enable_reset(&self) -> bool {
        *self == TimeoutCtrl::EnableReset
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn is_enable_interrupt(&self) -> bool {
        *self == TimeoutCtrl::EnableInterrupt
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn is_disable_both(&self) -> bool {
        *self == TimeoutCtrl::DisableBoth
    }
}
#[doc = "Field `TIMEOUT_CTRL` writer - TIMEOUT fault control"]
pub type TimeoutCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 3, TimeoutCtrl>;
impl<'a, REG> TimeoutCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn enable_reset(self) -> &'a mut crate::W<REG> {
        self.variant(TimeoutCtrl::EnableReset)
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn enable_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(TimeoutCtrl::EnableInterrupt)
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn disable_both(self) -> &'a mut crate::W<REG> {
        self.variant(TimeoutCtrl::DisableBoth)
    }
}
#[doc = "MISCOMPARE fault control\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MiscompareCtrl {
    #[doc = "1: Enable reset"]
    EnableReset = 1,
    #[doc = "2: Enable interrupt"]
    EnableInterrupt = 2,
    #[doc = "4: Disable both reset and interrupt"]
    DisableBoth = 4,
}
impl From<MiscompareCtrl> for u8 {
    #[inline(always)]
    fn from(variant: MiscompareCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for MiscompareCtrl {
    type Ux = u8;
}
impl crate::IsEnum for MiscompareCtrl {}
#[doc = "Field `MISCOMPARE_CTRL` reader - MISCOMPARE fault control"]
pub type MiscompareCtrlR = crate::FieldReader<MiscompareCtrl>;
impl MiscompareCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<MiscompareCtrl> {
        match self.bits {
            1 => Some(MiscompareCtrl::EnableReset),
            2 => Some(MiscompareCtrl::EnableInterrupt),
            4 => Some(MiscompareCtrl::DisableBoth),
            _ => None,
        }
    }
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn is_enable_reset(&self) -> bool {
        *self == MiscompareCtrl::EnableReset
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn is_enable_interrupt(&self) -> bool {
        *self == MiscompareCtrl::EnableInterrupt
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn is_disable_both(&self) -> bool {
        *self == MiscompareCtrl::DisableBoth
    }
}
#[doc = "Field `MISCOMPARE_CTRL` writer - MISCOMPARE fault control"]
pub type MiscompareCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 3, MiscompareCtrl>;
impl<'a, REG> MiscompareCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn enable_reset(self) -> &'a mut crate::W<REG> {
        self.variant(MiscompareCtrl::EnableReset)
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn enable_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(MiscompareCtrl::EnableInterrupt)
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn disable_both(self) -> &'a mut crate::W<REG> {
        self.variant(MiscompareCtrl::DisableBoth)
    }
}
#[doc = "SEQUENCE fault control\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SequenceCtrl {
    #[doc = "1: Enable reset"]
    EnableReset = 1,
    #[doc = "2: Enable interrupt"]
    EnableInterrupt = 2,
    #[doc = "4: Disable both reset and interrupt"]
    DisableBoth = 4,
}
impl From<SequenceCtrl> for u8 {
    #[inline(always)]
    fn from(variant: SequenceCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for SequenceCtrl {
    type Ux = u8;
}
impl crate::IsEnum for SequenceCtrl {}
#[doc = "Field `SEQUENCE_CTRL` reader - SEQUENCE fault control"]
pub type SequenceCtrlR = crate::FieldReader<SequenceCtrl>;
impl SequenceCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<SequenceCtrl> {
        match self.bits {
            1 => Some(SequenceCtrl::EnableReset),
            2 => Some(SequenceCtrl::EnableInterrupt),
            4 => Some(SequenceCtrl::DisableBoth),
            _ => None,
        }
    }
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn is_enable_reset(&self) -> bool {
        *self == SequenceCtrl::EnableReset
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn is_enable_interrupt(&self) -> bool {
        *self == SequenceCtrl::EnableInterrupt
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn is_disable_both(&self) -> bool {
        *self == SequenceCtrl::DisableBoth
    }
}
#[doc = "Field `SEQUENCE_CTRL` writer - SEQUENCE fault control"]
pub type SequenceCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 3, SequenceCtrl>;
impl<'a, REG> SequenceCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn enable_reset(self) -> &'a mut crate::W<REG> {
        self.variant(SequenceCtrl::EnableReset)
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn enable_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(SequenceCtrl::EnableInterrupt)
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn disable_both(self) -> &'a mut crate::W<REG> {
        self.variant(SequenceCtrl::DisableBoth)
    }
}
#[doc = "STATE fault control\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StateCtrl {
    #[doc = "1: Enable reset"]
    EnableReset = 1,
    #[doc = "2: Enable interrupt"]
    EnableInterrupt = 2,
    #[doc = "4: Disable both reset and interrupt"]
    DisableBoth = 4,
}
impl From<StateCtrl> for u8 {
    #[inline(always)]
    fn from(variant: StateCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for StateCtrl {
    type Ux = u8;
}
impl crate::IsEnum for StateCtrl {}
#[doc = "Field `STATE_CTRL` reader - STATE fault control"]
pub type StateCtrlR = crate::FieldReader<StateCtrl>;
impl StateCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<StateCtrl> {
        match self.bits {
            1 => Some(StateCtrl::EnableReset),
            2 => Some(StateCtrl::EnableInterrupt),
            4 => Some(StateCtrl::DisableBoth),
            _ => None,
        }
    }
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn is_enable_reset(&self) -> bool {
        *self == StateCtrl::EnableReset
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn is_enable_interrupt(&self) -> bool {
        *self == StateCtrl::EnableInterrupt
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn is_disable_both(&self) -> bool {
        *self == StateCtrl::DisableBoth
    }
}
#[doc = "Field `STATE_CTRL` writer - STATE fault control"]
pub type StateCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 3, StateCtrl>;
impl<'a, REG> StateCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn enable_reset(self) -> &'a mut crate::W<REG> {
        self.variant(StateCtrl::EnableReset)
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn enable_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(StateCtrl::EnableInterrupt)
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn disable_both(self) -> &'a mut crate::W<REG> {
        self.variant(StateCtrl::DisableBoth)
    }
}
#[doc = "ADDRESS fault control\n\nValue on reset: 4"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressCtrl {
    #[doc = "1: Enable reset"]
    EnableReset = 1,
    #[doc = "2: Enable interrupt"]
    EnableInterrupt = 2,
    #[doc = "4: Disable both reset and interrupt"]
    DisableBoth = 4,
}
impl From<AddressCtrl> for u8 {
    #[inline(always)]
    fn from(variant: AddressCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for AddressCtrl {
    type Ux = u8;
}
impl crate::IsEnum for AddressCtrl {}
#[doc = "Field `ADDRESS_CTRL` reader - ADDRESS fault control"]
pub type AddressCtrlR = crate::FieldReader<AddressCtrl>;
impl AddressCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<AddressCtrl> {
        match self.bits {
            1 => Some(AddressCtrl::EnableReset),
            2 => Some(AddressCtrl::EnableInterrupt),
            4 => Some(AddressCtrl::DisableBoth),
            _ => None,
        }
    }
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn is_enable_reset(&self) -> bool {
        *self == AddressCtrl::EnableReset
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn is_enable_interrupt(&self) -> bool {
        *self == AddressCtrl::EnableInterrupt
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn is_disable_both(&self) -> bool {
        *self == AddressCtrl::DisableBoth
    }
}
#[doc = "Field `ADDRESS_CTRL` writer - ADDRESS fault control"]
pub type AddressCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 3, AddressCtrl>;
impl<'a, REG> AddressCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Enable reset"]
    #[inline(always)]
    pub fn enable_reset(self) -> &'a mut crate::W<REG> {
        self.variant(AddressCtrl::EnableReset)
    }
    #[doc = "Enable interrupt"]
    #[inline(always)]
    pub fn enable_interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(AddressCtrl::EnableInterrupt)
    }
    #[doc = "Disable both reset and interrupt"]
    #[inline(always)]
    pub fn disable_both(self) -> &'a mut crate::W<REG> {
        self.variant(AddressCtrl::DisableBoth)
    }
}
#[doc = "IRQ pause control\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IrqPause {
    #[doc = "1: Keep the timer running"]
    RunTimer = 1,
    #[doc = "2: Stop the timer"]
    PauseTimer = 2,
}
impl From<IrqPause> for u8 {
    #[inline(always)]
    fn from(variant: IrqPause) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for IrqPause {
    type Ux = u8;
}
impl crate::IsEnum for IrqPause {}
#[doc = "Field `IRQ_PAUSE` reader - IRQ pause control"]
pub type IrqPauseR = crate::FieldReader<IrqPause>;
impl IrqPauseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<IrqPause> {
        match self.bits {
            1 => Some(IrqPause::RunTimer),
            2 => Some(IrqPause::PauseTimer),
            _ => None,
        }
    }
    #[doc = "Keep the timer running"]
    #[inline(always)]
    pub fn is_run_timer(&self) -> bool {
        *self == IrqPause::RunTimer
    }
    #[doc = "Stop the timer"]
    #[inline(always)]
    pub fn is_pause_timer(&self) -> bool {
        *self == IrqPause::PauseTimer
    }
}
#[doc = "Field `IRQ_PAUSE` writer - IRQ pause control"]
pub type IrqPauseW<'a, REG> = crate::FieldWriter<'a, REG, 2, IrqPause>;
impl<'a, REG> IrqPauseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Keep the timer running"]
    #[inline(always)]
    pub fn run_timer(self) -> &'a mut crate::W<REG> {
        self.variant(IrqPause::RunTimer)
    }
    #[doc = "Stop the timer"]
    #[inline(always)]
    pub fn pause_timer(self) -> &'a mut crate::W<REG> {
        self.variant(IrqPause::PauseTimer)
    }
}
#[doc = "DEBUG_HALT control\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DebugHaltCtrl {
    #[doc = "1: Keep the timer running"]
    RunTimer = 1,
    #[doc = "2: Stop the timer"]
    PauseTimer = 2,
}
impl From<DebugHaltCtrl> for u8 {
    #[inline(always)]
    fn from(variant: DebugHaltCtrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for DebugHaltCtrl {
    type Ux = u8;
}
impl crate::IsEnum for DebugHaltCtrl {}
#[doc = "Field `DEBUG_HALT_CTRL` reader - DEBUG_HALT control"]
pub type DebugHaltCtrlR = crate::FieldReader<DebugHaltCtrl>;
impl DebugHaltCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<DebugHaltCtrl> {
        match self.bits {
            1 => Some(DebugHaltCtrl::RunTimer),
            2 => Some(DebugHaltCtrl::PauseTimer),
            _ => None,
        }
    }
    #[doc = "Keep the timer running"]
    #[inline(always)]
    pub fn is_run_timer(&self) -> bool {
        *self == DebugHaltCtrl::RunTimer
    }
    #[doc = "Stop the timer"]
    #[inline(always)]
    pub fn is_pause_timer(&self) -> bool {
        *self == DebugHaltCtrl::PauseTimer
    }
}
#[doc = "Field `DEBUG_HALT_CTRL` writer - DEBUG_HALT control"]
pub type DebugHaltCtrlW<'a, REG> = crate::FieldWriter<'a, REG, 2, DebugHaltCtrl>;
impl<'a, REG> DebugHaltCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Keep the timer running"]
    #[inline(always)]
    pub fn run_timer(self) -> &'a mut crate::W<REG> {
        self.variant(DebugHaltCtrl::RunTimer)
    }
    #[doc = "Stop the timer"]
    #[inline(always)]
    pub fn pause_timer(self) -> &'a mut crate::W<REG> {
        self.variant(DebugHaltCtrl::PauseTimer)
    }
}
impl R {
    #[doc = "Bits 0:1 - Lock control"]
    #[inline(always)]
    pub fn lock_ctrl(&self) -> LockCtrlR {
        LockCtrlR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:4 - TIMEOUT fault control"]
    #[inline(always)]
    pub fn timeout_ctrl(&self) -> TimeoutCtrlR {
        TimeoutCtrlR::new(((self.bits >> 2) & 7) as u8)
    }
    #[doc = "Bits 5:7 - MISCOMPARE fault control"]
    #[inline(always)]
    pub fn miscompare_ctrl(&self) -> MiscompareCtrlR {
        MiscompareCtrlR::new(((self.bits >> 5) & 7) as u8)
    }
    #[doc = "Bits 8:10 - SEQUENCE fault control"]
    #[inline(always)]
    pub fn sequence_ctrl(&self) -> SequenceCtrlR {
        SequenceCtrlR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 14:16 - STATE fault control"]
    #[inline(always)]
    pub fn state_ctrl(&self) -> StateCtrlR {
        StateCtrlR::new(((self.bits >> 14) & 7) as u8)
    }
    #[doc = "Bits 17:19 - ADDRESS fault control"]
    #[inline(always)]
    pub fn address_ctrl(&self) -> AddressCtrlR {
        AddressCtrlR::new(((self.bits >> 17) & 7) as u8)
    }
    #[doc = "Bits 28:29 - IRQ pause control"]
    #[inline(always)]
    pub fn irq_pause(&self) -> IrqPauseR {
        IrqPauseR::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - DEBUG_HALT control"]
    #[inline(always)]
    pub fn debug_halt_ctrl(&self) -> DebugHaltCtrlR {
        DebugHaltCtrlR::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Lock control"]
    #[inline(always)]
    pub fn lock_ctrl(&mut self) -> LockCtrlW<ControlSpec> {
        LockCtrlW::new(self, 0)
    }
    #[doc = "Bits 2:4 - TIMEOUT fault control"]
    #[inline(always)]
    pub fn timeout_ctrl(&mut self) -> TimeoutCtrlW<ControlSpec> {
        TimeoutCtrlW::new(self, 2)
    }
    #[doc = "Bits 5:7 - MISCOMPARE fault control"]
    #[inline(always)]
    pub fn miscompare_ctrl(&mut self) -> MiscompareCtrlW<ControlSpec> {
        MiscompareCtrlW::new(self, 5)
    }
    #[doc = "Bits 8:10 - SEQUENCE fault control"]
    #[inline(always)]
    pub fn sequence_ctrl(&mut self) -> SequenceCtrlW<ControlSpec> {
        SequenceCtrlW::new(self, 8)
    }
    #[doc = "Bits 14:16 - STATE fault control"]
    #[inline(always)]
    pub fn state_ctrl(&mut self) -> StateCtrlW<ControlSpec> {
        StateCtrlW::new(self, 14)
    }
    #[doc = "Bits 17:19 - ADDRESS fault control"]
    #[inline(always)]
    pub fn address_ctrl(&mut self) -> AddressCtrlW<ControlSpec> {
        AddressCtrlW::new(self, 17)
    }
    #[doc = "Bits 28:29 - IRQ pause control"]
    #[inline(always)]
    pub fn irq_pause(&mut self) -> IrqPauseW<ControlSpec> {
        IrqPauseW::new(self, 28)
    }
    #[doc = "Bits 30:31 - DEBUG_HALT control"]
    #[inline(always)]
    pub fn debug_halt_ctrl(&mut self) -> DebugHaltCtrlW<ControlSpec> {
        DebugHaltCtrlW::new(self, 30)
    }
}
#[doc = "Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`control::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ControlSpec;
impl crate::RegisterSpec for ControlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`control::R`](R) reader structure"]
impl crate::Readable for ControlSpec {}
#[doc = "`write(|w| ..)` method takes [`control::W`](W) writer structure"]
impl crate::Writable for ControlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONTROL to value 0x5009_2492"]
impl crate::Resettable for ControlSpec {
    const RESET_VALUE: u32 = 0x5009_2492;
}
