#[doc = "Register `MOD` reader"]
pub type R = crate::R<ModSpec>;
#[doc = "Register `MOD` writer"]
pub type W = crate::W<ModSpec>;
#[doc = "Watchdog Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wden {
    #[doc = "0: Timer stopped"]
    Stop = 0,
    #[doc = "1: Timer running"]
    Run = 1,
}
impl From<Wden> for bool {
    #[inline(always)]
    fn from(variant: Wden) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDEN` reader - Watchdog Enable"]
pub type WdenR = crate::BitReader<Wden>;
impl WdenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wden {
        match self.bits {
            false => Wden::Stop,
            true => Wden::Run,
        }
    }
    #[doc = "Timer stopped"]
    #[inline(always)]
    pub fn is_stop(&self) -> bool {
        *self == Wden::Stop
    }
    #[doc = "Timer running"]
    #[inline(always)]
    pub fn is_run(&self) -> bool {
        *self == Wden::Run
    }
}
#[doc = "Field `WDEN` writer - Watchdog Enable"]
pub type WdenW<'a, REG> = crate::BitWriter<'a, REG, Wden>;
impl<'a, REG> WdenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Timer stopped"]
    #[inline(always)]
    pub fn stop(self) -> &'a mut crate::W<REG> {
        self.variant(Wden::Stop)
    }
    #[doc = "Timer running"]
    #[inline(always)]
    pub fn run(self) -> &'a mut crate::W<REG> {
        self.variant(Wden::Run)
    }
}
#[doc = "Watchdog Reset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdreset {
    #[doc = "0: Interrupt"]
    Interrupt = 0,
    #[doc = "1: Reset"]
    Reset = 1,
}
impl From<Wdreset> for bool {
    #[inline(always)]
    fn from(variant: Wdreset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDRESET` reader - Watchdog Reset Enable"]
pub type WdresetR = crate::BitReader<Wdreset>;
impl WdresetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdreset {
        match self.bits {
            false => Wdreset::Interrupt,
            true => Wdreset::Reset,
        }
    }
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn is_interrupt(&self) -> bool {
        *self == Wdreset::Interrupt
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == Wdreset::Reset
    }
}
#[doc = "Field `WDRESET` writer - Watchdog Reset Enable"]
pub type WdresetW<'a, REG> = crate::BitWriter<'a, REG, Wdreset>;
impl<'a, REG> WdresetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt"]
    #[inline(always)]
    pub fn interrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Wdreset::Interrupt)
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(Wdreset::Reset)
    }
}
#[doc = "Watchdog Timeout Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdtof {
    #[doc = "0: Watchdog event has not occurred."]
    Clear = 0,
    #[doc = "1: Watchdog event has occurred (causes a chip reset if WDRESET = 1)."]
    Reset = 1,
}
impl From<Wdtof> for bool {
    #[inline(always)]
    fn from(variant: Wdtof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDTOF` reader - Watchdog Timeout Flag"]
pub type WdtofR = crate::BitReader<Wdtof>;
impl WdtofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdtof {
        match self.bits {
            false => Wdtof::Clear,
            true => Wdtof::Reset,
        }
    }
    #[doc = "Watchdog event has not occurred."]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Wdtof::Clear
    }
    #[doc = "Watchdog event has occurred (causes a chip reset if WDRESET = 1)."]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == Wdtof::Reset
    }
}
#[doc = "Field `WDTOF` writer - Watchdog Timeout Flag"]
pub type WdtofW<'a, REG> = crate::BitWriter<'a, REG, Wdtof>;
impl<'a, REG> WdtofW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Watchdog event has not occurred."]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Wdtof::Clear)
    }
    #[doc = "Watchdog event has occurred (causes a chip reset if WDRESET = 1)."]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(Wdtof::Reset)
    }
}
#[doc = "Warning Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdint {
    #[doc = "0: No flag"]
    NoFlag = 0,
    #[doc = "1: Flag"]
    Flag = 1,
}
impl From<Wdint> for bool {
    #[inline(always)]
    fn from(variant: Wdint) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDINT` reader - Warning Interrupt Flag"]
pub type WdintR = crate::BitReader<Wdint>;
impl WdintR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdint {
        match self.bits {
            false => Wdint::NoFlag,
            true => Wdint::Flag,
        }
    }
    #[doc = "No flag"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Wdint::NoFlag
    }
    #[doc = "Flag"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Wdint::Flag
    }
}
#[doc = "Field `WDINT` writer - Warning Interrupt Flag"]
pub type WdintW<'a, REG> = crate::BitWriter1C<'a, REG, Wdint>;
impl<'a, REG> WdintW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No flag"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wdint::NoFlag)
    }
    #[doc = "Flag"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Wdint::Flag)
    }
}
#[doc = "Watchdog Update Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdprotect {
    #[doc = "0: Flexible"]
    Flexible = 0,
    #[doc = "1: Threshold"]
    Threshold = 1,
}
impl From<Wdprotect> for bool {
    #[inline(always)]
    fn from(variant: Wdprotect) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDPROTECT` reader - Watchdog Update Mode"]
pub type WdprotectR = crate::BitReader<Wdprotect>;
impl WdprotectR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdprotect {
        match self.bits {
            false => Wdprotect::Flexible,
            true => Wdprotect::Threshold,
        }
    }
    #[doc = "Flexible"]
    #[inline(always)]
    pub fn is_flexible(&self) -> bool {
        *self == Wdprotect::Flexible
    }
    #[doc = "Threshold"]
    #[inline(always)]
    pub fn is_threshold(&self) -> bool {
        *self == Wdprotect::Threshold
    }
}
#[doc = "Field `WDPROTECT` writer - Watchdog Update Mode"]
pub type WdprotectW<'a, REG> = crate::BitWriter<'a, REG, Wdprotect>;
impl<'a, REG> WdprotectW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Flexible"]
    #[inline(always)]
    pub fn flexible(self) -> &'a mut crate::W<REG> {
        self.variant(Wdprotect::Flexible)
    }
    #[doc = "Threshold"]
    #[inline(always)]
    pub fn threshold(self) -> &'a mut crate::W<REG> {
        self.variant(Wdprotect::Threshold)
    }
}
#[doc = "Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: No Lock"]
    NoLock = 0,
    #[doc = "1: Lock"]
    Lock = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - Lock"]
pub type LockR = crate::BitReader<Lock>;
impl LockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lock {
        match self.bits {
            false => Lock::NoLock,
            true => Lock::Lock,
        }
    }
    #[doc = "No Lock"]
    #[inline(always)]
    pub fn is_no_lock(&self) -> bool {
        *self == Lock::NoLock
    }
    #[doc = "Lock"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Lock::Lock
    }
}
#[doc = "Field `LOCK` writer - Lock"]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No Lock"]
    #[inline(always)]
    pub fn no_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::NoLock)
    }
    #[doc = "Lock"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock)
    }
}
#[doc = "Debug Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugEn {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<DebugEn> for bool {
    #[inline(always)]
    fn from(variant: DebugEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DEBUG_EN` reader - Debug Enable"]
pub type DebugEnR = crate::BitReader<DebugEn>;
impl DebugEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DebugEn {
        match self.bits {
            false => DebugEn::Disable,
            true => DebugEn::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DebugEn::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DebugEn::Enable
    }
}
#[doc = "Field `DEBUG_EN` writer - Debug Enable"]
pub type DebugEnW<'a, REG> = crate::BitWriter<'a, REG, DebugEn>;
impl<'a, REG> DebugEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DebugEn::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DebugEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Watchdog Enable"]
    #[inline(always)]
    pub fn wden(&self) -> WdenR {
        WdenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Watchdog Reset Enable"]
    #[inline(always)]
    pub fn wdreset(&self) -> WdresetR {
        WdresetR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Watchdog Timeout Flag"]
    #[inline(always)]
    pub fn wdtof(&self) -> WdtofR {
        WdtofR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Warning Interrupt Flag"]
    #[inline(always)]
    pub fn wdint(&self) -> WdintR {
        WdintR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Watchdog Update Mode"]
    #[inline(always)]
    pub fn wdprotect(&self) -> WdprotectR {
        WdprotectR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Lock"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Debug Enable"]
    #[inline(always)]
    pub fn debug_en(&self) -> DebugEnR {
        DebugEnR::new(((self.bits >> 6) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Watchdog Enable"]
    #[inline(always)]
    pub fn wden(&mut self) -> WdenW<ModSpec> {
        WdenW::new(self, 0)
    }
    #[doc = "Bit 1 - Watchdog Reset Enable"]
    #[inline(always)]
    pub fn wdreset(&mut self) -> WdresetW<ModSpec> {
        WdresetW::new(self, 1)
    }
    #[doc = "Bit 2 - Watchdog Timeout Flag"]
    #[inline(always)]
    pub fn wdtof(&mut self) -> WdtofW<ModSpec> {
        WdtofW::new(self, 2)
    }
    #[doc = "Bit 3 - Warning Interrupt Flag"]
    #[inline(always)]
    pub fn wdint(&mut self) -> WdintW<ModSpec> {
        WdintW::new(self, 3)
    }
    #[doc = "Bit 4 - Watchdog Update Mode"]
    #[inline(always)]
    pub fn wdprotect(&mut self) -> WdprotectW<ModSpec> {
        WdprotectW::new(self, 4)
    }
    #[doc = "Bit 5 - Lock"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<ModSpec> {
        LockW::new(self, 5)
    }
    #[doc = "Bit 6 - Debug Enable"]
    #[inline(always)]
    pub fn debug_en(&mut self) -> DebugEnW<ModSpec> {
        DebugEnW::new(self, 6)
    }
}
#[doc = "Mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mod_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mod_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModSpec;
impl crate::RegisterSpec for ModSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mod_::R`](R) reader structure"]
impl crate::Readable for ModSpec {}
#[doc = "`write(|w| ..)` method takes [`mod_::W`](W) writer structure"]
impl crate::Writable for ModSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x08;
}
#[doc = "`reset()` method sets MOD to value 0"]
impl crate::Resettable for ModSpec {}
