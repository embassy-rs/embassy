#[doc = "Register `REMAP` reader"]
pub type R = crate::R<RemapSpec>;
#[doc = "Register `REMAP` writer"]
pub type W = crate::W<RemapSpec>;
#[doc = "Remap Lock Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remaplk {
    #[doc = "0: Lock disabled: can write to REMAP"]
    LockDisabled = 0,
    #[doc = "1: Lock enabled: cannot write to REMAP"]
    LockEnabled = 1,
}
impl From<Remaplk> for bool {
    #[inline(always)]
    fn from(variant: Remaplk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REMAPLK` reader - Remap Lock Enable"]
pub type RemaplkR = crate::BitReader<Remaplk>;
impl RemaplkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Remaplk {
        match self.bits {
            false => Remaplk::LockDisabled,
            true => Remaplk::LockEnabled,
        }
    }
    #[doc = "Lock disabled: can write to REMAP"]
    #[inline(always)]
    pub fn is_lock_disabled(&self) -> bool {
        *self == Remaplk::LockDisabled
    }
    #[doc = "Lock enabled: cannot write to REMAP"]
    #[inline(always)]
    pub fn is_lock_enabled(&self) -> bool {
        *self == Remaplk::LockEnabled
    }
}
#[doc = "Field `REMAPLK` writer - Remap Lock Enable"]
pub type RemaplkW<'a, REG> = crate::BitWriter<'a, REG, Remaplk>;
impl<'a, REG> RemaplkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Lock disabled: can write to REMAP"]
    #[inline(always)]
    pub fn lock_disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Remaplk::LockDisabled)
    }
    #[doc = "Lock enabled: cannot write to REMAP"]
    #[inline(always)]
    pub fn lock_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Remaplk::LockEnabled)
    }
}
#[doc = "Field `LIM` reader - LIM Remapping Address"]
pub type LimR = crate::FieldReader;
#[doc = "Field `LIM` writer - LIM Remapping Address"]
pub type LimW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Field `LIMDP` reader - LIMDP Remapping Address"]
pub type LimdpR = crate::FieldReader;
#[doc = "Field `LIMDP` writer - LIMDP Remapping Address"]
pub type LimdpW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bit 0 - Remap Lock Enable"]
    #[inline(always)]
    pub fn remaplk(&self) -> RemaplkR {
        RemaplkR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 16:22 - LIM Remapping Address"]
    #[inline(always)]
    pub fn lim(&self) -> LimR {
        LimR::new(((self.bits >> 16) & 0x7f) as u8)
    }
    #[doc = "Bits 24:30 - LIMDP Remapping Address"]
    #[inline(always)]
    pub fn limdp(&self) -> LimdpR {
        LimdpR::new(((self.bits >> 24) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Remap Lock Enable"]
    #[inline(always)]
    pub fn remaplk(&mut self) -> RemaplkW<RemapSpec> {
        RemaplkW::new(self, 0)
    }
    #[doc = "Bits 16:22 - LIM Remapping Address"]
    #[inline(always)]
    pub fn lim(&mut self) -> LimW<RemapSpec> {
        LimW::new(self, 16)
    }
    #[doc = "Bits 24:30 - LIMDP Remapping Address"]
    #[inline(always)]
    pub fn limdp(&mut self) -> LimdpW<RemapSpec> {
        LimdpW::new(self, 24)
    }
}
#[doc = "Data Remap\n\nYou can [`read`](crate::Reg::read) this register and get [`remap::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`remap::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RemapSpec;
impl crate::RegisterSpec for RemapSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`remap::R`](R) reader structure"]
impl crate::Readable for RemapSpec {}
#[doc = "`write(|w| ..)` method takes [`remap::W`](W) writer structure"]
impl crate::Writable for RemapSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets REMAP to value 0"]
impl crate::Resettable for RemapSpec {}
