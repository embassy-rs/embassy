#[doc = "Register `WAKLCKA` reader"]
pub type R = crate::R<WaklckaSpec>;
#[doc = "Register `WAKLCKA` writer"]
pub type W = crate::W<WaklckaSpec>;
#[doc = "Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: Lock is disabled"]
    Disable = 0,
    #[doc = "1: Lock is enabled"]
    Enable = 1,
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
            false => Lock::Disable,
            true => Lock::Enable,
        }
    }
    #[doc = "Lock is disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lock::Disable
    }
    #[doc = "Lock is enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lock::Enable
    }
}
#[doc = "Field `LOCK` writer - Lock"]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Lock is disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Disable)
    }
    #[doc = "Lock is enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Lock"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Lock"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<WaklckaSpec> {
        LockW::new(self, 0)
    }
}
#[doc = "Wakeup Lock A\n\nYou can [`read`](crate::Reg::read) this register and get [`waklcka::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`waklcka::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WaklckaSpec;
impl crate::RegisterSpec for WaklckaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`waklcka::R`](R) reader structure"]
impl crate::Readable for WaklckaSpec {}
#[doc = "`write(|w| ..)` method takes [`waklcka::W`](W) writer structure"]
impl crate::Writable for WaklckaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WAKLCKA to value 0"]
impl crate::Resettable for WaklckaSpec {}
