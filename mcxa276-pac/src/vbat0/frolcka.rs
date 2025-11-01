#[doc = "Register `FROLCKA` reader"]
pub type R = crate::R<FrolckaSpec>;
#[doc = "Register `FROLCKA` writer"]
pub type W = crate::W<FrolckaSpec>;
#[doc = "Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: Do not block"]
    Disable = 0,
    #[doc = "1: Block"]
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
    #[doc = "Do not block"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lock::Disable
    }
    #[doc = "Block"]
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
    #[doc = "Do not block"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Disable)
    }
    #[doc = "Block"]
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
    pub fn lock(&mut self) -> LockW<FrolckaSpec> {
        LockW::new(self, 0)
    }
}
#[doc = "FRO16K Lock A\n\nYou can [`read`](crate::Reg::read) this register and get [`frolcka::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frolcka::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrolckaSpec;
impl crate::RegisterSpec for FrolckaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frolcka::R`](R) reader structure"]
impl crate::Readable for FrolckaSpec {}
#[doc = "`write(|w| ..)` method takes [`frolcka::W`](W) writer structure"]
impl crate::Writable for FrolckaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FROLCKA to value 0"]
impl crate::Resettable for FrolckaSpec {}
