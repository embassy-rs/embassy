#[doc = "Register `SRAM_XEN` reader"]
pub type R = crate::R<SramXenSpec>;
#[doc = "Register `SRAM_XEN` writer"]
pub type W = crate::W<SramXenSpec>;
#[doc = "RAMX0 Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ramx0Xen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<Ramx0Xen> for bool {
    #[inline(always)]
    fn from(variant: Ramx0Xen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMX0_XEN` reader - RAMX0 Execute permission control."]
pub type Ramx0XenR = crate::BitReader<Ramx0Xen>;
impl Ramx0XenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ramx0Xen {
        match self.bits {
            false => Ramx0Xen::Disable,
            true => Ramx0Xen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ramx0Xen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ramx0Xen::Enable
    }
}
#[doc = "Field `RAMX0_XEN` writer - RAMX0 Execute permission control."]
pub type Ramx0XenW<'a, REG> = crate::BitWriter<'a, REG, Ramx0Xen>;
impl<'a, REG> Ramx0XenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ramx0Xen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ramx0Xen::Enable)
    }
}
#[doc = "RAMX1 Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ramx1Xen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<Ramx1Xen> for bool {
    #[inline(always)]
    fn from(variant: Ramx1Xen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMX1_XEN` reader - RAMX1 Execute permission control."]
pub type Ramx1XenR = crate::BitReader<Ramx1Xen>;
impl Ramx1XenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ramx1Xen {
        match self.bits {
            false => Ramx1Xen::Disable,
            true => Ramx1Xen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ramx1Xen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ramx1Xen::Enable
    }
}
#[doc = "Field `RAMX1_XEN` writer - RAMX1 Execute permission control."]
pub type Ramx1XenW<'a, REG> = crate::BitWriter<'a, REG, Ramx1Xen>;
impl<'a, REG> Ramx1XenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ramx1Xen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ramx1Xen::Enable)
    }
}
#[doc = "RAMA0 Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rama0Xen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<Rama0Xen> for bool {
    #[inline(always)]
    fn from(variant: Rama0Xen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMA0_XEN` reader - RAMA0 Execute permission control."]
pub type Rama0XenR = crate::BitReader<Rama0Xen>;
impl Rama0XenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rama0Xen {
        match self.bits {
            false => Rama0Xen::Disable,
            true => Rama0Xen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rama0Xen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rama0Xen::Enable
    }
}
#[doc = "Field `RAMA0_XEN` writer - RAMA0 Execute permission control."]
pub type Rama0XenW<'a, REG> = crate::BitWriter<'a, REG, Rama0Xen>;
impl<'a, REG> Rama0XenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rama0Xen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rama0Xen::Enable)
    }
}
#[doc = "RAMAx (excepts RAMA0) Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rama1Xen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<Rama1Xen> for bool {
    #[inline(always)]
    fn from(variant: Rama1Xen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMA1_XEN` reader - RAMAx (excepts RAMA0) Execute permission control."]
pub type Rama1XenR = crate::BitReader<Rama1Xen>;
impl Rama1XenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rama1Xen {
        match self.bits {
            false => Rama1Xen::Disable,
            true => Rama1Xen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rama1Xen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rama1Xen::Enable
    }
}
#[doc = "Field `RAMA1_XEN` writer - RAMAx (excepts RAMA0) Execute permission control."]
pub type Rama1XenW<'a, REG> = crate::BitWriter<'a, REG, Rama1Xen>;
impl<'a, REG> Rama1XenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rama1Xen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rama1Xen::Enable)
    }
}
#[doc = "RAMBx Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RambXen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<RambXen> for bool {
    #[inline(always)]
    fn from(variant: RambXen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMB_XEN` reader - RAMBx Execute permission control."]
pub type RambXenR = crate::BitReader<RambXen>;
impl RambXenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RambXen {
        match self.bits {
            false => RambXen::Disable,
            true => RambXen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RambXen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RambXen::Enable
    }
}
#[doc = "Field `RAMB_XEN` writer - RAMBx Execute permission control."]
pub type RambXenW<'a, REG> = crate::BitWriter<'a, REG, RambXen>;
impl<'a, REG> RambXenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RambXen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RambXen::Enable)
    }
}
#[doc = "RAMCx Execute permission control.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RamcXen {
    #[doc = "0: Execute permission is disabled, R/W are enabled."]
    Disable = 0,
    #[doc = "1: Execute permission is enabled, R/W/X are enabled."]
    Enable = 1,
}
impl From<RamcXen> for bool {
    #[inline(always)]
    fn from(variant: RamcXen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMC_XEN` reader - RAMCx Execute permission control."]
pub type RamcXenR = crate::BitReader<RamcXen>;
impl RamcXenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RamcXen {
        match self.bits {
            false => RamcXen::Disable,
            true => RamcXen::Enable,
        }
    }
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RamcXen::Disable
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RamcXen::Enable
    }
}
#[doc = "Field `RAMC_XEN` writer - RAMCx Execute permission control."]
pub type RamcXenW<'a, REG> = crate::BitWriter<'a, REG, RamcXen>;
impl<'a, REG> RamcXenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute permission is disabled, R/W are enabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RamcXen::Disable)
    }
    #[doc = "Execute permission is enabled, R/W/X are enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RamcXen::Enable)
    }
}
#[doc = "This 1-bit field provides a mechanism to limit writes to the this register (and SRAM_XEN_DP) to protect its contents. Once set, this bit remains asserted until a system reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: This register is not locked and can be altered."]
    Lock0 = 0,
    #[doc = "1: This register is locked and cannot be altered."]
    Lock1 = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - This 1-bit field provides a mechanism to limit writes to the this register (and SRAM_XEN_DP) to protect its contents. Once set, this bit remains asserted until a system reset."]
