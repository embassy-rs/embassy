#[doc = "Register `DEBUG_LOCK_EN` reader"]
pub type R = crate::R<DebugLockEnSpec>;
#[doc = "Register `DEBUG_LOCK_EN` writer"]
pub type W = crate::W<DebugLockEnSpec>;
#[doc = "Controls write access to the security registers\n\nValue on reset: 10"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LockAll {
    #[doc = "0: Any other value than b1010: disables write access to all registers"]
    Disable = 0,
    #[doc = "10: Enables write access to all registers"]
    Enable = 10,
}
impl From<LockAll> for u8 {
    #[inline(always)]
    fn from(variant: LockAll) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for LockAll {
    type Ux = u8;
}
impl crate::IsEnum for LockAll {}
#[doc = "Field `LOCK_ALL` reader - Controls write access to the security registers"]
pub type LockAllR = crate::FieldReader<LockAll>;
impl LockAllR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<LockAll> {
        match self.bits {
            0 => Some(LockAll::Disable),
            10 => Some(LockAll::Enable),
            _ => None,
        }
    }
    #[doc = "Any other value than b1010: disables write access to all registers"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == LockAll::Disable
    }
    #[doc = "Enables write access to all registers"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == LockAll::Enable
    }
}
#[doc = "Field `LOCK_ALL` writer - Controls write access to the security registers"]
pub type LockAllW<'a, REG> = crate::FieldWriter<'a, REG, 4, LockAll>;
impl<'a, REG> LockAllW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Any other value than b1010: disables write access to all registers"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(LockAll::Disable)
    }
    #[doc = "Enables write access to all registers"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(LockAll::Enable)
    }
}
impl R {
    #[doc = "Bits 0:3 - Controls write access to the security registers"]
    #[inline(always)]
    pub fn lock_all(&self) -> LockAllR {
        LockAllR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Controls write access to the security registers"]
    #[inline(always)]
    pub fn lock_all(&mut self) -> LockAllW<DebugLockEnSpec> {
        LockAllW::new(self, 0)
    }
}
#[doc = "Control Write Access to Security\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_lock_en::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_lock_en::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DebugLockEnSpec;
impl crate::RegisterSpec for DebugLockEnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`debug_lock_en::R`](R) reader structure"]
impl crate::Readable for DebugLockEnSpec {}
#[doc = "`write(|w| ..)` method takes [`debug_lock_en::W`](W) writer structure"]
impl crate::Writable for DebugLockEnSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DEBUG_LOCK_EN to value 0x0a"]
impl crate::Resettable for DebugLockEnSpec {
    const RESET_VALUE: u32 = 0x0a;
}
