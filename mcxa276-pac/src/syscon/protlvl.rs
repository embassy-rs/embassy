#[doc = "Register `PROTLVL` reader"]
pub type R = crate::R<ProtlvlSpec>;
#[doc = "Register `PROTLVL` writer"]
pub type W = crate::W<ProtlvlSpec>;
#[doc = "Control privileged access of EIM, ERM, Flexcan, MBC, SCG.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priv {
    #[doc = "0: privileged access is disabled. the peripherals could be access in user mode."]
    Disable = 0,
    #[doc = "1: privileged access is enabled. the peripherals could be access in privilege mode."]
    Enable = 1,
}
impl From<Priv> for bool {
    #[inline(always)]
    fn from(variant: Priv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PRIV` reader - Control privileged access of EIM, ERM, Flexcan, MBC, SCG."]
pub type PrivR = crate::BitReader<Priv>;
impl PrivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Priv {
        match self.bits {
            false => Priv::Disable,
            true => Priv::Enable,
        }
    }
    #[doc = "privileged access is disabled. the peripherals could be access in user mode."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Priv::Disable
    }
    #[doc = "privileged access is enabled. the peripherals could be access in privilege mode."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Priv::Enable
    }
}
#[doc = "Field `PRIV` writer - Control privileged access of EIM, ERM, Flexcan, MBC, SCG."]
pub type PrivW<'a, REG> = crate::BitWriter<'a, REG, Priv>;
impl<'a, REG> PrivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "privileged access is disabled. the peripherals could be access in user mode."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Priv::Disable)
    }
    #[doc = "privileged access is enabled. the peripherals could be access in privilege mode."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Priv::Enable)
    }
}
#[doc = "Control write access to Nonsecure MPU memory regions.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locknsmpu {
    #[doc = "0: Unlock these registers. privileged access to Nonsecure MPU memory regions is allowed."]
    Enable = 0,
    #[doc = "1: Disable writes to the MPU_CTRL_NS, MPU_RNR_NS, MPU_RBAR_NS, MPU_RLAR_NS, MPU_RBAR_A_NSn and MPU_RLAR_A_NSn. All writes to the registers are ignored."]
    Disable = 1,
}
impl From<Locknsmpu> for bool {
    #[inline(always)]
    fn from(variant: Locknsmpu) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCKNSMPU` reader - Control write access to Nonsecure MPU memory regions."]
pub type LocknsmpuR = crate::BitReader<Locknsmpu>;
impl LocknsmpuR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Locknsmpu {
        match self.bits {
            false => Locknsmpu::Enable,
            true => Locknsmpu::Disable,
        }
    }
    #[doc = "Unlock these registers. privileged access to Nonsecure MPU memory regions is allowed."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Locknsmpu::Enable
    }
    #[doc = "Disable writes to the MPU_CTRL_NS, MPU_RNR_NS, MPU_RBAR_NS, MPU_RLAR_NS, MPU_RBAR_A_NSn and MPU_RLAR_A_NSn. All writes to the registers are ignored."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Locknsmpu::Disable
    }
}
#[doc = "Field `LOCKNSMPU` writer - Control write access to Nonsecure MPU memory regions."]
pub type LocknsmpuW<'a, REG> = crate::BitWriter<'a, REG, Locknsmpu>;
impl<'a, REG> LocknsmpuW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Unlock these registers. privileged access to Nonsecure MPU memory regions is allowed."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Locknsmpu::Enable)
    }
    #[doc = "Disable writes to the MPU_CTRL_NS, MPU_RNR_NS, MPU_RBAR_NS, MPU_RLAR_NS, MPU_RBAR_A_NSn and MPU_RLAR_A_NSn. All writes to the registers are ignored."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Locknsmpu::Disable)
    }
}
#[doc = "This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: This register is not locked and can be altered."]
    Lock0 = 0,
    #[doc = "1: This register is locked and cannot be altered until a system reset."]
    Lock1 = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
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
    #[doc = "This register is locked and cannot be altered until a system reset."]
    #[inline(always)]
    pub fn is_lock_1(&self) -> bool {
        *self == Lock::Lock1
    }
}
#[doc = "Field `LOCK` writer - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
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
    #[doc = "This register is locked and cannot be altered until a system reset."]
    #[inline(always)]
    pub fn lock_1(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock1)
    }
}
impl R {
    #[doc = "Bit 0 - Control privileged access of EIM, ERM, Flexcan, MBC, SCG."]
    #[inline(always)]
    pub fn priv_(&self) -> PrivR {
        PrivR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 16 - Control write access to Nonsecure MPU memory regions."]
    #[inline(always)]
    pub fn locknsmpu(&self) -> LocknsmpuR {
        LocknsmpuR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Control privileged access of EIM, ERM, Flexcan, MBC, SCG."]
    #[inline(always)]
    pub fn priv_(&mut self) -> PrivW<ProtlvlSpec> {
        PrivW::new(self, 0)
    }
    #[doc = "Bit 16 - Control write access to Nonsecure MPU memory regions."]
    #[inline(always)]
    pub fn locknsmpu(&mut self) -> LocknsmpuW<ProtlvlSpec> {
        LocknsmpuW::new(self, 16)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<ProtlvlSpec> {
        LockW::new(self, 31)
    }
}
#[doc = "Protect Level Control\n\nYou can [`read`](crate::Reg::read) this register and get [`protlvl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`protlvl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ProtlvlSpec;
impl crate::RegisterSpec for ProtlvlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`protlvl::R`](R) reader structure"]
impl crate::Readable for ProtlvlSpec {}
#[doc = "`write(|w| ..)` method takes [`protlvl::W`](W) writer structure"]
impl crate::Writable for ProtlvlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PROTLVL to value 0"]
impl crate::Resettable for ProtlvlSpec {}