pub type LockR = crate::BitReader<Lock>;
impl LockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lock {
        match self.bits {
            false => Lock::Lock0,
            true => Lock::Lock1,
        }
    }
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn is_lock_0(&self) -> bool {
        *self == Lock::Lock0
    }
    #[doc = "This register is locked and cannot be altered."]
    #[inline(always)]
    pub fn is_lock_1(&self) -> bool {
        *self == Lock::Lock1
    }
}
#[doc = "Field `LOCK` writer - This 1-bit field provides a mechanism to limit writes to the this register (and SRAM_XEN_DP) to protect its contents. Once set, this bit remains asserted until a system reset."]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn lock_0(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock0)
    }
    #[doc = "This register is locked and cannot be altered."]
    #[inline(always)]
    pub fn lock_1(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock1)
    }
}
impl R {
    #[doc = "Bit 0 - RAMX0 Execute permission control."]
    #[inline(always)]
    pub fn ramx0_xen(&self) -> Ramx0XenR {
        Ramx0XenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RAMX1 Execute permission control."]
    #[inline(always)]
    pub fn ramx1_xen(&self) -> Ramx1XenR {
        Ramx1XenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RAMA0 Execute permission control."]
    #[inline(always)]
    pub fn rama0_xen(&self) -> Rama0XenR {
        Rama0XenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - RAMAx (excepts RAMA0) Execute permission control."]
    #[inline(always)]
    pub fn rama1_xen(&self) -> Rama1XenR {
        Rama1XenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - RAMBx Execute permission control."]
    #[inline(always)]
    pub fn ramb_xen(&self) -> RambXenR {
        RambXenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - RAMCx Execute permission control."]
    #[inline(always)]
    pub fn ramc_xen(&self) -> RamcXenR {
        RamcXenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register (and SRAM_XEN_DP) to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RAMX0 Execute permission control."]
    #[inline(always)]
    pub fn ramx0_xen(&mut self) -> Ramx0XenW<SramXenSpec> {
        Ramx0XenW::new(self, 0)
    }
    #[doc = "Bit 1 - RAMX1 Execute permission control."]
    #[inline(always)]
    pub fn ramx1_xen(&mut self) -> Ramx1XenW<SramXenSpec> {
        Ramx1XenW::new(self, 1)
    }
    #[doc = "Bit 2 - RAMA0 Execute permission control."]
    #[inline(always)]
    pub fn rama0_xen(&mut self) -> Rama0XenW<SramXenSpec> {
        Rama0XenW::new(self, 2)
    }
    #[doc = "Bit 3 - RAMAx (excepts RAMA0) Execute permission control."]
    #[inline(always)]
    pub fn rama1_xen(&mut self) -> Rama1XenW<SramXenSpec> {
        Rama1XenW::new(self, 3)
    }
    #[doc = "Bit 4 - RAMBx Execute permission control."]
    #[inline(always)]
    pub fn ramb_xen(&mut self) -> RambXenW<SramXenSpec> {
        RambXenW::new(self, 4)
    }
    #[doc = "Bit 5 - RAMCx Execute permission control."]
    #[inline(always)]
    pub fn ramc_xen(&mut self) -> RamcXenW<SramXenSpec> {
        RamcXenW::new(self, 5)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register (and SRAM_XEN_DP) to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<SramXenSpec> {
        LockW::new(self, 31)
    }
}
#[doc = "RAM XEN Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sram_xen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sram_xen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SramXenSpec;
impl crate::RegisterSpec for SramXenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sram_xen::R`](R) reader structure"]
impl crate::Readable for SramXenSpec {}
#[doc = "`write(|w| ..)` method takes [`sram_xen::W`](W) writer structure"]
impl crate::Writable for SramXenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SRAM_XEN to value 0"]
impl crate::Resettable for SramXenSpec {}
